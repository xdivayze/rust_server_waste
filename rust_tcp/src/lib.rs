use std::fs::File;
use std::io::Write;
use std::string::String;
use image::GenericImageView;


use log::info;
use rscam::Camera;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time;
use uuid::Uuid;

//TODO add enumeration
//TODO add real time data handling
//TODO optimize


#[derive(Debug)]
pub enum WasteType {
    Init,
    Plastic,
    Carton,
    Textile,
    Food,
    Paper,
    Glass,
    Metal,
    Packing,
    UmaThurman,
    Medical,
    Battery,
    Hazardous,
    Organic,
    Electronic,
    Wood,
    Mixed,
    Other,
}

pub async fn send_data_to_tcp(stream: &mut TcpStream, data: &[u8]) -> Result<String, std::io::Error> {
    info!("Sending data to server ---> {}", String::from_utf8_lossy(data));
    stream.write_all(data).await?;

    let mut buf = [0; 1024];
    let response = stream.read(&mut buf).await?;
    let response = String::from_utf8(Vec::from(&buf[..response])).unwrap();
    info!("Response from server: {}", &response);
    Ok(response.parse().unwrap())
}

pub fn take_picture<'a>(camera:&'a Camera, path: &'a String) -> Result<&'a String, rscam::Error> {
    let time = time::Instant::now();
    let temp_path = format!("{}.jpg", path);
    let frame = camera.capture()?;
    let mut file = File::create(&temp_path)?;
    file.write_all(&frame[..])?;
    info!("Picture taken in {}ms", time.elapsed().as_millis());
    Ok(path)
}


pub async fn split_picture_horizontally(split_count: u32, waste: &Waste, stream: &mut TcpStream) -> Result<Vec<Waste>, std::io::Error> {
    let mut waste_vec = Vec::new();
    let image = image::open(format!("{}.jpg", waste.image_path)).unwrap();
    let (width, height) = image.dimensions();
    let split_width = width / split_count;
    let mut x = 0;
    for i in 0..split_count {
        let image = image.clone();
        let cropped_image = image.crop_imm(x, 0, split_width, height);
        let mut path = String::from("../");
        path.push_str(&waste.image_path);
        path.push_str(&i.to_string());
        path.push_str(".jpg");
        cropped_image.save(path.clone()).unwrap();
        let waste = Waste::new(waste.id, {
            let mut python_path = String::from("images/");
            python_path.push_str(&path.split("/").last().unwrap().to_string());
            python_path
        }, waste.number*10+i, stream).await;
        waste_vec.push(waste);
        x += split_width;
    }
    Ok(waste_vec)
}

#[derive(Debug)]
pub struct Waste {
    pub waste_type: WasteType,
    pub id: Uuid,
    pub image_path: String,
    pub number: u32,
    pub probability: f32,
}


impl Waste {
    async fn new(id: Uuid, image_path: String, number: u32, stream: &mut TcpStream) -> Waste {
        let results = send_data_to_tcp(stream, image_path.as_bytes()).await.unwrap().to_ascii_lowercase().as_str().split(",").map(|s| s.to_string()).collect::<Vec<String>>();
        Waste {
            waste_type: {
                match results[0].trim() {
                    "plastic" => WasteType::Plastic,
                    "carton" => WasteType::Carton,
                    "textile" => WasteType::Textile,
                    "food" => WasteType::Food,
                    "paper" => WasteType::Paper,
                    "glass" => WasteType::Glass,
                    "metal" => WasteType::Metal,
                    "packing" => WasteType::Packing,
                    "uma thurman" => WasteType::UmaThurman,
                    "medical" => WasteType::Medical,
                    "battery" => WasteType::Battery,
                    "hazardous" => WasteType::Hazardous,
                    "organic" => WasteType::Organic,
                    "electronic" => WasteType::Electronic,
                    "wood" => WasteType::Wood,
                    "mixed" => WasteType::Mixed,
                    "other" => WasteType::Other,
                    _ => WasteType::Other,
                }
            },
            id,
            image_path: format!("images/{}", id.to_string()),
            number,
            probability: results[1].trim().parse().unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio::net::TcpStream;
    use uuid::Uuid;

    use super::*;

    #[tokio::test]
    async fn test_split_picture_horizontally() {
        env_logger::init();

        let mut camera = Camera::new("/dev/video0").unwrap_or_else(|e| {
            panic!("Failed to open /dev/video0: {}", e);
        });

        camera.start(&rscam::Config {
            interval: (1, 30),
            resolution: (1280, 720),
            format: b"MJPG",
            ..Default::default()
        }).unwrap();


        let mut stream = TcpStream::connect("127.0.0.1:8080").await.unwrap();
        let mut images_dir = String::from("images");

        let uuid = Uuid::new_v4();

        let path = take_picture(&camera, {
            images_dir.push_str("/");
            images_dir.push_str(&uuid.to_string());
            images_dir
        }).unwrap();

        camera.stop().unwrap();

        let waste = Waste {
            waste_type: WasteType::Plastic,
            id: uuid,
            image_path: path,
            number: 0,
            probability: 0.0,
        };
        let images = split_picture_horizontally(3, waste, &mut stream).await.unwrap();
        for image in &images {
            info!("{:?}", image);
        }

        assert_eq!(images.len(), 3);
    }
}
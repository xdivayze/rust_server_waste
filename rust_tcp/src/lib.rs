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
//TODO add error handling
//TODO optimize

#[derive(Debug)]
pub enum WasteType {
    Init,
    Paper,
    Cardboard,
    Glass,
    Plastics,
    Metal,
    Wood,
    Leather,
    Rubber,
    Hazardous,
    Compost,
    Residual,
    Organic,
    Battery,
    Electronic,
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

pub fn take_picture(camera: &Camera, path: String) -> Result<String, rscam::Error> {
    let time = time::Instant::now();
    let temp_path = format!("{}.jpg", path);
    let frame = camera.capture()?;
    let mut file = File::create(&temp_path)?;
    file.write_all(&frame[..])?;
    info!("Picture taken in {}ms", time.elapsed().as_millis());
    Ok(path)
}


pub async fn split_picture_horizontally(split_count: u32, waste: Waste, stream: &mut TcpStream) -> Result<Vec<Waste>, std::io::Error> {
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
        }, waste.number + i, stream).await;
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
                match &results[0].trim()[0..2] {
                    "or" => WasteType::Organic,
                    "pa" => WasteType::Paper,
                    "ca" => WasteType::Cardboard,
                    "gl" => WasteType::Glass,
                    "pl" => WasteType::Plastics,
                    "me" => WasteType::Metal,
                    "wo" => WasteType::Wood,
                    "le" => WasteType::Leather,
                    "ru" => WasteType::Rubber,
                    "ha" => WasteType::Hazardous,
                    "co" => WasteType::Compost,
                    "re" => WasteType::Residual,
                    "ba" => WasteType::Battery,
                    "el" => WasteType::Electronic,
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
    use std::fs;
    use std::path::Path;
    use json::object;
    use tokio::net::TcpStream;
    use uuid::Uuid;

    use super::*;

    #[tokio::test]
    async fn test_accuracy() {
        env_logger::init();
        assert!(Path::new("test_cache.jsonl").exists() || fs::File::create("test_cache.jsonl").is_ok());
        let mut cache_file = fs::OpenOptions::new().append(true).write(true).open("test_cache.jsonl").unwrap();


        let master_directory_iterator = fs::read_dir("/home/cavej/Downloads/garbage_collection_data/garbage_classification").unwrap();

        let stream = &mut TcpStream::connect("127.0.0.1:8080").await.unwrap();

        let time = time::Instant::now();

        let mut count = 0;

        for dir in master_directory_iterator {
            let dir = dir.unwrap();
            for image in dir.path().read_dir().unwrap() {
                count += 1;
                if count % 10 == 0 {
                    let image = image.unwrap();
                    let image_path = image.path();
                    let mut waste = Waste::new(Uuid::new_v4(), image_path.to_str().unwrap().to_string(), 0, stream).await;
                    waste.image_path = image.path().to_str().unwrap().to_string();
                    let data = object! {
                    waste_type_actual: dir.file_name().into_string().unwrap(),
                    waste_type_result:  format!("{:?}",waste.waste_type),
                    id: waste.id.to_string(),
                    image_path: waste.image_path,
                    number: waste.number,
                    probability: waste.probability,
                    };
                    cache_file.write_all(format!("{}\n", data.dump()).as_bytes()).unwrap();
                } else { continue; }

            }
        }
        info!("Accuracy test took {}ms", time.elapsed().as_millis());
        assert!(Path::new("test_cache.jsonl").exists());
    }

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
            waste_type: WasteType::Init,
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
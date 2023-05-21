use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use json::{Null, object};
use rscam::Camera;
use tokio::fs::File;
use tokio::net::TcpStream;
use uuid::Uuid;

use rust_tcp::{split_picture_horizontally, take_picture, Waste, WasteType};

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    const SPLIT_COUNT: u32 = 3;
    assert!(Path::new("images").exists() || fs::create_dir("images").is_ok());
    assert!(Path::new("cache.jsonl").exists() || fs::File::create("cache.jsonl").is_ok());
    assert!(Path::new("../images").exists() || fs::create_dir("../images").is_ok());
    let mut cache_file = OpenOptions::new().write(true).append(true).open("cache.jsonl").unwrap();

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


    let handle = tokio::spawn(async move {
        let id = Uuid::new_v4();
        let image_path = format!("images/{}", id.to_string());
        let path = take_picture(&camera, image_path).unwrap_or_else(|e| {
            panic!("Exception taking picture, {}", e);
        });
        let master_waste = Waste {
            waste_type: WasteType::Init,
            id,
            image_path: path,
            number: 0,
            probability: 0.0,
        };

        let mut data = object! {
            waste_type: format!("{:?}",master_waste.waste_type),
            id: master_waste.id.to_string(),
            image_path: master_waste.image_path.clone(),
            number: master_waste.number,
            probability: master_waste.probability,
            children: Null,
        };

        let waste_array = split_picture_horizontally(SPLIT_COUNT, master_waste, &mut stream).await.unwrap();
        let mut children = vec![];
        for waste_c in &waste_array {
            children.push(object! {
                waste_type: format!("{:?}",waste_c.waste_type),
                id: waste_c.id.to_string(),
                image_path: waste_c.image_path.clone(),
                number: waste_c.number,
                probability: waste_c.probability,
            });
        }
        data.insert("children", children.clone()).unwrap();
        assert!(cache_file.write_all(data.dump().as_bytes()).is_ok());

    });

    handle.await.unwrap();
    Ok(())
}


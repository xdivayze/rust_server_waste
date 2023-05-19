use std::io::{Read, Write};
use std::net::TcpStream;
use rscam::{Camera, Config};
use rustTCP::send_data_async;

//TODO spawn threads to handle multiple images sent to socket server
//TODO implement video to image conversion

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let mut camera = Camera::new("/dev/video0").unwrap();
    camera.start(&Config {
        interval: (1, 15),
        resolution: (640,480),
        format: b"MJPG",
        ..Default::default()
    }).unwrap_or_else(|e| {
        panic!("Failed to start camera: {}", e);
    });

    for i in 1..10 {
        tokio::spawn(async move {
            println!("Hello from a thread! tcp returned {}", send_data_async(format!("images/{}.jpg", i), &mut camera).await.unwrap());
        });
    }

    Ok(())

}

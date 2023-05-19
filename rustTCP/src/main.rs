use log::info;
use rscam::Camera;
use tokio::net::TcpStream;
use rustTCP::{send_data_to_tcp, take_picture};

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let mut camera = Camera::new("/dev/video0").unwrap_or_else(|e| {
        panic!("Failed to open /dev/video0: {}", e);
    });
    let mut stream = TcpStream::connect("127.0.0.1:8080").await.unwrap();

    camera.start(&rscam::Config {
        interval: (1, 30),
        resolution: (1280, 720),
        format: b"MJPG",
        ..Default::default()
    }).unwrap();

    let handle = tokio::spawn(async move {
        let data = send_data_to_tcp(&mut stream,format!("{}", take_picture(&camera, String::from("../../images/test.jpg")).unwrap_or_else(|e| {
            panic!("Failed to capture: {}", e)
        })).as_bytes()).await.unwrap();
        info!("{}",data);
        data
    });

    handle.await.unwrap();
    Ok(())
}
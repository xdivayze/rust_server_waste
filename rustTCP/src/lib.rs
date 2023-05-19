use std::io::{Read, Write};
use std::net::TcpStream;
use rscam::{Camera, Config};
use std::time;

pub async fn send_data_async(string: String, camera: &mut Camera) -> Result<String, Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect("127.0.0.1:65432").unwrap_or_else(|e| {
        panic!("Failed to connect: {}", e);
    });

    let data = take_webcam_feed(camera)?;
    stream.write(&data[..]).unwrap();

    //print the response
    let mut data = [0 as u8; 50]; // using 50 byte buffer

    let stream_returned = stream.read(&mut data).unwrap();
    let stream_returned = String::from_utf8_lossy(&data[..stream_returned]).to_string();

    println!("{} --- inside lib.rs", stream_returned);

    Ok(stream_returned)
}

pub fn take_webcam_feed(camera: &mut Camera) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    //get time elapsed
    let time = time::Instant::now();

    let frame = camera.capture().unwrap();

    Ok({
        println!("time elapsed: {}", (time::Instant::now() - time).as_millis());
        frame.to_vec()
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use super::*;

    #[test]
    fn test_send_data_async() {
        let mut camera = Camera::new("/dev/video0").unwrap();
        camera.start(&Config {
            interval: (1, 15),
            resolution: (640,480),
            format: b"MJPG",
            ..Default::default()
        }).unwrap_or_else(|e| {
            panic!("Failed to start camera: {}", e);
        });
        let image_bytes = take_webcam_feed(&mut camera).unwrap_or_else(|e| {
            panic!("Failed to connect: {}", e);
        });
        let mut file = fs::File::create("images/img_1.png").unwrap();
        file.write_all(&image_bytes).unwrap();
        assert!(fs::metadata("images/img_1.png").is_ok());
    }
}
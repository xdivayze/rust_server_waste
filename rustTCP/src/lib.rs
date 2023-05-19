use std::fs::File;
use std::io::Write;
use log::info;

use rscam::Camera;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

//TODO add enumeration
//TODO add real time data handling
//TODO add error handling
//TODO add caching
//TODO optimize

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
    let frame = camera.capture()?;
    let mut file = File::create(&path)?;
    file.write_all(&frame[..])?;
    Ok(format!("images/{}",path.split("/").last().unwrap().to_string()))
}



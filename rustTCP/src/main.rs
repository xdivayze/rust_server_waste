use std::io::{Read, Write};
use std::net::TcpStream;

//TODO spawn threads to handle multiple iamges sent to socket server
//TODO implement video to image conversion

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:65432").unwrap_or_else(|e| {
        panic!("Failed to connect: {}", e);
    });

    let data = "images/img_1.png";
    stream.write(data.as_bytes()).unwrap();

    //print the response
    let mut data = [0 as u8; 50]; // using 50 byte buffer
    match stream.read(&mut data) {
        Ok(_) => {
            println!("Response: {}", String::from_utf8_lossy(&data[..]));
        }
        Err(e) => {
            println!("Failed to receive data: {}", e);
        }
    }
}

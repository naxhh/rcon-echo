use bytes::{BufMut, BytesMut};
use std::error::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[derive(Debug)]
struct RconPacket {
    id: i32,
    r#type: i32,
    body: String,
}

impl RconPacket {
    fn new(id: i32, r#type: i32, body: String) -> Self {
        Self { id, r#type, body }
    }

    fn serialize(&self) -> Vec<u8> {
        let mut buffer = BytesMut::new();
        buffer.put_i32_le(self.id);
        buffer.put_i32_le(self.r#type);
        buffer.put_slice(self.body.as_bytes());
        buffer.put_u8(0); // Null terminator for body
        buffer.put_u8(0); // Null terminator for packet

        let size = buffer.len() as i32;
        let mut packet = BytesMut::new();
        packet.put_i32_le(size);
        packet.extend_from_slice(&buffer);

        packet.to_vec()
    }

    fn deserialize(data: &[u8]) -> Result<Self, Box<dyn Error>> {
        if data.len() < 10 {
            return Err("Invalid packet size".into());
        }

        let _size = i32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let id = i32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let r#type = i32::from_le_bytes([data[8], data[9], data[10], data[11]]);
        let body = String::from_utf8(data[12..data.len() - 2].to_vec())?;

        Ok(Self::new(id, r#type, body))
    }
}

async fn handle_client(
    mut socket: tokio::net::TcpStream,
    password: String,
) -> Result<(), Box<dyn Error>> {
    let mut buffer = [0; 4096];
    let mut authenticated = false;

    loop {
        let n = socket.read(&mut buffer).await?;
        if n == 0 {
            break; // Connection closed
        }

        let packet = RconPacket::deserialize(&buffer[..n])?;

        match packet.r#type {
            3 if !authenticated => {
                // Auth packet
                if packet.body == password {
                    authenticated = true;
                    let response = RconPacket::new(packet.id, 2, "".to_string()); // Auth response
                    socket.write_all(&response.serialize()).await?;
                } else {
                    let response = RconPacket::new(-1, 2, "".to_string()); // Auth failed
                    socket.write_all(&response.serialize()).await?;
                    break;
                }
            }
            2 if authenticated => {
                // Command packet
                println!("Received command: {}", packet.body);
                let response = RconPacket::new(packet.id, 2, "".to_string()); // Command response
                socket.write_all(&response.serialize()).await?;
            }
            _ => {
                break; // Invalid packet type
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("0.0.0.0:27015").await?;
    let password = "password".to_string(); // Set your RCON password here

    println!("RCON server listening on 0.0.0.0:27015");

    loop {
        let (socket, _) = listener.accept().await?;
        let password = password.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, password).await {
                eprintln!("Error handling client: {}", e);
            }
        });
    }
}

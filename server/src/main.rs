use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;

#[tokio::main]
async fn main() {
    let tcp = TcpListener::bind("127.0.0.1:8883").await.unwrap();
    println!("Successfully binded to 127.0.0.1:8883! Waiting for connections!");
    
    while let Ok((stream, addr)) = tcp.accept().await {
        println!("Client at {}:{} has connected!", addr.ip().to_string(), addr.port());
        tokio::spawn(async move {
            let s = tokio_tungstenite::accept_async(stream).await.unwrap();
            let (mut write,mut read) = s.split();

            tokio::spawn(async move {
                loop {
                    write.send(Message::Text("Hi there I am server!".into())).await.unwrap();
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            });

            while let Ok(msg) = read.next().await.unwrap() {
                println!("{}:{} -> {}", addr.ip().to_string(), addr.port(), msg.to_text().unwrap());
            }
        });
    }
}
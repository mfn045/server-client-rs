use futures_util::{future::join, SinkExt, StreamExt};
use tokio::{io::{self, AsyncBufReadExt, BufReader}, net::TcpListener};
use tokio_tungstenite::tungstenite::Message;
use tokio::sync::broadcast::channel;

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    
    let tcp = TcpListener::bind("127.0.0.1:8883").await.unwrap();

    println!("Successfully binded to 127.0.0.1:8883! Waiting for connections!");

    let (tx,mut _rx) = channel::<String>(16);

    let inp_handle = tokio::spawn({
        let tx_net = tx.clone();
        async move {
            let stdin = BufReader::new(io::stdin());
            let mut lines = stdin.lines();
            while let Ok(Some(msg)) = lines.next_line().await {
                tx_net.send(msg.to_string()).unwrap();
            }
        }
    });
    
    let net_handle = async move {
        while let Ok((stream, addr)) = tcp.accept().await {
            println!("Client at {}:{} has connected!", addr.ip().to_string(), addr.port());
            tokio::spawn({
                let tx_net = tx.clone();
                async move {
                    let s = tokio_tungstenite::accept_async(stream).await.unwrap();
                    let (mut write,mut read) = s.split();
                    write.send(Message::Text("Welcome to the server!".into())).await.unwrap();

                    let mut rx_peer = tx_net.subscribe();

                    // Receiving
                    let recv_handle = tokio::spawn(async move {
                        println!("Receiving task enabled!");
                        while let Ok(msg) = read.next().await.unwrap() {
                            tx_net.send(msg.to_string()).unwrap();
                            println!("{}:{} -> {}", addr.ip().to_string(), addr.port(), msg.to_text().unwrap());
                        }
                    });

                    // Sending
                    let send_handle = tokio::spawn(async move {
                        println!("Sending task enabled!");
                        while let Ok(msg) = rx_peer.recv().await {
                            write.send(Message::Text(msg.into())).await.unwrap();
                        }
                    });

                    let (_,_) = join(recv_handle, send_handle).await;
                }
            }).await.unwrap();
    }
    };

    let (_,_) = join(net_handle,inp_handle).await;
}
use std::{io::Error, time::{self, Duration}};
use futures_util::{future::join, SinkExt, StreamExt};
use tokio::{io::{self, AsyncBufReadExt, BufReader}, sync::mpsc::{self, Receiver, Sender}, task::JoinHandle};
use tokio_tungstenite::{connect_async, tungstenite::Message};



#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> Result<(), Error> {

    let (tx, rx) = mpsc::channel::<String>(1024);

    let net_handle = net_helper(rx).await;

    let inp_handle = inp_helper(tx).await;

    let (_,_) = join(net_handle, inp_handle).await;

    Ok(())
}

async fn net_helper(mut rx: Receiver<String>) -> JoinHandle<()> {
    let url: &'static str = "ws://127.0.0.1:8883";

    let t = time::UNIX_EPOCH;

    let connection =  connect_async(url).await;
    match connection {
        Ok((wss,resp)) => {
            let (mut write,mut read) = wss.split();
            println!("Split WebSocket stream to sink and source!");

            tokio::spawn({
                async move {
                    println!("Created network task!");
                    // Receiving
                    let recv_handle = tokio::spawn(async move {
                            println!("Enabled Receiving!");
                            while let Some(opt) = read.next().await {
                                if let Ok(msg) = opt {
                                    println!("Server: {}", msg.to_text().unwrap());
                                } else if let Err(err) = opt {
                                    println!("Found an error: {}", err);
                                }
                            }
                        });
                    
                    // Sending
                    let send_handle = tokio::spawn(async move {
                            println!("Enabled Sending!");
                            if let Some(msg) = rx.recv().await {
                                write.send(Message::Text(msg.into())).await.unwrap();
                            }
                        });

                    let (_,_) = join(recv_handle, send_handle).await;

                }
            })
        },
        Err(err) => { println!("[Error] Failed to connected to {}!", url); Ok(()) }
    }
}

async fn inp_helper(tx: Sender<String>) -> JoinHandle<()> {
    tokio::spawn({
        async move {
            println!("Created input task!");
            let reader = BufReader::new(io::stdin());
            let mut lines = reader.lines();
            
            while let Ok(Some(msg)) = lines.next_line().await {
                let send = tx.send(msg).await;
                if let Err(a) = send {
                    println!("Error sending message: {}", a.to_string());
                }
            }
        }
    })
}
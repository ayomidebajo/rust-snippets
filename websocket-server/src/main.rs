use futures::sink::SinkExt;
use futures::stream::StreamExt;

use std::fs::{self, File};
use std::io::prelude::*;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::{Error as WsError, Message};

type AnyError = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() -> Result<(), AnyError> {
    let addr = "127.0.0.1:9000";
    let listener = TcpListener::bind(addr).await?;

    println!("Listening on {:?}", addr);

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("{:?} connected.", addr);

        tokio::spawn(async move {
            let mut ws_stream = accept_async(stream).await?;
            println!("Handshake successful.");
            let mut new_file = File::create("record.wav").expect("error creating file");
            while let Some(item) = ws_stream.next().await {
                match item {
                    Ok(msg) => {
                        match msg {
                            Message::Binary(text) => {
                                // println!("Received text message: {:?}", text);
                                // new_file.write_all(text.clone());
                                new_file.write_all(text.as_slice());
                                
                                ws_stream.send(Message::Binary(text)).await?;

                                println!("Message sent back.");
                            }
                            Message::Close(frame) => {
                                println!("Received close message: {:?}", frame);

                                if let Err(e) = ws_stream.close(None).await {
                                    match e {
                                        WsError::ConnectionClosed => (),
                                        _ => {
                                            println!("Error while closing: {}", e);
                                            break;
                                        }
                                    }
                                }

                                println!("Sent close message.");

                                println!("Closing...");
                                break;
                            }
                            _ => (),
                        }
                    }
                    Err(e) => {
                        println!("Error receiving message: \n{0:?}\n{0}", e);
                    }
                }
            }

            Ok::<(), AnyError>(())
        });
    }
}

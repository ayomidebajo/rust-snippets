use futures::sink::SinkExt;
use futures::stream::StreamExt;

use tokio::net::TcpStream;
use tokio::time::{sleep, Duration};

use tokio_tungstenite::client_async;
use tokio_tungstenite::tungstenite::{Message, Error as WsError};

type AnyError = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() -> Result<(), AnyError> {
    let addr = "127.0.0.1:9000";
    let url = format!("ws://{}", addr);

    let stream = TcpStream::connect(addr).await?;

    println!("Connected to {:?}", addr);

    let (mut ws_stream, _) = client_async(&url, stream).await?;

    println!("Handshake successful.");

    for n in 0..10 {
        let text = format!("This is message {}.", n);
        let msg = Message::Text(text.clone());

        ws_stream.send(msg).await?;

        println!("Message sent: {:?}", text);

        if let Some(item) = ws_stream.next().await {
            match item {
                Ok(msg) => {
                    match msg {
                        Message::Text(text) => {
                            println!("Received text message: {}", text);
                        },
                        Message::Close(frame) => {
                            println!("Received close message: {:?}", frame);

                            if let Err(e) = ws_stream.close(None).await {
                                match e {
                                    WsError::ConnectionClosed => (),
                                    _ => {
                                        println!("Error while closing: {}", e);
                                        break;
                                    },
                                }
                            }

                            println!("Sent close message.");

                            println!("Closing...");
                            return Ok(());
                        },
                        _ => (),
                    }
                },
                Err(e) => {
                    println!("Error receiving message: \n{0:?}\n{0}", e);
                }
            }
        } else {
            break;
        }

        sleep(Duration::from_secs(1)).await;
    }

    ws_stream.close(None).await?;

    println!("Sent close message.");

    while let Some(item) = ws_stream.next().await {
        match item {
            Ok(msg) => {
                match msg {
                    Message::Close(frame) => {
                        println!("Received close message: {:?}", frame);

                        break;
                    },
                    _ => (),
                }
            },
            Err(e) => {
                println!("Error receiving message: \n{0:?}\n{0}", e);
            }
        }
    }

    println!("Closing...");
    Ok(())
}
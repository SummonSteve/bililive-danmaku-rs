mod bili;

use futures_channel::mpsc;
use futures_util::{StreamExt, SinkExt};
use serde_json::Value;
use std::time::Duration;
use tokio::time;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;
use tracing::{error, info};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let url = Url::parse("wss://broadcastlv.chat.bilibili.com/sub").unwrap();
    let (ws_stream, response) = connect_async(url).await.expect("Failed to connect");
    let (mut write, mut read) = ws_stream.split();

    info!("Status code: {}", response.status());
    for (ref header, _value) in response.headers() {
        info!("{}: {:?}", header, _value);
    }

    let handshake_packet = bili::encode("{\"roomid\": 21129652}", 7);
    let mut interval = time::interval(Duration::from_secs(30));

    write.send(Message::Binary(handshake_packet)).await.unwrap();
    
    let handle =tokio::spawn(async move {
        loop{
            tokio::select!{
                msg = read.next() => {
                    match msg {
                        Some(msg) => {
                            let data = msg.unwrap().into_data();
                            if data.len() > 0 {
                                handle_packet(data).await;
                            }
                        }
                        None => break,
                    }
                }
                _ = interval.tick() => {
                    write.send(Message::Binary(bili::encode("", 2))).await.unwrap();
                }
            }
        }
    });

    tokio::join!(handle);
}

async fn handle_packet(data: Vec<u8>) {
    let packet = bili::decode(data);
    info!("Recieved packet: {}", packet.op);
    match packet.op {
        8 => {
            info!("加入房间");
        }

        5 => {
            for body in packet.body {
                match serde_json::from_str(&body.unwrap()) {
                    Ok::<Value, _>(data) => {
                        match data["cmd"].as_str() {
                            Some(cmd) => match cmd {
                                
                                "DANMU_MSG" => {
                                    info!(
                                        "{}: {}",
                                        //data["info"][3][1],
                                        //data["info"][3][0],
                                        data["info"][2][1],
                                        data["info"][1]
                                    );
                                }
    
                                "SEND_GIFT" => {
                                    info!(
                                        "{}送了{}个{}",
                                        data["data"]["uname"],
                                        data["data"]["action"],
                                        data["data"]["giftName"]
                                    );
                                }
    
                                "WELCOME" => {
                                    info!("欢迎 {}", data["data"]["uname"]);
                                }
    
                                _ => {
                                    //info!("Unknown msg type MSG:{:?}", data["cmd"]);
                                    
                                }
                            },
                            None => {
                                return;
                            }
                        };
                    }
                    Err(e) => {
                        error!("json解析错误 {}", e);
                        return;
                    }
                };
            }
        }

        3 => {
            let data: Value = serde_json::from_str(&packet.body[0].as_ref().unwrap()).unwrap();
            info!("人气: {}", data["count"]);
        }

        _ => {}
    }
}


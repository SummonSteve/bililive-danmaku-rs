mod bili;

use futures_channel::mpsc;
use futures_util::{future, pin_mut, StreamExt};
use serde_json::Value;
use std::time::Duration;
use tokio::time;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;

#[tokio::main]
async fn main() {
    let url = Url::parse("wss://broadcastlv.chat.bilibili.com/sub").unwrap();
    let (tx, rx) = futures_channel::mpsc::unbounded();
    let (ws_stream, response) = connect_async(url).await.expect("Failed to connect");
    let (write, read) = ws_stream.split();

    println!("Status code: {}", response.status());
    for (ref header, _value) in response.headers() {
        println!("{}: {:?}", header, _value);
    }

    let handshake = bili::encode("{\"roomid\": Replace with room_id here}", 7);
    tx.unbounded_send(Message::binary(handshake)).unwrap();

    tokio::spawn(heartbeat(tx));
    let to_ws = rx.map(Ok).forward(write);
    let ws_to = {
        read.for_each(|message| async {
            let data = message.unwrap().into_data();
            if data.len() > 0 {
                handle_packet(data).await;
            }
        })
    };

    pin_mut!(to_ws, ws_to);
    future::select(to_ws, ws_to).await;
}

async fn handle_packet(data: Vec<u8>) {
    let packet = bili::decode(data);
    println!("Recieved packet: {}", packet.op);
    match packet.op {
        8 => {
            println!("加入房间");
        }

        5 => {
            match serde_json::from_str(&packet.body.unwrap()) {
                Ok::<Value, _>(data) => {
                    match data["cmd"].as_str() {
                        Some(cmd) => match cmd {
                            "DANMU_MSG" => {
                                println!("DANMU_MSG");
                                println!(
                                    "{}: {}",
                                    //data["info"][3][1],
                                    //data["info"][3][0],
                                    data["info"][2][1],
                                    data["info"][1]
                                );
                            }

                            "SEND_GIFT" => {
                                println!(
                                    "{}送了{}个{}",
                                    data["data"]["uname"],
                                    data["data"]["action"],
                                    data["data"]["giftName"]
                                );
                            }

                            "WELCOME" => {
                                println!("欢迎 {}", data["data"]["uname"]);
                            }

                            _ => {
                                println!("Unknown msg type MSG:{:?}", data["cmd"]);
                                
                            }
                        },
                        None => {
                            println!("parse error");
                            return;
                        }
                    };
                }
                Err(e) => {
                    println!("json解析错误 {}", e);
                    return;
                }
            };
        }

        3 => {
            let data: Value = serde_json::from_str(&packet.body.unwrap()).unwrap();
            println!("人气: {}", data["count"]);
        }

        _ => {}
    }
}

async fn heartbeat(tx: mpsc::UnboundedSender<Message>) {
    let heartbeat_packet = bili::encode("", 2);
    let mut freq = time::interval(Duration::from_secs(30));
    let mut started = false;
    loop {
        freq.tick().await;
        if started {
            tx.unbounded_send(Message::binary(heartbeat_packet.clone()))
                .unwrap();
            println!("sent heartbeat");
        }
        started = true;
    }
}

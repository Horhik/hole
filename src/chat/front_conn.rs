use crate::chat::types::PackedMessage;
use crate::db;
use async_std::{
    io,
    net::{TcpListener, TcpStream},
    task,
};
use async_tungstenite::{accept_async, tungstenite::Message, WebSocketStream};
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use serde_derive::Deserialize;
use std::env;
use std::sync::mpsc::{Receiver, Sender};

use super::stay_awake::request_repeater;

type SP = Sender<PackedMessage>;
type RP = Receiver<PackedMessage>;

#[derive(Deserialize, Debug)]
struct FrontMsg {
    user_id: u32,
    receiver_id: u32,
    message: String,
    time: String,
}

pub fn listen_client(server_sender: SP, client_receiver: RP) -> io::Result<()> {
    task::block_on(connect_to_client(server_sender, client_receiver))
}


async fn connect_to_client(server_sender: SP, client_receiver: RP) -> io::Result<()> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:5948".to_string());

    let listener = TcpListener::bind(&addr).await?;

    let client_repeater = server_sender.clone();
    if let Ok((stream, _)) = listener.accept().await {
        let addr = stream
            .peer_addr()
            .expect("connected streams should have a peer address");
        println!("Peer address: {}", addr);

        let ws = accept_async(stream)
            .await
            .expect("err during the ws handshake");
        let (sender, receiver) = ws.split();
        println!("connected to: {}", addr);

        let t1 = task::spawn(connection_for_sending(receiver, server_sender));
        connection_for_receiving(sender, client_receiver, client_repeater).await?;
        t1.await?;
    }

    Ok(())
}

async fn connection_for_receiving(
    mut sender: SplitSink<WebSocketStream<TcpStream>, Message>,
    client_receiver: RP,
    server_sender: SP,
) -> io::Result<()> {
    while let Ok(res) = client_receiver.recv() {
        //TODO call client get after receiving NodeHello
        if res.message.lines().next() == Some("NodeHello") {
            let server_sender = server_sender.clone();
            task::spawn(request_repeater(server_sender)).await?;
            println!("HEY NODE HELLO \n {}", res.message);
        }

        sender
            .send(Message::Text(String::from(res.message).to_owned()))
            .await
            .expect("ooops");
    }
    Ok(())
}

async fn connection_for_sending(
    mut receiver: SplitStream<WebSocketStream<TcpStream>>,
    server_sender: SP,
) -> io::Result<()> {
    let mut new_msg = receiver.next();
    loop {
        match new_msg.await {
            Some(msg) => {
                let jsoned = msg.unwrap();
                let res: serde_json::Result<FrontMsg> =
                    serde_json::from_str(jsoned.to_text().unwrap());
                if let Ok(received_msg) = res {
                    let msg = received_msg.message;
                    db::start_db().unwrap();

                    server_sender.send(PackedMessage { message: msg }).unwrap();
                /* message example
                {
                "user_id": 123456789,
                "receiver_id": 123456789,
                "message": "hey dude",
                "time": "Tue Oct 13 2020 18:31:22 GMT+0300 (Eastern European Summer Time)"
                }
                     */
                } else {
                    println!("seems, that messsage formatted wrong");
                    println!("{:?}", res);
                }

                new_msg = receiver.next();
            }
            None => {
                break;
            }
        }
    }
    Ok(())
}

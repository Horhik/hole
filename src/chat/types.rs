//type Decoded = String;
//type Encoded = String;

// trait Handler<State> {
//     fn process(code: State) -> Message;
//     fn send(socket: &WebSocketStream<TcpStream>, msg: Message);
//}
// pub struct MessageServer {
//     new_message: bool,
//     text: String,
// }

// pub struct MessageClient {
//     message_queue: Vec<Message>,
// }

// impl MessageServer {
//     fn new() -> MessageServer {
//         MessageServer {
//             new_message: false,
//             text: String::from(""),
//         }
//     }
// }

// impl MessageClient {
//     fn new() -> MessageClient {
//         MessageClient {
//             message_queue: vec![],
//         }
//     }
//}

pub struct PackedMessage {
    pub message: String,
}

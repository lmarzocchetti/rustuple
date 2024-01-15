use tungstenite::{connect, Message};
use url::Url;
use rustuple::*;

fn main() {
    // let (mut socket, response) =
    //     connect(Url::parse("ws://localhost:9001/socket").unwrap()).expect("Can't connect");

    // println!("Connected to the server");
    // println!("Response HTTP code: {}", response.status());
    // println!("Response contains the following headers:");
    // for (ref header, _value) in response.headers() {
    //     println!("* {}", header);
    // }

    // socket
    //     .send(Message::Text("Hello WebSocket".into()))
    //     .unwrap();

    // loop {
    //     let msg = socket.read().expect("Error reading message");
    //     println!("Received: {}", msg);
    // }

    let x1 = Field::Type(rustuple::Type::Integer);
    let x2 = Field::Value(rustuple::Value::Integer(1));

    let a = tuple! {
        x1,
        x2
    };

    println!("{:?}", a);
}

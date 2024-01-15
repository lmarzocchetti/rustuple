use rustuple::data::*;
use rustuple::tuple;
use rustuple::tuple_space::*;
use tungstenite::{connect, Message};
use url::Url;

fn main() -> Result<(), TupleError> {
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

    let mut tuple_space = TupleSpace::new("ws://localhost:9001/socket");

    let x1 = Field::Type(Type::Integer);
    let x2 = Field::Value(Value::Integer(1));

    let a = tuple! {
        x1,
        x2
    };

    tuple_space.inb(a)?;

    Ok(())

}

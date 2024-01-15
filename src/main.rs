use core::time;
use std::{net::TcpListener, thread::sleep};
use std::thread::spawn;
use tungstenite::{
    accept_hdr,
    handshake::server::{ErrorResponse, Request, Response},
};

fn callback(req: &Request, mut response: Response) -> Result<Response, ErrorResponse> {
    println!("Received a new ws handshake");
    println!("The request's path is: {}", req.uri().path());
    println!("The request's headers are:");
    for (ref header, _value) in req.headers() {
        println!("* {}", header);
    }

    let headers = response.headers_mut();
    headers.append("MyCustomHeader", ":)".parse().unwrap());
    headers.append("SOME_TUNGSTENITE_HEADER", "header_value".parse().unwrap());

    Ok(response)
}

fn main() {
    let server = TcpListener::bind("127.0.0.1:9001").unwrap();

    for stream in server.incoming() {
        spawn(move || {
            let mut websocket = accept_hdr(stream.unwrap(), callback).unwrap();

            sleep(time::Duration::from_secs(3));

            loop {
                let msg = websocket.read().unwrap();

                if msg.is_binary() || msg.is_text() {
                    websocket.send(msg).unwrap();
                }
            }
        });
    }
}

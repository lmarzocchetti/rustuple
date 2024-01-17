use clap::Parser;
use rustuple::data::{Operation, Tuple, TupleError};
use std::fmt::Display;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread::spawn;
use std::vec;
use std::{net::TcpListener, thread::sleep};
use tungstenite::{
    accept_hdr,
    handshake::server::{ErrorResponse, Request, Response},
};
use tungstenite::{Message, WebSocket};

#[derive(Parser)]
struct Cli {
    /// Ip Address
    ip_addr: String,

    /// Port number
    port_num: String,
}

#[derive(Clone)]
struct TupleSpace {
    tuples: Arc<Mutex<Vec<Tuple>>>,
}

impl TupleSpace {
    pub fn new() -> Self {
        TupleSpace {
            tuples: Arc::new(Mutex::new(vec![])),
        }
    }

    pub fn clone(&self) -> Self {
        TupleSpace {
            tuples: Arc::clone(&self.tuples),
        }
    }

    pub fn out(&mut self, tuple: Tuple) -> Result<(), TupleError> {
        let mut space = self.tuples.lock().unwrap();

        if !space.iter().any(|elem| elem == &tuple) {
            space.push(tuple);
            return Ok(());
        }

        Err(TupleError::TupleAlreadyPresentError)
    }

    pub fn in_non_bl(&mut self, tuple: &Tuple) -> Result<Vec<Tuple>, TupleError> {
        let mut space = self.tuples.lock().unwrap();

        let ret = space
            .iter()
            .filter(|&elem| elem.len() == tuple.len())
            .filter(|&elem| elem.matching_tuples(tuple.clone()))
            .cloned()
            .collect::<Vec<Tuple>>();

        if ret.is_empty() {
            return Err(TupleError::NoMatchingTupleError);
        } else {
            return Ok(ret);
        }
    }
}

impl Display for TupleSpace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[\n").unwrap();

        for i in (*self.tuples.lock().unwrap()).iter() {
            write!(f, "{}\n", i).unwrap()
        }

        write!(f, "]").unwrap();

        Ok(())
    }
}

fn deserialize(message: String) -> Result<Operation, TupleError> {
    match serde_json::from_str(&message) {
        Ok(res) => Ok(res),
        Err(e) => {
            println!("Error serializing! Error: {}", e);
            Err(TupleError::Error)
        }
    }
}

fn serialize_vector(vector: Vec<Tuple>) -> Result<String, TupleError> {
    match serde_json::to_string(&vector) {
        Ok(res) => Ok(res),
        Err(e) => {
            println!("Error serializing! Error: {}", e);
            Err(TupleError::Error)
        }
    }
}

fn handle_out(space: &mut TupleSpace, tuple: Tuple) -> Result<(), TupleError> {
    if !tuple.has_data_only() {
        return Err(TupleError::TupleNotOnlyDataError);
    }

    space.out(tuple)
}

fn handle_in_non_bl(
    space: &mut TupleSpace,
    socket: &mut WebSocket<TcpStream>,
    tuple: Tuple,
) -> Result<(), TupleError> {
    if tuple.has_data_only() {
        return Err(TupleError::TupleOnlyDataError);
    }

    let ret = space.in_non_bl(&tuple)?;
    let serialized = serialize_vector(ret)?;

    match socket.write(Message::Text(serialized)) {
        Ok(_) => Ok(()),
        Err(_) => return Err(TupleError::Error),
    }
}

fn incoming_operations(
    space: &mut TupleSpace,
    socket: &mut WebSocket<TcpStream>,
    message: String,
) -> Result<(), TupleError> {
    let operation = deserialize(message)?;

    match operation {
        Operation::Out(val) => return handle_out(space, val),
        Operation::InBl(val) => panic!("InBl not implemented"),
        Operation::RdBl(val) => panic!("RdBl not implemented"),
        Operation::InNonBl(val) => return handle_in_non_bl(space, socket, val),
        Operation::RdNonBl(val) => panic!("RdNonBl not implemented"),
    }
}

fn callback(req: &Request, response: Response) -> Result<Response, ErrorResponse> {
    println!("Received a new ws handshake");
    println!("The request's path is: {}", req.uri().path());
    println!("The request's headers are:");
    for (ref header, _value) in req.headers() {
        println!("* {}", header);
    }

    Ok(response)
}

fn main() {
    let args = Cli::parse();

    let server = TcpListener::bind(format!("{}:{}", args.ip_addr, args.port_num)).unwrap();

    let space = TupleSpace::new();

    for stream in server.incoming() {
        let mut cloned = space.clone();
        spawn(move || {
            let mut websocket = accept_hdr(stream.unwrap(), callback).unwrap();

            loop {
                let msg = websocket.read();

                let res = match msg {
                    Ok(mex) => match mex {
                        Message::Text(val) => incoming_operations(&mut cloned, &mut websocket, val),
                        Message::Close(_) => {
                            break;
                        }
                        _ => panic!("Operation not permitted!"),
                    },
                    Err(err) => {
                        break;
                    }
                };

                match res {
                    Ok(_) => {
                        let _ = websocket.send(Message::Text(
                            serde_json::to_string(&TupleError::NoError).unwrap(),
                        ));
                    }
                    Err(error) => {
                        let _ =
                            websocket.send(Message::Text(serde_json::to_string(&error).unwrap()));
                    }
                }

                println!("Tuple Space: {}", cloned);
            }
        });
    }
}

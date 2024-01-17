use clap::Parser;
use rustuple::data::{Operation, Tuple, TupleError};
use std::fmt::Display;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::thread::spawn;
use std::time::Duration;
use std::vec;
use tungstenite::{
    accept_hdr,
    handshake::server::{ErrorResponse, Request, Response},
};
use tungstenite::{Message, WebSocket};

/// Parser for command line arguments
#[derive(Parser)]
struct Cli {
    /// Ip Address
    ip_addr: String,

    /// Port number
    port_num: String,
}

/// Struct to create a new Tuple data space, which is mutually accessed by threads
#[derive(Clone)]
struct TupleSpace {
    tuples: Arc<Mutex<Vec<Tuple>>>,
}

impl TupleSpace {
    /// Construct a new Tuple Space
    pub fn new() -> Self {
        TupleSpace {
            tuples: Arc::new(Mutex::new(vec![])),
        }
    }

    /// Needed for mutual exclusion to increment the strong reference counting of the Arc
    pub fn clone(&self) -> Self {
        TupleSpace {
            tuples: Arc::clone(&self.tuples),
        }
    }

    /// Insert a new Tuple in the Tuple Space and return Ok(()) if Tuple Space not contain the specific Tuple, otherwise an Error
    pub fn out(&mut self, tuple: Tuple) -> Result<(), TupleError> {
        let mut space = self.tuples.lock().unwrap();

        if !space.iter().any(|elem| elem == &tuple) {
            space.push(tuple);
            return Ok(());
        }

        Err(TupleError::TupleAlreadyPresentError)
    }

    /// Extract some tuples out of the Tuple Space, returning Ok(Vec<Tuple>) if at least one is matching, otherwise return an Error
    pub fn _in(&mut self, tuple: &Tuple) -> Result<Vec<Tuple>, TupleError> {
        let mut space = self.tuples.lock().unwrap();
        let mut to_remove: Vec<usize> = vec![];

        let ret = space
            .iter()
            .filter(|&elem| elem.len() == tuple.len())
            .filter(|&elem| elem.matching_tuples(tuple.clone()))
            .cloned()
            .collect::<Vec<Tuple>>();

        for i in ret.clone().iter() {
            let finded = space.iter_mut().enumerate().find(|elem| elem.1 == i);
            match finded {
                Some(val) => to_remove.push(val.0),
                None => continue,
            }
        }

        for i in to_remove.iter() {
            space.remove(*i);
        }

        if ret.is_empty() {
            return Err(TupleError::NoMatchingTupleError);
        } else {
            return Ok(ret);
        }
    }

    /// Read some tuples of the Tuple Space, returning Ok(Vec<Tuple>) if at least one is matching, otherwise return an Error
    pub fn _rd(&mut self, tuple: &Tuple) -> Result<Vec<Tuple>, TupleError> {
        let space = self.tuples.lock().unwrap();

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

fn handle_in_bl(
    space: &mut TupleSpace,
    socket: &mut WebSocket<TcpStream>,
    tuple: Tuple,
) -> Result<(), TupleError> {
    if tuple.has_data_only() {
        return Err(TupleError::TupleOnlyDataError);
    }

    let mut ret = space._in(&tuple);
    while ret.is_err() {
        sleep(Duration::from_secs(1));
        ret = space._in(&tuple);
    }

    let serialized = serialize_vector(ret.unwrap())?;

    match socket.write(Message::Text(serialized)) {
        Ok(_) => Ok(()),
        Err(_) => return Err(TupleError::Error),
    }
}

fn handle_rd_bl(
    space: &mut TupleSpace,
    socket: &mut WebSocket<TcpStream>,
    tuple: Tuple,
) -> Result<(), TupleError> {
    if tuple.has_data_only() {
        return Err(TupleError::TupleOnlyDataError);
    }

    let mut ret = space._rd(&tuple);
    while ret.is_err() {
        sleep(Duration::from_secs(1));
        ret = space._rd(&tuple);
    }

    let serialized = serialize_vector(ret.unwrap())?;

    match socket.write(Message::Text(serialized)) {
        Ok(_) => Ok(()),
        Err(_) => return Err(TupleError::Error),
    }
}

fn handle_in_non_bl(
    space: &mut TupleSpace,
    socket: &mut WebSocket<TcpStream>,
    tuple: Tuple,
) -> Result<(), TupleError> {
    if tuple.has_data_only() {
        return Err(TupleError::TupleOnlyDataError);
    }

    let ret = space._in(&tuple)?;
    let serialized = serialize_vector(ret)?;

    match socket.write(Message::Text(serialized)) {
        Ok(_) => Ok(()),
        Err(_) => return Err(TupleError::Error),
    }
}

fn handle_rd_non_bl(
    space: &mut TupleSpace,
    socket: &mut WebSocket<TcpStream>,
    tuple: Tuple,
) -> Result<(), TupleError> {
    if tuple.has_data_only() {
        return Err(TupleError::TupleOnlyDataError);
    }

    let ret = space._rd(&tuple)?;
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
        Operation::InBl(val) => return handle_in_bl(space, socket, val),
        Operation::RdBl(val) => return handle_rd_bl(space, socket, val),
        Operation::InNonBl(val) => return handle_in_non_bl(space, socket, val),
        Operation::RdNonBl(val) => return handle_rd_non_bl(space, socket, val),
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
                    Err(_) => {
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

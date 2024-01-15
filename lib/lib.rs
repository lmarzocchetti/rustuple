pub mod data {
    use serde::{Deserialize, Serialize};

    /// Type allowed in the Field type to do pattern matching
    #[derive(Serialize, Deserialize, Debug)]
    pub enum Type {
        Integer,
        Float,
        String,
    }

    /// Values allowed in the Field type
    #[derive(Serialize, Deserialize, Debug)]
    pub enum Value {
        Integer(i32),
        Float(f32),
        String(String),
    }

    /// Type that represent a single field in Tuple
    #[derive(Serialize, Deserialize, Debug)]
    pub enum Field {
        /// Concrete value
        Value(Value),

        /// Type used for make a matching in the Tuple Space
        Type(Type),
    }

    /// Basic type for the Tuple Space
    #[derive(Serialize, Deserialize, Debug)]
    pub struct Tuple {
        tuples: Vec<Field>,
    }

    impl Tuple {
        pub fn new() -> Self {
            Tuple { tuples: vec![] }
        }

        pub fn add(&mut self, field: Field) {
            self.tuples.push(field)
        }
    }

    /// An Enumeration to represent all Operation permitted on the Tuple Space
    #[derive(Serialize, Deserialize, Debug)]
    pub enum Operation {
        /// Puts a tuple in the Tuple Space
        Out(Tuple),

        /// Takes out a tuple that matches a given pattern from the Tuple Space (Blocking)
        In(Tuple),

        /// Copies a tuple that matches a given pattern from the Tuple Space (Blocking)
        Rd(Tuple),

        /// Takes out a tuple that matches a given pattern from the Tuple Space (Non Blocking)
        Inp(Tuple),

        /// Copies a tuple that matches a given pattern from the Tuple Space (Non Blocking)
        Rdp(Tuple),
    }
}

/// Macro to create a new Tuple with a variable number of arguments
#[macro_export]
macro_rules! tuple {
    ($($x:expr),*) => {
        {
            use rustuple::data::Tuple;
            let mut temp_tuple = Tuple::new();
            $(
                temp_tuple.add($x);
            )*
            temp_tuple
        }
    };
}

pub mod tuple_space {
    use std::net::TcpStream;

    use tungstenite::{connect, Message, WebSocket, stream::MaybeTlsStream};
    use url::Url;

    use crate::data::{Tuple, Operation};

    #[derive(Debug, Clone, Copy)]
    pub enum TupleError {
        ExampleError
    }

    pub struct TupleSpace {
        socket: WebSocket<MaybeTlsStream<TcpStream>>
    }

    impl TupleSpace {
        pub fn new(ip_addr: &str) -> Self {
            let (socket, response) = 
                connect(Url::parse(ip_addr).unwrap()).expect("Can't connect");
            
            println!("Connected to the server");
            println!("Response HTTP code: {}", response.status());

            TupleSpace{socket: socket}
        }

        fn serialize(operation: Operation) -> Result<String, TupleError> {
            match serde_json::to_string(&operation) {
                Ok(res) => Ok(res),
                Err(e) => {
                    println!("Error serializing! Error: {}", e);
                    Err(TupleError::ExampleError)
                }
            }
        }

        pub fn outb(&mut self, tuple: Tuple) -> Result<(), TupleError> {
            let serialized = TupleSpace::serialize(Operation::Out(tuple))?;

            let res = self.socket
                .send(Message::Text(serialized));

            match res {
                Ok(_) => Ok(()),
                Err(_) => Err(TupleError::ExampleError)
            }
        }

        pub fn inb(&mut self, tuple: Tuple) -> Result<(), TupleError> {
            let serialized = TupleSpace::serialize(Operation::In(tuple))?;

            let res = self.socket
                .send(Message::Text(serialized));

            match res {
                Ok(_) => Ok(()),
                Err(_) => Err(TupleError::ExampleError)
            }
        }
    }
}
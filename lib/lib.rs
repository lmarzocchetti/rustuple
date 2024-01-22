/// Modules that contain Data structures used by the library
pub mod data {
    use std::fmt::Display;

    use serde::{Deserialize, Serialize};

    /// Error used by the library and returned by the server
    #[derive(Serialize, Deserialize, Debug, Clone, Copy)]
    pub enum TupleError {
        TupleAlreadyPresentError,
        TupleNotOnlyDataError,
        TupleOnlyDataError,
        NoMatchingTupleError,
        Error,
        NoError
    }

    /// Type allowed in the Field type to do pattern matching
    #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
    pub enum Type {
        Integer,
        String,
    }

    impl Display for Type {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Type::Integer => write!(f, "Integer"),
                Type::String => write!(f, "String")
            }
        }
    }

    /// Values allowed in the Field type
    #[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
    pub enum Value {
        Integer(i32),
        String(String),
    }

    impl Display for Value {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Value::Integer(val) => write!(f, "{}", val),
                Value::String(val) => write!(f, "{}", val)
            }
        }
    }

    /// Type that represent a single field in Tuple
    #[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
    pub enum Field {
        /// Concrete value
        Value(Value),

        /// Type used for make a matching in the Tuple Space
        Type(Type),
    }

    impl Display for Field {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Field::Value(val) => write!(f, "{}", val),
                Field::Type(val) => write!(f, "{}", val)
            }
        }
    }

    /// Basic type for the Tuple Space
    #[derive(Serialize, Deserialize, Clone, Debug)]
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

        /// Helper method to check if the tuple contain only data and it is not a tuple for pattern matching
        pub fn has_data_only(&self) -> bool {
            for i in self.tuples.iter() {
                match i {
                    Field::Type(_) => return false,
                    _ => continue
                }
            }

            true
        }

        pub fn equal(&self, other: &Tuple) -> bool {
            let mut is_equal = true;

            for (this, other) in self.iter().zip(other.iter()) {
                if this != other {
                    is_equal = false;
                    break;
                }
            }

            is_equal
        }

        pub fn len(&self) -> usize {
            self.tuples.len()
        }

        /// Iterate over elements of the Tuple
        pub fn iter(&self) -> std::slice::Iter<'_, Field> {
            self.tuples.iter()
        }

        /// Return true if two tuples match, otherwise false
        pub fn matching_tuples(&self, other: Tuple) -> bool {
            for (i, j) in self.tuples.iter().zip(other.tuples.iter()) {
                match j {
                    Field::Value(_) => {
                        if i == j {
                            continue;
                        }
                        else {
                            return false;
                        }
                    },
                    Field::Type(tp) => {
                        match tp {
                            Type::Integer => {
                                match i {
                                    Field::Value(val) => {
                                        match val {
                                            Value::Integer(_) => continue,
                                            _ => return false
                                        }
                                    },
                                    _ => panic!("Saved a tuple with a type Field!")
                                }
                            },
                            Type::String => {
                                match i {
                                    Field::Value(val) => {
                                        match val {
                                            Value::String(_) => continue,
                                            _ => return false
                                        }
                                    },
                                    _ => panic!("Saved a tuple with a type Field!")
                                }
                            }
                        }
                    }
                }
            }
            true
        }
    }

    impl Display for Tuple {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "(").unwrap();

            for (idx, i) in self.tuples.iter().enumerate() {
                if idx != self.tuples.len() - 1 {
                    match write!(f, "{}, ", i) {
                        Ok(_) => (),
                        Err(_) => ()
                    }
                }
                else {
                    match write!(f, "{})", i) {
                        Ok(_) => (),
                        Err(_) => ()
                    }
                }
            }

            Ok(())
        }
    }

    /// An Enumeration to represent all Operation permitted on the Tuple Space
    #[derive(Serialize, Deserialize, Debug)]
    pub enum Operation {
        /// Puts a tuple in the Tuple Space
        Out(Tuple),

        /// Takes out a tuple that matches a given pattern from the Tuple Space (Blocking)
        InBl(Tuple),

        /// Copies a tuple that matches a given pattern from the Tuple Space (Blocking)
        RdBl(Tuple),

        /// Takes out a tuple that matches a given pattern from the Tuple Space (Non Blocking)
        InNonBl(Tuple),

        /// Copies a tuple that matches a given pattern from the Tuple Space (Non Blocking)
        RdNonBl(Tuple),
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

/// Module that contain the implementation of the Tuple Space operations using the data structures of the "data" module
pub mod tuple_space {
    use std::net::TcpStream;

    use tungstenite::{connect, Message, WebSocket, stream::MaybeTlsStream};
    use url::Url;

    use crate::data::{Tuple, Operation, TupleError};

    /// Struct to handle the connection and the operation between the client and the server
    pub struct TupleSpace {
        socket: WebSocket<MaybeTlsStream<TcpStream>>
    }

    impl TupleSpace {
        /// Construct a new Tuple Space and open the connection on the ip address and port passed in
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
                    Err(TupleError::Error)
                }
            }
        }

        fn deserialize_error(msg: Message) -> TupleError {
            match msg {
                Message::Text(val) => return serde_json::from_str(&val).unwrap(),
                _ => panic!("Errore: Messaggio ricevuto non e' in forma testuale!")
            }
        }

        fn deserialize_vector(msg: Message) -> Result<Vec<Tuple>, serde_json::Error> {
            match msg {
                Message::Text(val) => return serde_json::from_str(&val),
                _ => panic!("Errore: Messaggio ricevuto non e' in forma testuale!")
            }
        }

        /// Close the connection
        pub fn close(&mut self) {
            self.socket.close(Option::None).unwrap();
            self.socket.flush().unwrap()
        }

        /// Implementation of the Out operation for the client, returing an Error
        pub fn out(&mut self, tuple: Tuple) -> Result<(), TupleError> {
            let serialized = TupleSpace::serialize(Operation::Out(tuple))?;

            let send = self.socket
                .send(Message::Text(serialized));

            match send {
                Ok(_) => (),
                Err(_) => return Err(TupleError::Error)
            }

            let res = self.socket.read().unwrap();
            let res_deser = TupleSpace::deserialize_error(res);
            match res_deser {
                TupleError::NoError => return Ok(()),
                TupleError::TupleNotOnlyDataError => return Err(TupleError::TupleNotOnlyDataError),
                TupleError::TupleAlreadyPresentError => return Err(TupleError::TupleAlreadyPresentError),
                _ => return Err(TupleError::Error)
            }
        }

        /// Implementation of the operations which are the same for the in and rd operation (blocking and non-blocking)
        fn in_rd(&mut self, operation: String) -> Result<Vec<Tuple>, TupleError> {
            let res = self.socket
                .send(Message::Text(operation));

            match res {
                Ok(_) => (),
                Err(_) => return Err(TupleError::Error)
            }

            let res = self.socket.read().unwrap();
            let vector = match TupleSpace::deserialize_vector(res.clone()) {
                Ok(vec) => vec,
                Err(_) => {
                    return Err(TupleSpace::deserialize_error(res));
                }
            };

            let no_error = self.socket.read().unwrap();
            let no = TupleSpace::deserialize_error(no_error);

            match no {
                TupleError::NoError => Ok(vector),
                _ => Err(no)
            }
        }

        /// In blocking operation, return when the requested matching tuples are finded and erased in the server
        pub fn in_bl(&mut self, tuple: Tuple) -> Result<Vec<Tuple>, TupleError> {
            let serialized = TupleSpace::serialize(Operation::InBl(tuple))?;

            self.in_rd(serialized)
        }

        /// Rd blocking operation, return when the requested matching tuples are finded in the server
        pub fn rd_bl(&mut self, tuple: Tuple) -> Result<Vec<Tuple>, TupleError> {
            let serialized = TupleSpace::serialize(Operation::RdBl(tuple))?;

            self.in_rd(serialized)
        }

        /// Rd non-blocking operation, return NoMatchingTupleError in case of no matching tuples
        pub fn rd_non_bl(&mut self, tuple: Tuple) -> Result<Vec<Tuple>, TupleError> {
            let serialized = TupleSpace::serialize(Operation::RdNonBl(tuple))?;

            self.in_rd(serialized)
        }

        /// In non-blocking operation, return NoMatchingTupleError in case of no matching tuples
        pub fn in_non_bl(&mut self, tuple: Tuple) -> Result<Vec<Tuple>, TupleError> {
            let serialized = TupleSpace::serialize(Operation::InNonBl(tuple))?;

            self.in_rd(serialized)
        }
    }
}

use std::vec;

#[derive(Debug)]
pub enum Type {
    Integer,
    Float,
    String
}

#[derive(Debug)]
pub enum Value {
    Integer(i32),
    Float(f32),
    String(String)
}

#[derive(Debug)]
pub enum Field {
    Value(Value),
    Type(Type)
}

#[derive(Debug)]
pub struct Tuple {
    tuples: Vec<Field>
}

impl Tuple {
    pub fn new() -> Self {
        Tuple{tuples: vec![]}
    }

    pub fn add(&mut self, field: Field) {
        self.tuples.push(field)
    }
}

#[macro_export]
macro_rules! tuple {
    ($($x:expr),*) => {
        {
            use rustuple::Tuple;
            let mut temp_tuple = Tuple::new();
            $(
                temp_tuple.add($x);
            )*
            temp_tuple
        }
    };
}

/// An Enumeration to represent all Operation permitted on the Tuple Space
pub enum Operation {
    /// Puts a tuple in the Tuple Space
    Out(String),

    /// Takes out a tuple that matches a given pattern from the Tuple Space (Blocking)
    In(String),

    /// Copies a tuple that matches a given pattern from the Tuple Space (Blocking)
    Rd(String),

    /// Takes out a tuple that matches a given pattern from the Tuple Space (Non Blocking)
    Inp(String),

    /// Copies a tuple that matches a given pattern from the Tuple Space (Non Blocking)
    Rdp(String)
}
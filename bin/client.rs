use rustuple::data::*;
use rustuple::tuple;
use rustuple::tuple_space::*;

fn main() -> Result<(), TupleError> {
    let mut tuple_space = TupleSpace::new("ws://localhost:9001/socket");

    let x1 = Field::Value(Value::String("Mannaggia".to_string()));
    let x2 = Field::Value(Value::Integer(2));

    let a = tuple! {
        x1,
        x2
    };

    tuple_space.out(a)?;

    Ok(())
}

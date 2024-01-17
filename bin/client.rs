use rustuple::data::*;
use rustuple::tuple;
use rustuple::tuple_space::*;

fn main() -> Result<(), TupleError> {
    let mut tuple_space = TupleSpace::new("ws://localhost:9001/socket");

    let x1 = Field::Value(Value::String("Mannaggia".to_string()));
    let x2 = Field::Value(Value::Integer(3));

    let a = tuple! {
        x1,
        x2
    };

    tuple_space.out(a)?;

    let x1 = Field::Type(Type::String);
    let x2 = Field::Value(Value::Integer(2));
    let find = tuple!(
        x1,
        x2
    );

    let res = tuple_space.in_non_bl(find)?;

    for i in res.iter() {
        println!("{}", i);
    }

    tuple_space.close();

    Ok(())
}

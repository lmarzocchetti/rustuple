# rustuple
A Rust tuple space implementation using Web Socket!

### Project division
This library is composed by a Server and a Library to write components that can communicate with each other in the Tuple Space.

### Operations implemented (Linda)
- Out: Put a Tuple in the Tuple space
- In (non-blocking): Extract a tuple from the Tuple space
- In (blocking): Blocking version of In
- Rd (non-blocking): Read a tuple from the Tuple space
- Rd (blocking): Blocking version of Rd

### Compile and using
Compile the server:
```
$ cargo build --bin --release rustuple 
```
Run the server (in the target/release folder):
```
$ ./rustuple <IP_ADDR> <PORT_NUM>
```
I use in the example client IP_ADDR = "127.0.0.1" and PORT_NUM = "9001"

Compile the example client that used the library:
```
$ cargo build --bin --release client
```

To use the Library create a new [[bin]] in the Cargo.toml file and simply use:
```
use rustuple::data;
use rustuple::tuple_space;
```

In the data module there are all the data structures and in the tuple_space there is the struct to connect and access the Tuple Space. (see the bin folder for the example client)

### Documentation
You can access the documentation by run:
```
$ cargo doc --release --open 
```

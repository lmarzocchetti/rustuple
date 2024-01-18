use std::thread::sleep;
use std::thread::spawn;
use std::time::Duration;

use rustuple::data::*;
use rustuple::tuple;
use rustuple::tuple_space::*;

const NUM_NODES: i32 = 8;

struct Node {
    id: i32,
    right_neighbor: i32,
    current_leader: i32,
    tuple_space: TupleSpace
}

impl Node {
    fn new(id: i32, right_neighbor: i32, ip_addr: &str) -> Self {
        Node { id:id, right_neighbor: right_neighbor, current_leader: 0, tuple_space: TupleSpace::new(ip_addr) }
    }

    /// Message is composed by this tuple (id_receiver, id)
    fn send_leader_proposal(&mut self, prop_id: i32) -> Result<(), TupleError> {
        let tuple = tuple!(
            Field::Value(Value::Integer(self.right_neighbor)),
            Field::Value(Value::Integer(prop_id))
        );
        
        self.tuple_space.out(tuple)
    }

    fn receive_leader_proposal(&mut self) -> Result<Vec<Tuple>, TupleError> {
        let tuple = tuple!(
            Field::Value(Value::Integer(self.id)),
            Field::Type(Type::Integer)
        );

        self.tuple_space.in_non_bl(tuple)
    }

    fn send_halt_message(&mut self) -> Result<(), TupleError> {
        let tuple = tuple!(
            Field::Value(Value::String("HALT".to_string())),
            Field::Value(Value::Integer(self.id))
        );

        self.tuple_space.out(tuple)
    }

    fn control_halt_message(&mut self) -> Result<i32, TupleError> {
        let tuple = tuple!(
            Field::Value(Value::String("HALT".to_string())),
            Field::Type(Type::Integer)
        );

        match self.tuple_space.rd_non_bl(tuple) {
            Ok(val) => {
                match val[0].iter().last().unwrap() {
                    Field::Value(Value::Integer(id)) => {
                        return Ok(id.clone());
                    },
                    _ => panic!("Not possible")
                };
            },
            Err(_) => Err(TupleError::Error)
        }
    }

    fn run(&mut self) -> Result<(), TupleError> {
        self.send_leader_proposal(self.id)?;

        loop {
            // control HALT message
            let halt = self.control_halt_message();
            let _ = match halt {
                Ok(val) => {
                    println!("Id {}: Received HALT message from {}, it is the new Leader!", self.id, val);
                    self.current_leader = val;
                    break;
                },
                Err(_) => ()
            };

            let received_proposal: Vec<Tuple>;
            let received_proposal_or_err = self.receive_leader_proposal();
            match received_proposal_or_err {
                Ok(val) => received_proposal = val,
                Err(_) => continue
            }
            sleep(Duration::from_secs(1));

            match received_proposal[0].iter().last().unwrap() {
                Field::Value(Value::Integer(val)) => {
                    if val > &self.id {
                        println!("Id {}: Received proposal from {}. I'm going to FORWARD it!", self.id, val);
                        self.send_leader_proposal(*val)?;
                    }
                    else if val < &self.id {
                        println!("Id {}: Received proposal from {}. I'm going to DISCARD it!", self.id, val);
                        self.send_leader_proposal(self.id)?;
                    }
                    else {
                        println!("Id {}: Received proposal with my Id. I'm going to HALT message to everyone to declare myself as the new leader!", self.id);
                        self.current_leader = self.id;
                        self.send_halt_message()?;
                        break;
                    }
                },
                _ => panic!("Not possible")
            };
        }

        Ok(())
    }

    fn close(&mut self) {
        self.tuple_space.close()
    }
}

fn main() -> Result<(), TupleError> {
    let mut handles = vec![];

    for i in 1..NUM_NODES+1 {
        handles.push(spawn(move || {
            let mut right = i + 1;
            
            if i == NUM_NODES {
                right = 1;
            }

            let mut node = Node::new(i, right, "ws://localhost:9001/socket");
            let _ = node.run();
            node.close();
        }));
    }

    for handle in handles {
        let _ = handle.join();
    }

    Ok(())
}
use std::thread::spawn;
use std::vec;

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
        
        match self.tuple_space.out(tuple) {
            Ok(_) => Ok(()),
            Err(TupleError::TupleAlreadyPresentError) => Ok(()),
            Err(err) =>  Err(err)   
        }
    }

    fn receive_leader_proposal(&mut self) -> Result<Vec<Tuple>, TupleError> {
        let tuple = tuple!(
            Field::Value(Value::Integer(self.id)),
            Field::Type(Type::Integer)
        );

        // change to rd in case
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

    fn received_max(vector: &Vec<Tuple>) -> Vec<Tuple> {
        let a = vector.iter().enumerate().map(|(idx, a)| {
            match a.iter().last().unwrap() {
                Field::Value(Value::Integer(val)) => (idx, val.clone()),
                _ => (idx, 0),
            }
        })
        .max_by_key(|x| {
            x.1
        }).unwrap();

        let ret = &vector[a.0];

        vec![ret.clone()]
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

            let mut received_proposal: Vec<Tuple>;
            let received_proposal_or_err = self.receive_leader_proposal();
            match received_proposal_or_err {
                Ok(val) => received_proposal = val,
                Err(_) => continue
            }

            if received_proposal.len() != 1 && !received_proposal.is_empty() {
                received_proposal = Node::received_max(&received_proposal);
            }

            assert!(received_proposal.len() == 1);

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

            println!("Spawning node with ID: {} and RIGHT: {}", i, right);

            let mut node = Node::new(i, right, "ws://localhost:9001/socket");
            let err = node.run();
            match err {
                Err(err) => println!("Node with Id: {} return an Error {:?}", i, err),
                Ok(_) => ()

            }
            node.close();
        }));
    }

    for (idx, handle) in handles.into_iter().enumerate() {
        let err = handle.join();
        println!("Closed thread {}", idx);
        match err {
            Err(err) => println!("Error {:?}", err),
            Ok(_) => ()

        }
    }

    Ok(())
}
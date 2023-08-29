use std::sync::Arc;

use crate::env::{Cluster, Executor, ProcessId, ProcessType, Receiver, Sender};
use crate::env::{Env, Router};
use std::sync::atomic::AtomicU32;
// pub struct ZMQEnv<T, R, S>
// where
//     T: Router,
//     R: Receiver,
//     S: Sender,
// {
//     id_gen: Arc<AtomicU32>,
//     new_channel_fn: fn() -> (R, S),
//     router: T,
//     cluster: Arc<Cluster>,
// }

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    #[test]
    fn zmp_example() {
        let context = zmq::Context::new();
        let responder = context.socket(zmq::REP).unwrap();
        assert!(responder.bind("tcp://*:5555").is_ok());

        let requester = context.socket(zmq::REQ).unwrap();
        assert!(requester.connect("tcp://localhost:5555").is_ok());

        let client = thread::spawn(move || {
            let mut msg = zmq::Message::new();
            for request_nbr in 0..10 {
                println!("Sending Hello {}...", request_nbr);
                requester.send("Hello", 0).unwrap();
                requester.recv(&mut msg, 0).unwrap();
                println!("Received World {}: {}", msg.as_str().unwrap(), request_nbr);
            }
        });

        thread::sleep(Duration::from_millis(1000));

        let server: thread::JoinHandle<_> = thread::spawn(move || {
            let mut msg = zmq::Message::new();
            for _ in 0..10 {
                responder.recv(&mut msg, 0).unwrap();
                println!("Received {}", msg.as_str().unwrap());
                responder.send("World", 0).unwrap();
            }
        });

        _ = server.join();
        _ = client.join();
    }
}

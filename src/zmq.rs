use std::collections::HashMap;
use std::sync::atomic::AtomicU32;
use std::sync::Arc;
use std::sync::Mutex;
use std::task::Context;
use std::thread;

use chashmap::CHashMap;
use log::debug;
use log::info;
use protobuf::Message;
use protobuf::MessageField;
use zmq::Socket;

use crate::constants::INCOMING_PORT;
use crate::env::Cluster;
use crate::env::Env;
use crate::env::Executor;
use crate::env::ProcessId;
use crate::env::ProcessType;
use crate::env::Receiver;
use crate::env::Router;
use crate::env::Sender;
use crate::local::EnvState;
use crate::proto::proto;

impl ProcessId {
    pub fn addr_sender(&self) -> String {
        format!("tcp://{}:{}", self.ip, INCOMING_PORT)
    }
}

#[derive(Clone)]
pub struct ZMQRouter {
    context: zmq::Context,
}

impl ZMQRouter {
    fn new(c: zmq::Context) -> Self {
        Self { context: c }
    }
}

pub struct WireMessage {
    to: ProcessId,
    message: crate::message::Message,
}

impl Into<proto::WireMessage> for WireMessage {
    fn into(self) -> proto::WireMessage {
        let mut def = proto::WireMessage::default();
        def.to = MessageField::some(self.to.into());
        def.message = MessageField::some(self.message.into());
        def
    }
}

impl From<proto::WireMessage> for WireMessage {
    fn from(value: proto::WireMessage) -> Self {
        Self {
            to: ProcessId::from(value.to.unwrap()),
            message: crate::message::Message::from(value.message.unwrap()),
        }
    }
}

static SOCK_COUNT: AtomicU32 = AtomicU32::new(0);

impl Router for ZMQRouter {
    fn send(&self, id: &ProcessId, m: crate::message::Message) {
        let s = self.context.socket(zmq::PUSH).unwrap();
        assert!(s.connect(&id.addr_sender()).is_ok());
        // info!(
        //     "socket count: {}, from: {}, message: {}",
        //     SOCK_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
        //     id,
        //     m
        // );
        let p: crate::proto::proto::WireMessage = WireMessage {
            to: id.clone(),
            message: m,
        }
        .into();
        s.send(p.write_to_bytes().unwrap(), 0).unwrap()
    }
}

pub struct ZMQPoller<S>
where
    S: Sender,
{
    context: zmq::Context,
    m: Arc<CHashMap<ProcessId, S>>,
}

impl<S> ZMQPoller<S>
where
    S: Sender,
{
    fn new(c: zmq::Context) -> ZMQPoller<S> {
        Self {
            context: c,
            m: Arc::new(CHashMap::new()),
        }
    }

    pub fn start(&self, addr: &str) {
        let server = self.context.socket(zmq::PULL).unwrap();
        assert!(server.bind(addr).is_ok());

        loop {
            match Socket::recv_bytes(&server, 0) {
                Ok(b) => match crate::proto::proto::WireMessage::parse_from_bytes(&b) {
                    Ok(m) => self.handle(m.into()),
                    Err(e) => panic!("unexpected error while parsing {}", e),
                },
                Err(e) => panic!("polling encountered error {}", e),
            }
        }
    }

    fn handle(&self, m: WireMessage) {
        self.m
            .get(&m.to)
            .map(|s| {
                debug!("polled message {}: {}", m.to, m.message);
                s.send(&m.message)
            })
            .unwrap_or_else(|| panic!("unable to find the sender for id {}", m.to))
    }
}

impl<S> ZMQPoller<S>
where
    S: Sender,
{
    fn add(&self, id: ProcessId, new_sender: S) {
        self.m.insert(id, new_sender);
    }
}

pub struct ZMQEnv<R, S>
where
    R: Receiver,
    S: Sender,
{
    new_channel_fn: fn() -> (R, S),
    router: ZMQRouter,
    pub poller: ZMQPoller<S>,
    state: EnvState,
}

impl<R, S> Env<ZMQRouter> for ZMQEnv<R, S>
where
    R: Receiver + Send,
    S: Sender + Send + Sync,
{
    fn router(&self) -> &ZMQRouter {
        &self.router
    }

    fn register<E: Executor + Send + 'static>(
        &'static self,
        id: ProcessId,
        t: ProcessType,
        executor: E,
    ) {
        let (new_receiver, new_sender) = (self.new_channel_fn)();
        self.poller.add(id.clone(), new_sender);
        let clone = self.clone();
        let jh = thread::spawn(move || {
            executor.exec(new_receiver, clone);
        });
        self.state.add(&id, t, jh);
    }

    fn cluster(&self) -> &Cluster {
        &self.state.cluster
    }

    fn new_id(&self) -> u32 {
        self.state.new_id()
    }
}

impl<R: Receiver, S: Sender> ZMQEnv<R, S> {
    pub fn new(new_channel_fn: fn() -> (R, S)) -> ZMQEnv<R, S> {
        let context = zmq::Context::new();
        ZMQEnv {
            new_channel_fn: new_channel_fn,
            router: ZMQRouter::new(context.clone()),
            poller: ZMQPoller::new(context),
            state: EnvState::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        net::{IpAddr, Ipv4Addr},
        thread,
        time::Duration,
    };

    use crate::{
        env::{Env, ProcessId, Router},
        message::Message,
        pval::Command,
    };

    use super::ZMQEnv;

    #[test]
    fn zmq_mock() {
        let env = ZMQEnv::new(|| {
            let (s, r) = crossbeam::channel::unbounded();
            return (r, s);
        });

        let local_host = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let port = 6060;
        let id = ProcessId::new(local_host, port, env.new_id());
        let addr = id.addr_sender();

        let (s, r) = crossbeam::channel::unbounded();

        let poller = env.poller;
        let router = env.router;

        poller.add(id.clone(), s);
        thread::spawn(move || {
            poller.start(&addr);
        });

        let msg = Message::Request(
            id.clone(),
            Command::new_from_str(id.clone(), format!("Request:1",), format!("Op:1")),
        );

        router.send(&id, msg.clone());
        router.send(&id, msg.clone());

        assert_eq!(r.recv().unwrap().id(), msg.id());
        assert_eq!(r.recv().unwrap().id(), msg.id());
    }

    #[test]
    fn zmp_multi_message() {
        let ctx = zmq::Context::new();
        let receiver = ctx.socket(zmq::PULL).unwrap();
        receiver.bind("tcp://127.0.0.1:5555").unwrap();

        let sender = ctx.socket(zmq::PUSH).unwrap();
        sender.connect("tcp://127.0.0.1:5555").unwrap();
        sender.send("bar", 0).expect("send failed");
        sender.send("bar", 0).expect("send failed");

        assert_eq!(&receiver.recv_bytes(0).unwrap(), b"bar");
        assert_eq!(&receiver.recv_bytes(0).unwrap(), b"bar");
    }

    #[test]
    fn zmp_multi_server() {
        let context = zmq::Context::new();
        let server1 = context.socket(zmq::REP).unwrap();
        assert!(server1.bind("tcp://*:5555").is_ok());

        let server2 = context.socket(zmq::REP).unwrap();
        assert!(server2.bind("tcp://*:5556").is_ok());

        let client = context.socket(zmq::REQ).unwrap();
        assert!(client.connect("tcp://localhost:5555").is_ok());
        client.send("Hello server 1", 0).unwrap();

        let client2 = context.socket(zmq::REQ).unwrap();
        assert!(client2.connect("tcp://localhost:5556").is_ok());
        client2.send("Hello server 2", 0).unwrap();

        let mut msg = zmq::Message::new();
        server1.recv(&mut msg, 0).unwrap();
        assert!(msg.as_str().unwrap().eq("Hello server 1"));

        server2.recv(&mut msg, 0).unwrap();
        assert!(msg.as_str().unwrap().eq("Hello server 2"));
    }
}

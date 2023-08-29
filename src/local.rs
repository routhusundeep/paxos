use crossbeam::channel;
use log::debug;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use super::{
    env::{Cluster, Env, Executor, GetErr, ProcessId, ProcessType, Receiver, Router, Sender},
    message::Message,
};

impl Sender for channel::Sender<Message> {
    fn send(&self, m: &Message) {
        let res = self.send(m.clone());
        match res {
            Ok(()) => {}
            Err(e) => {
                debug!("errored during send {}", e)
            }
        }
    }
}

impl Receiver for channel::Receiver<Message> {
    fn try_get(&self) -> Result<Message, GetErr> {
        let g = self.try_recv();
        match g {
            Ok(m) => return Ok(m),
            Err(_) => Err(GetErr::None),
        }
    }

    fn get(&self, sleep: u64) -> Message {
        loop {
            match self.try_get() {
                Ok(m) => return m,
                Err(_) => thread::sleep(Duration::from_nanos(sleep)),
            }
        }
    }
}

pub struct RouterMap<S>
where
    S: Sender,
{
    m: Mutex<HashMap<ProcessId, S>>,
}

impl<S> RouterMap<S>
where
    S: Sender,
{
    fn new(sleep: u64) -> RouterMap<S> {
        RouterMap {
            m: Mutex::new(HashMap::new()),
        }
    }
}

impl<S: Sender> RouterMap<S> {
    fn add(&self, id: ProcessId, r: S) {
        self.m.lock().unwrap().insert(id, r);
    }
}

impl<S: Sender> Router for RouterMap<S> {
    fn send(&self, id: &ProcessId, m: &Message) {
        debug!("{} ----> {} ...... message: {}", m.id(), id, m);
        let guard = self.m.lock();
        match guard.unwrap().get_mut(&id) {
            Some(r) => r.send(&m),
            None => panic!("not possible"),
        }
    }
}

#[derive(Clone)]
pub struct InMemEnv<R, S>
where
    R: Receiver,
    S: Sender,
{
    id_gen: Arc<AtomicU32>,
    new_channel_fn: fn() -> (R, S),
    sender: Arc<RouterMap<S>>,
    cluster: Arc<Cluster>,
    join_handles: Arc<Mutex<Vec<JoinHandle<()>>>>,
}

impl<R, S> Env<RouterMap<S>> for InMemEnv<R, S>
where
    R: Receiver + Send,
    S: Sender + Send,
{
    fn router(&self) -> &RouterMap<S> {
        &self.sender
    }

    fn register<E: Executor + Send + 'static>(
        &'static self,
        id: ProcessId,
        t: ProcessType,
        executor: E,
    ) {
        let (new_receiver, new_sender) = (self.new_channel_fn)();
        self.sender.add(id.clone(), new_sender);
        self.cluster.add(t, id.clone());
        let clone = self.clone();
        let jh = thread::spawn(move || {
            executor.exec(new_receiver, clone);
        });
        self.join_handles.lock().unwrap().push(jh);
    }

    fn cluster(&self) -> &Cluster {
        &self.cluster
    }

    fn new_id(&self) -> u32 {
        self.id_gen.fetch_add(1, Ordering::SeqCst)
    }
}

impl<R: Receiver, S: Sender> InMemEnv<R, S> {
    pub fn new(new_channel_fn: fn() -> (R, S)) -> InMemEnv<R, S> {
        let router = RouterMap::new(1000);
        InMemEnv {
            id_gen: Arc::new(AtomicU32::new(0)),
            new_channel_fn: new_channel_fn,
            sender: Arc::new(router),
            cluster: Arc::new(Cluster::new()),
            join_handles: Arc::new(Mutex::new(vec![])),
        }
    }
}

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
    fn new() -> RouterMap<S> {
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
    fn send(&self, id: &ProcessId, m: Message) {
        debug!("{} ----> {} ...... message: {}", m.id(), id, m);
        let guard = self.m.lock();
        match guard.unwrap().get_mut(&id) {
            Some(r) => r.send(&m),
            None => panic!("not possible"),
        }
    }
}

#[derive(Clone)]
pub struct EnvState {
    id_gen: Arc<AtomicU32>,
    pub cluster: Arc<Cluster>,
    join_handles: Arc<Mutex<Vec<JoinHandle<()>>>>,
}

impl EnvState {
    pub fn new() -> Self {
        Self {
            id_gen: Arc::new(AtomicU32::new(0)),
            cluster: Arc::new(Cluster::new()),
            join_handles: Arc::new(Mutex::new(vec![])),
        }
    }
    pub fn add(&self, pid: &ProcessId, t: ProcessType, jh: JoinHandle<()>) {
        self.cluster.add(t, pid.clone());
        self.join_handles.lock().unwrap().push(jh);
    }

    pub fn new_id(&self) -> u32 {
        self.id_gen.fetch_add(1, Ordering::SeqCst)
    }
}

#[derive(Clone)]
pub struct InMemEnv<R, S>
where
    R: Receiver,
    S: Sender,
{
    new_channel_fn: fn() -> (R, S),
    router: Arc<RouterMap<S>>,
    state: EnvState,
}

impl<R, S> Env<RouterMap<S>> for InMemEnv<R, S>
where
    R: Receiver + Send,
    S: Sender + Send,
{
    fn router(&self) -> &RouterMap<S> {
        &self.router
    }

    fn register<E: Executor + Send + 'static>(
        &'static self,
        id: ProcessId,
        t: ProcessType,
        executor: E,
    ) {
        let (new_receiver, new_sender) = (self.new_channel_fn)();
        self.router.add(id.clone(), new_sender);
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

impl<R: Receiver, S: Sender> InMemEnv<R, S> {
    pub fn new(new_channel_fn: fn() -> (R, S)) -> InMemEnv<R, S> {
        InMemEnv {
            new_channel_fn: new_channel_fn,
            router: Arc::new(RouterMap::new()),
            state: EnvState::new(),
        }
    }
}

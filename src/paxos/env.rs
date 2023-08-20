use std::{
    collections::HashMap,
    error::Error,
    fmt::Display,
    sync::{Mutex, RwLock},
};

use super::message::Message;

#[derive(Eq, Ord, PartialEq, PartialOrd, Clone, Hash, Debug)]
pub struct ProcessId {
    name: String,
}
impl ProcessId {
    pub fn new(s: String) -> ProcessId {
        ProcessId { name: s }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }
}

impl Display for ProcessId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(PartialEq, Eq, Hash)]
pub enum ProcessType {
    Acceptor,
    Replica,
    Leader,
    Scout,
    Commander,
}

pub struct Cluster {
    map: Mutex<HashMap<ProcessType, Vec<ProcessId>>>,
}

impl Cluster {
    pub fn new() -> Cluster {
        Cluster {
            map: Mutex::new(HashMap::new()),
        }
    }

    fn copy_vec(s: &Vec<ProcessId>) -> Vec<ProcessId> {
        let mut res = vec![];
        for i in s.into_iter() {
            res.push(i.clone());
        }
        return res;
    }

    fn get(&self, t: ProcessType) -> Vec<ProcessId> {
        return Self::copy_vec(self.map.lock().unwrap().get(&t).unwrap());
    }

    pub fn acceptors(&self) -> Vec<ProcessId> {
        return self.get(ProcessType::Acceptor);
    }

    pub fn replicas(&self) -> Vec<ProcessId> {
        return self.get(ProcessType::Replica);
    }

    pub fn leaders(&self) -> Vec<ProcessId> {
        return self.get(ProcessType::Leader);
    }

    pub fn add(&self, t: ProcessType, id: ProcessId) {
        let guard = self.map.lock();
        let mut m = guard.unwrap();
        let o = m.get_mut(&t);
        if o.is_none() {
            m.insert(t, vec![id]);
        } else {
            o.unwrap().push(id);
        }
    }
}

pub trait Sender {
    fn send(&self, p: &ProcessId, m: &Message);
}

#[derive(Debug)]
pub enum GetErr {
    None,
}

pub trait Receiver {
    fn try_get(&self) -> Result<Message, GetErr>;
    fn add(&self, m: Message);
}

pub trait Executor<R, S>
where
    R: Receiver,
    S: Sender,
{
    fn exec<E: Env<R, S>>(self, env: &mut E);
}

pub trait Env<R, S>
where
    R: Receiver,
    S: Sender,
{
    fn register<E: Executor<R, S> + Send + 'static>(
        &mut self,
        id: ProcessId,
        t: ProcessType,
        executor: E,
    );
    fn read(&self, id: &ProcessId) -> Message;
    fn create_receiver(&self) -> R;
    fn sender(&self) -> &S;
    fn cluster(&self) -> &Cluster;
}

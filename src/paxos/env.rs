use std::{collections::HashMap, fmt::Display, sync::Mutex};

use uuid::Uuid;

use super::message::Message;

#[derive(Eq, Ord, PartialEq, PartialOrd, Hash, Clone, Debug)]
pub enum ProcessId {
    Local(Uuid),
}
impl ProcessId {
    pub fn new() -> ProcessId {
        let id = Uuid::new_v4();
        ProcessId::Local(id)
    }
}

impl Display for ProcessId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessId::Local(u) => write!(f, "{}", u),
        }
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
    map: Mutex<HashMap<ProcessType, Vec<ProcessId>>>, // TODO: make it concurrent hashmap
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
        let guard = self.map.lock();
        return Self::copy_vec(guard.unwrap().get(&t).unwrap());
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

pub trait Router {
    fn send(&self, p: &ProcessId, m: &Message);
}

pub trait Sender {
    fn send(&self, m: &Message);
}

#[derive(Debug)]
pub enum GetErr {
    None,
}

pub trait Receiver {
    fn try_get(&self) -> Result<Message, GetErr>;
    fn get(&self, sleep: u64) -> Message;
}

pub trait Executor {
    fn exec<R: Receiver, T: Router, E: Env<T>>(self, reciever: R, env: &mut E);
}

pub trait Env<T>
where
    T: Router,
{
    fn register<E: Executor + Send + 'static>(
        &mut self,
        id: ProcessId,
        t: ProcessType,
        executor: E,
    );
    fn router(&self) -> T;
    fn cluster(&self) -> &Cluster;
}

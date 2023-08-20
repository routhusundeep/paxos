use crossbeam::channel;
use log::{debug, info, trace};
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use super::{
    env::{Cluster, Env, Executor, GetErr, ProcessId, ProcessType, Receiver, Sender},
    message::Message,
};

#[derive(Clone)]
pub struct Channel<T> {
    sleep: u64,
    s: channel::Sender<T>,
    r: channel::Receiver<T>,
}

impl<T> Channel<T> {
    fn send(&self, m: T) -> Result<(), channel::SendError<T>> {
        self.s.send(m)
    }

    fn get(&self) -> Result<T, GetErr> {
        // println!("{:#?}", thread::current().id());
        let g = self.r.try_recv();
        match g {
            Ok(m) => return Ok(m),
            Err(_) => Err(GetErr::None),
        }
    }
}

impl Receiver for Channel<Message> {
    fn try_get(&self) -> Result<Message, GetErr> {
        self.get()
    }

    fn add(&self, m: Message) {
        let g = self.send(m);
        g.unwrap()
    }
}

impl<T> Channel<T> {
    pub fn new(sleep: u64) -> Channel<T> {
        let (s, r): (channel::Sender<T>, channel::Receiver<T>) = channel::unbounded();
        Channel {
            sleep: sleep,
            s: s,
            r,
        }
    }

    pub fn new_default() -> Channel<T> {
        Channel::new(1000)
    }
}

pub struct MessageRouter<R> {
    sleep: u64,
    m: Mutex<HashMap<ProcessId, R>>,
}

impl<R: Receiver> MessageRouter<R> {
    fn new(sleep: u64) -> MessageRouter<R> {
        MessageRouter {
            sleep: sleep,
            m: Mutex::new(HashMap::new()),
        }
    }

    fn put(&self, id: ProcessId, r: R) {
        self.m.lock().unwrap().insert(id, r);
    }

    fn get(&self, id: &ProcessId) -> Message {
        loop {
            {
                // print!("{:#?}", thread::current().id());
                let guard = self.m.lock();
                match guard.unwrap().get(id).unwrap().try_get() {
                    Ok(m) => return m,
                    Err(_) => {}
                }
            }
            thread::sleep(Duration::from_nanos(self.sleep));
        }
    }

    fn send(&self, id: &ProcessId, m: Message) {
        match self.m.lock().unwrap().get_mut(&id) {
            Some(r) => r.add(m.clone()),
            None => panic!("not possible"),
        }
    }
}

impl<R: Receiver> Sender for MessageRouter<R> {
    fn send(&self, id: &ProcessId, m: &Message) {
        debug!("{} ----> {} ...... message: {}", m.id(), id, m);
        self.send(id, m.clone())
    }
}

#[derive(Clone)]
pub struct InMemEnv<R>
where
    R: Receiver,
{
    new_receiver_fn: fn() -> R,
    sender: Arc<MessageRouter<R>>,
    cluster: Arc<Cluster>,
}

impl<R> Env<R, MessageRouter<R>> for InMemEnv<R>
where
    R: Receiver + Send + Clone + 'static,
{
    fn create_receiver(&self) -> R {
        (self.new_receiver_fn)()
    }

    fn sender(&self) -> &MessageRouter<R> {
        &self.sender
    }

    fn register<E: Executor<R, MessageRouter<R>> + Send + 'static>(
        &mut self,
        id: ProcessId,
        t: ProcessType,
        executor: E,
    ) {
        let new_receiver = <InMemEnv<R> as Env<R, MessageRouter<R>>>::create_receiver(self);
        self.sender.put(id.clone(), new_receiver);
        self.cluster.add(t, id.clone());
        let mut clone = self.clone();
        thread::spawn(move || {
            executor.exec(&mut clone);
        });
    }

    fn read(&self, id: &ProcessId) -> Message {
        let m = self.sender.get(id);
        debug!("{} <---- {} ...... message: {}", id, m.id(), m);
        return m;
    }

    fn cluster(&self) -> &Cluster {
        &self.cluster
    }
}

impl<R: Receiver> InMemEnv<R> {
    pub fn new(new_receiver_fn: fn() -> R) -> InMemEnv<R> {
        let router = MessageRouter::new(1000);
        InMemEnv {
            new_receiver_fn: new_receiver_fn,
            sender: Arc::new(router),
            cluster: Arc::new(Cluster::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fmt::format,
        thread::{self, JoinHandle},
        time::Duration,
    };

    use crate::paxos::{env::ProcessId, local::Channel, message::Message, pval::Command};

    use super::MessageRouter;

    #[test]
    fn router_test() {
        let router = MessageRouter::new(1000);

        let n_chan = 5;
        let n_send = 5;
        let n_get = 5;

        for i in 1..n_chan + 1 {
            let id = ProcessId::new(format!("{}", 1));
            router.put(id, Channel::new_default());
        }

        for i in 1..n_chan + 1 {
            let id = ProcessId::new(format!("{}", 1));
            for j in 1..n_send + 1 {
                router.send(
                    &id.clone(),
                    Message::Request(
                        id.clone(),
                        Command::new_from_str(
                            id.clone(),
                            format!("{}:{}", i, j).to_string(),
                            "".to_string(),
                        ),
                    ),
                );
            }
        }

        for i in 1..n_chan + 1 {
            let id = ProcessId::new(format!("{}", 1));
            for j in 1..n_send + 1 {
                println!("{}", router.get(&id));
            }
        }
    }

    #[test]
    fn channel_test() {
        let wq: Channel<usize> = Channel::new(100);

        let nget: usize = 100;
        let nput: usize = 100;

        let mut handles: Vec<JoinHandle<usize>> = vec![];
        for i in 1..nput {
            let val = wq.clone();
            thread::spawn(move || {
                val.send(i);
            });
        }

        for _ in 1..nget {
            let mut val = wq.clone();
            let jh = thread::spawn(move || {
                return val.get().unwrap();
            });
            handles.push(jh);
        }

        let mut res = vec![];
        for jh in handles.into_iter() {
            let v = jh.join().unwrap();
            res.push(v);
        }

        res.sort();
        assert_eq!(nget, res.len() + 1);

        let exp: Vec<usize> = (1..nget).collect();
        assert_eq!(0, res.iter().zip(&exp).filter(|&(a, b)| a != b).count());
    }
}

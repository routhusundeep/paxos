use log::info;
use log::trace;

use super::env::Env;
use super::env::Executor;
use super::env::ProcessId;
use super::env::Receiver;
use super::env::Sender;
use super::message::Message;
use super::pval::BallotNumber;
use super::pval::PValue;
use std::collections::HashSet;

pub struct Acceptor {
    me: ProcessId,
    ballot: BallotNumber,
    accepted: HashSet<Box<PValue>>,
}

impl Acceptor {
    pub fn new(id: ProcessId) -> Acceptor {
        Acceptor {
            me: id.clone(),
            ballot: BallotNumber::first(id),
            accepted: HashSet::new(),
        }
    }
}

impl<R: Receiver, S: Sender> Executor<R, S> for Acceptor {
    fn exec<E: Env<R, S>>(mut self, env: &mut E) {
        loop {
            let msg = env.read(&self.me);

            match msg {
                Message::P1A(src, ballot) => {
                    let bc = ballot.clone();
                    if self.ballot < ballot {
                        self.ballot = ballot;
                    }

                    env.sender().send(
                        &src,
                        &Message::P1B(self.me.clone(), bc, copy_set(&self.accepted)),
                    );
                }
                Message::P2A(src, ballot, slot, command) => {
                    if self.ballot <= ballot {
                        self.ballot = ballot;
                        self.accepted.insert(Box::new(PValue::new(
                            self.ballot.clone(),
                            slot,
                            command,
                        )));
                    }
                    env.sender().send(
                        &src,
                        &Message::P2B(self.me.clone(), self.ballot.clone(), slot),
                    )
                }
                _ => panic!("unexpected message"),
            }
        }
    }
}

fn copy_set(s: &HashSet<Box<PValue>>) -> HashSet<Box<PValue>> {
    let mut res = HashSet::new();
    for i in s.into_iter() {
        res.insert(i.clone());
    }
    return res;
}

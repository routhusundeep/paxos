use log::info;
use log::trace;

use super::ds::Accepted;
use super::env::Env;
use super::env::Executor;
use super::env::ProcessId;
use super::env::Receiver;
use super::env::Router;
use super::env::Sender;
use super::message::Message;
use super::pval::BallotNumber;
use super::pval::PValue;
use super::pval::SlotNumber;
use std::cmp::max;
use std::collections::HashMap;
use std::collections::HashSet;

pub struct Acceptor {
    me: ProcessId,
    ballot: BallotNumber,
    accepted: Accepted,
}

impl Acceptor {
    pub fn new(id: ProcessId) -> Acceptor {
        Acceptor {
            me: id.clone(),
            ballot: BallotNumber::first(id),
            accepted: Accepted::new(),
        }
    }
}

impl Executor for Acceptor {
    fn exec<R: Receiver, T: Router, E: Env<T>>(mut self, reciever: R, env: &mut E) {
        loop {
            let msg = reciever.get(1000);

            match msg {
                Message::P1A(src, ballot) => {
                    let bc = ballot.clone();
                    if self.ballot < ballot {
                        self.ballot = ballot;
                    }

                    env.router().send(
                        &src,
                        &Message::P1B(self.me.clone(), bc, self.accepted.clone()),
                    );
                }
                Message::P2A(src, ballot, slot, command) => {
                    if self.ballot <= ballot {
                        self.ballot = ballot;
                        let p = PValue::new(self.ballot.clone(), slot, command);
                        self.accepted.insert(slot, p);
                    }
                    env.router().send(
                        &src,
                        &Message::P2B(self.me.clone(), self.ballot.clone(), slot),
                    )
                }
                _ => panic!("unexpected message"),
            }
        }
    }
}

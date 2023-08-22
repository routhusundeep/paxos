use core::panic;
use std::collections::HashMap;

use log::{info, trace};

use super::{
    env::{Env, Executor, ProcessId, Receiver, Router, Sender},
    message::Message,
    pval::{Command, SlotNumber},
};

pub struct Replica {
    me: ProcessId,
    slot: SlotNumber,
    proposals: HashMap<SlotNumber, Command>,
    decisions: HashMap<SlotNumber, Command>,
}

impl Replica {
    pub fn new(id: ProcessId) -> Replica {
        Replica {
            me: id,
            slot: 1,
            proposals: HashMap::new(),
            decisions: HashMap::new(),
        }
    }

    pub fn propose<T: Router, E: Env<T>>(&mut self, c: Command, env: &E) {
        if self.decisions.values().all(|com: &Command| c != *com) {
            let mut i = 1;
            loop {
                if !self.proposals.contains_key(&i) && !self.decisions.contains_key(&i) {
                    self.proposals.insert(i, c.clone());
                    for l in env.cluster().leaders().iter() {
                        env.router()
                            .send(l, &Message::Propose(self.me.clone(), i, c.clone()));
                    }
                    break;
                }
                i += 1;
            }
        }
    }

    pub fn perform(&mut self, c: Command) {
        info!("Replica {} performed {}", self.me, c);
        if self.decisions.values().any(|com: &Command| c == *com) {
            self.slot += 1
        }
    }
}

impl Executor for Replica {
    fn exec<R:Receiver, T: Router, E: Env<T>>(mut self, reciever: R, env: &mut E) {
        loop {
            let msg = reciever.get(1000);

            match msg {
                Message::Request(_, command) => {
                    self.propose(command, env);
                }
                Message::Decision(_, slot, command) => {
                    self.decisions.insert(slot, command);
                    loop {
                        match self.decisions.get(&self.slot) {
                            Some(c) => {
                                let cclone = c.clone();
                                match self.proposals.get(&self.slot) {
                                    Some(c2) => {
                                        if c != c2 {
                                            self.propose(c2.clone(), env);
                                        }
                                    }
                                    None => {}
                                }
                                self.perform(cclone);
                            }
                            None => break,
                        }
                    }
                }
                _ => panic!("unexpected"),
            }
        }
    }
}

use core::panic;
use std::collections::{HashMap, HashSet};

use super::{
    constants::SLEEP_TIME,
    ds::Accepted,
    env::{Env, Executor, ProcessId, ProcessType, Receiver, Router},
    message::Message,
    pval::{BallotNumber, Command, SlotNumber},
};

pub struct Leader {
    me: ProcessId,
    ballot: BallotNumber,
    active: bool,
    proposals: HashMap<SlotNumber, Box<Command>>,
}

impl Leader {
    pub fn new(me: ProcessId) -> Leader {
        Leader {
            me: me.clone(),
            ballot: BallotNumber::first(me),
            active: false,
            proposals: HashMap::new(),
        }
    }

    fn scout<T: Router, E: Env<T>>(&self, ballot: BallotNumber, env: &mut E) {
        let sid = ProcessId::new();
        let scout = Scout::new(sid.clone(), self.me.clone(), self.ballot.clone());
        env.register(scout.me.clone(), ProcessType::Scout, scout);
    }

    fn commander<T: Router, E: Env<T>>(
        &self,
        ballot: BallotNumber,
        slot: SlotNumber,
        command: Command,
        env: &mut E,
    ) {
        let cid = ProcessId::new();
        let commander = Commander::new(&cid, &self.me, ballot, slot, command);
        env.register(commander.me.clone(), ProcessType::Commander, commander);
    }
}

impl Executor for Leader {
    fn exec<R: Receiver, T: Router, E: Env<T>>(mut self, reciever: R, env: &mut E) {
        self.scout(self.ballot.clone(), env);
        loop {
            let msg = reciever.get(SLEEP_TIME);

            match msg {
                Message::Propose(_, slot, command) => {
                    if !self.proposals.contains_key(&slot) {
                        self.proposals.insert(slot, Box::new(command.clone()));
                        if self.active {
                            self.commander(self.ballot.clone(), slot, command, env);
                        }
                    }
                }
                Message::Adopt(_, ballot, values) => {
                    if self.ballot == ballot {
                        let mut max: HashMap<SlotNumber, BallotNumber> = HashMap::new();
                        for (s, pv) in values.iter() {
                            let bn = max.get(s);

                            if bn.map_or(true, |p| p < &self.ballot) {
                                max.insert(pv.slot, pv.ballot.clone());
                                self.proposals.insert(pv.slot, Box::new(pv.command.clone()));
                            }
                        }
                    }
                    for (sn, c) in self.proposals.iter() {
                        self.commander(self.ballot.clone(), *sn, *c.clone(), env);
                    }
                    self.active = true;
                }
                Message::Preempt(_, ballot) => {
                    if self.ballot < ballot {
                        let ballot = BallotNumber::new(ballot.round + 1, self.me.clone());
                        self.scout(ballot, env);
                        self.active = false;
                    }
                }
                _ => panic!("unexpected"),
            }
        }
    }
}

struct Scout {
    me: ProcessId,
    leader: ProcessId,
    ballot: BallotNumber,
}

impl Scout {
    fn new(id: ProcessId, leader: ProcessId, ballot: BallotNumber) -> Scout {
        Scout {
            me: id,
            leader: leader,
            ballot: ballot,
        }
    }
}

impl Executor for Scout {
    fn exec<R: Receiver, T: Router, E: Env<T>>(self, reciever: R, env: &mut E) {
        let msg = Message::P1A(self.me.clone(), self.ballot.clone());
        let mut wait: HashSet<ProcessId> = HashSet::new();
        for a in env.cluster().acceptors().iter() {
            env.router().send(a, &msg);
            wait.insert(a.clone());
        }

        let mut values: Accepted = Accepted::new();
        while 2 * wait.len() >= env.cluster().acceptors().len() {
            let msg = reciever.get(SLEEP_TIME);

            match msg {
                Message::P1B(pid, ballot, accepted) => {
                    if ballot != self.ballot {
                        env.router()
                            .send(&self.leader, &Message::Preempt(self.me.clone(), ballot));
                        return;
                    }
                    if wait.contains(&pid) {
                        wait.remove(&pid);
                        values.extend(accepted);
                    }
                }
                _ => panic!("not expected"),
            }

            env.router().send(
                &self.leader,
                &Message::Adopt(self.me.clone(), self.ballot.clone(), values.clone()),
            )
        }
    }
}

struct Commander {
    me: ProcessId,
    leader: ProcessId,
    ballot: BallotNumber,
    slot: SlotNumber,
    command: Command,
}

impl Commander {
    fn new(
        id: &ProcessId,
        leader: &ProcessId,
        ballot: BallotNumber,
        slot: SlotNumber,
        command: Command,
    ) -> Commander {
        Commander {
            me: id.clone(),
            leader: leader.clone(),
            ballot: ballot,
            slot: slot,
            command: command,
        }
    }
}

impl Executor for Commander {
    fn exec<R: Receiver, T: Router, E: Env<T>>(self, reciever: R, env: &mut E) {
        let msg = Message::P2A(
            self.me.clone(),
            self.ballot.clone(),
            self.slot,
            self.command.clone(),
        );
        let mut wait: HashSet<ProcessId> = HashSet::new();
        for a in env.cluster().acceptors().iter() {
            env.router().send(a, &msg);
            wait.insert(a.clone());
        }

        while 2 * wait.len() >= env.cluster().acceptors().len() {
            let msg = reciever.get(SLEEP_TIME);
            match msg {
                Message::P2B(pid, ballot, slot) => {
                    if self.ballot == ballot {
                        if wait.contains(&pid) {
                            wait.remove(&pid);
                        }
                    } else {
                        env.router()
                            .send(&self.leader, &Message::Preempt(self.me.clone(), ballot));
                        return;
                    }
                }
                _ => panic!("not expected"),
            }
        }

        for r in env.cluster().replicas().iter() {
            env.router().send(
                r,
                &Message::Decision(self.me.clone(), self.slot, self.command.clone()),
            );
        }
    }
}

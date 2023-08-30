use core::panic;
use std::collections::{hash_map::Iter, HashMap, HashSet};

use super::{
    constants::SLEEP_TIME,
    ds::Accepted,
    env::{Env, Executor, ProcessId, ProcessType, Receiver, Router},
    message::Message,
    pval::{BallotNumber, Command, SlotNumber},
};

#[derive(PartialEq, Eq)]
enum Status {
    DONE,
    PENDING,
}

struct Proposal {
    command: Command,
    status: Status,
}

struct Proposals {
    m: HashMap<SlotNumber, Proposal>,
}
impl Proposals {
    fn has(&self, slot: &u64) -> bool {
        self.m.contains_key(slot)
    }

    fn insert(&mut self, slot: u64, command: Command) {
        self.m.insert(
            slot,
            Proposal {
                command: command,
                status: Status::PENDING,
            },
        );
    }

    fn pending(&self) -> impl Iterator<Item = (&SlotNumber, &Proposal)> {
        self.m.iter().filter(|i| i.1.status == Status::PENDING)
    }

    fn done(&mut self, slot: &u64) {
        let x = self.m.get_mut(slot).expect("should always be present");
        x.status = Status::DONE
    }
}

pub struct Leader {
    me: ProcessId,
    ballot: BallotNumber,
    active: bool,
    proposals: Proposals,
}

impl Leader {
    pub fn new(me: ProcessId) -> Leader {
        Leader {
            me: me.clone(),
            ballot: BallotNumber::first(me),
            active: false,
            proposals: Proposals { m: HashMap::new() },
        }
    }

    fn scout<T: Router, E: Env<T>>(&self, ballot: BallotNumber, env: &'static E) {
        let sid = ProcessId::new(self.me.ip, self.me.port, env.new_id());
        let scout = Scout::new(sid.clone(), self.me.clone(), self.ballot.clone());
        env.register(scout.me.clone(), ProcessType::Scout, scout);
    }

    fn commander<T: Router, E: Env<T>>(
        &self,
        ballot: BallotNumber,
        slot: SlotNumber,
        command: Command,
        env: &'static E,
    ) {
        let cid = ProcessId::new(self.me.ip, self.me.port, env.new_id());
        let commander = Commander::new(&cid, &self.me, ballot, slot, command);
        env.register(commander.me.clone(), ProcessType::Commander, commander);
    }
}

impl Executor for Leader {
    fn exec<R: Receiver, T: Router, E: Env<T>>(mut self, reciever: R, env: &'static E) {
        self.scout(self.ballot.clone(), env);
        loop {
            let msg = reciever.get(SLEEP_TIME);

            match msg {
                Message::Propose(_, slot, command) => {
                    if !self.proposals.has(&slot) {
                        self.proposals.insert(slot, command.clone());
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
                                self.proposals.insert(pv.slot, pv.command.clone());
                            }
                        }

                        for (sn, c) in self.proposals.pending() {
                            self.commander(self.ballot.clone(), *sn, (*c).command.clone(), env);
                        }
                        self.active = true;
                    }
                }
                Message::Preempt(_, ballot) => {
                    if self.ballot < ballot {
                        self.ballot = BallotNumber::new(ballot.round + 1, self.me.clone());
                        self.scout(ballot, env);
                        self.active = false;
                    }
                }
                Message::Decision(id, slot, command) => {
                    self.proposals.done(&slot);
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
    fn exec<R: Receiver, T: Router, E: Env<T>>(self, reciever: R, env: &E) {
        let msg = Message::P1A(self.me.clone(), self.ballot.clone());
        let mut wait: HashSet<ProcessId> = HashSet::new();
        for a in env.cluster().acceptors().iter() {
            env.router().send(a, msg.clone());
            wait.insert(a.clone());
        }

        let mut values: Accepted = Accepted::new();
        while 2 * wait.len() >= env.cluster().acceptors().len() {
            let msg = reciever.get(SLEEP_TIME);

            match msg {
                Message::P1B(pid, ballot, accepted) => {
                    if ballot != self.ballot {
                        env.router()
                            .send(&self.leader, Message::Preempt(self.me.clone(), ballot));
                        return;
                    }
                    if wait.contains(&pid) {
                        wait.remove(&pid);
                        values.extend(accepted);
                    }
                }
                _ => panic!("not expected"),
            }
        }

        env.router().send(
            &self.leader,
            Message::Adopt(self.me.clone(), self.ballot.clone(), values.clone()),
        )
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
    fn exec<R: Receiver, T: Router, E: Env<T>>(self, reciever: R, env: &E) {
        let msg = Message::P2A(
            self.me.clone(),
            self.ballot.clone(),
            self.slot,
            self.command.clone(),
        );
        let mut wait: HashSet<ProcessId> = HashSet::new();
        for a in env.cluster().acceptors().iter() {
            env.router().send(a, msg.clone());
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
                            .send(&self.leader, Message::Preempt(self.me.clone(), ballot));
                        return;
                    }
                }
                _ => panic!("not expected"),
            }
        }

        let decision = Message::Decision(self.me.clone(), self.slot, self.command.clone());
        for r in env.cluster().replicas().iter() {
            env.router().send(r, decision.clone());
        }

        // send it to colocated leader
        env.router().send(&self.leader, decision);
    }
}

use super::{
    constants::SLEEP_TIME,
    ds::Accepted,
    env::{Env, Executor, ProcessId, Receiver, Router},
    message::Message,
    pval::{BallotNumber, PValue},
};

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
    fn exec<R: Receiver, T: Router, E: Env<T>>(mut self, reciever: R, env: &E) {
        loop {
            let msg = reciever.get(SLEEP_TIME);

            match msg {
                Message::P1A(src, ballot) => {
                    let bc = ballot.clone();
                    if self.ballot < ballot {
                        self.ballot = ballot;
                    }

                    env.router().send(
                        &src,
                        Message::P1B(self.me.clone(), bc, self.accepted.clone()),
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
                        Message::P2B(self.me.clone(), self.ballot.clone(), slot),
                    )
                }
                _ => panic!("unexpected message"),
            }
        }
    }
}

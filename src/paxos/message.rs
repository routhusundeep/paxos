use std::fmt::Display;

use super::{
    ds::Accepted,
    env::ProcessId,
    pval::{BallotNumber, Command, SlotNumber},
};

#[derive(Clone, Debug)]
pub enum Message {
    P1A(ProcessId, BallotNumber),
    P1B(ProcessId, BallotNumber, Accepted),
    P2A(ProcessId, BallotNumber, SlotNumber, Command),
    P2B(ProcessId, BallotNumber, SlotNumber),
    Preempt(ProcessId, BallotNumber),
    Adopt(ProcessId, BallotNumber, Accepted),
    Decision(ProcessId, SlotNumber, Command),
    Request(ProcessId, Command),
    Propose(ProcessId, SlotNumber, Command),
}

impl Message {
    pub fn id(&self) -> &ProcessId {
        return match self {
            Message::P1A(id, _) => id,
            Message::P1B(id, _, _) => id,
            Message::P2A(id, _, _, _) => id,
            Message::P2B(id, _, _) => id,
            Message::Preempt(id, _) => id,
            Message::Adopt(id, _, _) => id,
            Message::Decision(id, _, _) => id,
            Message::Request(id, _) => id,
            Message::Propose(id, _, _) => id,
        };
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::P1A(id, ballot) => write!(f, "P1A({}, {})", id, ballot),
            Message::P1B(id, ballot, pval) => write!(f, "P1B({}, {} ,{:#?})", id, ballot, pval),
            Message::P2A(id, ballot, slot, command) => {
                write!(f, "P2A({}, {}, {}, {})", id, ballot, slot, command)
            }
            Message::P2B(id, ballot, slot) => write!(f, "P2B({}, {}, {})", id, ballot, slot),
            Message::Preempt(id, ballot) => write!(f, "PREEMPT({}, {})", id, ballot),
            Message::Adopt(id, ballot, vals) => write!(f, "ADOPT({}, {}, {:#?})", id, ballot, vals),
            Message::Decision(id, slot, command) => {
                write!(f, "DECISION({}, {}, {})", id, slot, command)
            }
            Message::Request(id, command) => write!(f, "REQUEST({}, {})", id, command),
            Message::Propose(id, slot, command) => {
                write!(f, "PROPOSE({}, {}, {})", id, slot, command)
            }
        }
    }
}

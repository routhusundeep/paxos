use super::env::ProcessId;
use std::fmt::Display;
use std::str;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PValue {
    pub ballot: BallotNumber,
    pub slot: SlotNumber,
    pub command: Command,
}
impl PValue {
    pub fn new(ballot: BallotNumber, slot: i64, command: Command) -> PValue {
        PValue {
            ballot: ballot,
            slot: slot,
            command: command,
        }
    }
}
impl Display for PValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PValue({},{},{})", self.ballot, self.slot, self.command)
    }
}

pub type SlotNumber = i64;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Debug)]
pub struct BallotNumber {
    pub round: i64,
    pub process_id: ProcessId,
}
impl BallotNumber {
    pub fn new(round: i64, process_id: ProcessId) -> BallotNumber {
        BallotNumber {
            round: round,
            process_id: process_id,
        }
    }

    pub fn first(pid: ProcessId) -> BallotNumber {
        BallotNumber::new(0, pid)
    }
}

impl Display for BallotNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BallotNumber({}{})", self.round, self.process_id)
    }
}

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub struct Command {
    pub client: ProcessId,
    pub req_id: Vec<u8>,
    pub operation: Vec<u8>,
}

impl Command {
    pub fn new_from_str(id: ProcessId, req_id: String, op: String) -> Command {
        Command {
            client: id,
            req_id: req_id.into_bytes(),
            operation: op.into_bytes(),
        }
    }

    pub fn req_id_str(&self) -> &str {
        match str::from_utf8(&self.req_id) {
            Ok(v) => v,
            Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
        }
    }

    pub fn op_str(&self) -> &str {
        match str::from_utf8(&self.operation) {
            Ok(v) => v,
            Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
        }
    }
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Command({},{},{})",
            self.client,
            str::from_utf8(&self.req_id).unwrap(),
            str::from_utf8(&self.operation).unwrap()
        )
    }
}

use std::{collections::HashMap, fmt::Display};

use protobuf::MessageField;

use crate::{
    proto::proto::{self},
    pval::PValue,
};

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

impl Into<proto::ProcessId> for ProcessId {
    fn into(self) -> proto::ProcessId {
        let mut def = proto::ProcessId::default();
        def.ip = match self.ip {
            std::net::IpAddr::V4(v4) => Option::Some(proto::process_id::Ip::V4(v4.into())),
            std::net::IpAddr::V6(_) => todo!("ip v6 is not supported"),
        };
        def.port = self.port;
        def.id = self.id;
        def
    }
}

impl From<proto::ProcessId> for ProcessId {
    fn from(value: proto::ProcessId) -> Self {
        Self {
            ip: match value.ip {
                Some(proto::process_id::Ip::V4(v)) => std::net::IpAddr::V4(v.into()),
                _ => unreachable!("ip v6 is not supported"),
            },
            port: value.port,
            id: value.id,
        }
    }
}

impl Into<proto::BallotNumber> for BallotNumber {
    fn into(self) -> proto::BallotNumber {
        let mut def = proto::BallotNumber::default();
        def.round = self.round;
        def.process_id = MessageField::some(self.process_id.into());
        def
    }
}

impl From<proto::BallotNumber> for BallotNumber {
    fn from(value: proto::BallotNumber) -> Self {
        Self {
            round: value.round,
            process_id: ProcessId::from(value.process_id.unwrap()),
        }
    }
}

impl Into<proto::Command> for Command {
    fn into(self) -> proto::Command {
        let mut def = proto::Command::default();
        def.client = MessageField::some(self.client.into());
        def.req_id = self.req_id.clone().into();
        def.operation = self.operation.clone().into();
        def
    }
}

impl From<proto::Command> for Command {
    fn from(value: proto::Command) -> Self {
        Command {
            client: value.client.unwrap().into(),
            req_id: value.req_id.into(),
            operation: value.operation.into(),
        }
    }
}

impl Into<proto::PValue> for PValue {
    fn into(self) -> proto::PValue {
        let mut def = proto::PValue::default();
        def.ballot = MessageField::some(self.ballot.into());
        def.slot = self.slot;
        def.command = MessageField::some(self.command.into());
        def
    }
}

impl From<proto::PValue> for PValue {
    fn from(value: proto::PValue) -> Self {
        Self {
            ballot: value.ballot.unwrap().into(),
            slot: value.slot,
            command: value.command.unwrap().into(),
        }
    }
}

impl Into<HashMap<u64, proto::PValue>> for Accepted {
    fn into(self) -> HashMap<u64, proto::PValue> {
        self.map(|v| v.into())
    }
}

impl From<HashMap<u64, proto::PValue>> for Accepted {
    fn from(value: HashMap<u64, proto::PValue>) -> Self {
        let mut res = Self::new();
        for (k, v) in value.into_iter() {
            res.insert(k, v.into());
        }
        res
    }
}

impl Into<proto::Message> for Message {
    fn into(self) -> proto::Message {
        return match self {
            Message::P1A(id, ballot) => {
                let mut def = proto::Message::default();
                def.type_ = proto::MessageType::P1A.into();
                def.process = MessageField::some(id.into());
                def.ballot = MessageField::some(ballot.into());
                def
            }
            Message::P1B(id, ballot, accepted) => {
                let mut def = proto::Message::default();
                def.type_ = proto::MessageType::P1B.into();
                def.process = MessageField::some(id.into());
                def.ballot = MessageField::some(ballot.into());
                def.accepted = accepted.into();
                def
            }
            Message::P2A(id, ballot, slot, command) => {
                let mut def = proto::Message::default();
                def.type_ = proto::MessageType::P2A.into();
                def.process = MessageField::some(id.into());
                def.ballot = MessageField::some(ballot.into());
                def.slot = Option::Some(slot);
                def.command = MessageField::some(command.into());
                def
            }
            Message::P2B(id, ballot, slot) => {
                let mut def = proto::Message::default();
                def.type_ = proto::MessageType::P2B.into();
                def.process = MessageField::some(id.into());
                def.ballot = MessageField::some(ballot.into());
                def.slot = Option::Some(slot);
                def
            }
            Message::Preempt(id, ballot) => {
                let mut def = proto::Message::default();
                def.type_ = proto::MessageType::Preempt.into();
                def.process = MessageField::some(id.into());
                def.ballot = MessageField::some(ballot.into());
                def
            }
            Message::Adopt(id, ballot, accepted) => {
                let mut def = proto::Message::default();
                def.type_ = proto::MessageType::Adopt.into();
                def.process = MessageField::some(id.into());
                def.ballot = MessageField::some(ballot.into());
                def.accepted = accepted.into();
                def
            }
            Message::Decision(id, slot, command) => {
                let mut def = proto::Message::default();
                def.type_ = proto::MessageType::Decision.into();
                def.process = MessageField::some(id.into());
                def.slot = Option::Some(slot);
                def.command = MessageField::some(command.into());
                def
            }
            Message::Request(id, command) => {
                let mut def = proto::Message::default();
                def.type_ = proto::MessageType::Request.into();
                def.process = MessageField::some(id.into());
                def.command = MessageField::some(command.into());
                def
            }
            Message::Propose(id, slot, command) => {
                let mut def = proto::Message::default();
                def.type_ = proto::MessageType::Propose.into();
                def.process = MessageField::some(id.into());
                def.slot = Option::Some(slot);
                def.command = MessageField::some(command.into());
                def
            }
        };
    }
}

impl From<proto::Message> for Message {
    fn from(value: proto::Message) -> Self {
        match value.type_.enum_value() {
            Ok(t) => match t {
                proto::MessageType::P1A => {
                    Message::P1A(value.process.unwrap().into(), value.ballot.unwrap().into())
                }
                proto::MessageType::P1B => Message::P1B(
                    value.process.unwrap().into(),
                    value.ballot.unwrap().into(),
                    value.accepted.into(),
                ),
                proto::MessageType::P2A => Message::P2A(
                    value.process.unwrap().into(),
                    value.ballot.unwrap().into(),
                    value.slot.unwrap(),
                    value.command.unwrap().into(),
                ),
                proto::MessageType::P2B => Message::P2B(
                    value.process.unwrap().into(),
                    value.ballot.unwrap().into(),
                    value.slot.unwrap(),
                ),
                proto::MessageType::Preempt => {
                    Message::Preempt(value.process.unwrap().into(), value.ballot.unwrap().into())
                }
                proto::MessageType::Adopt => Message::Adopt(
                    value.process.unwrap().into(),
                    value.ballot.unwrap().into(),
                    value.accepted.into(),
                ),
                proto::MessageType::Decision => Message::Decision(
                    value.process.unwrap().into(),
                    value.slot.unwrap(),
                    value.command.unwrap().into(),
                ),
                proto::MessageType::Request => {
                    Message::Request(value.process.unwrap().into(), value.command.unwrap().into())
                }
                proto::MessageType::Propose => Message::Propose(
                    value.process.unwrap().into(),
                    value.slot.unwrap(),
                    value.command.unwrap().into(),
                ),
            },
            Err(_) => unreachable!("should always be present"),
        }
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

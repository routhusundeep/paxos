use std::{fmt::format, thread, time::Duration, vec};

use crate::paxos::{
    acceptor::Acceptor,
    env::{Env, ProcessId, Sender},
    leader::Leader,
    local::{Channel, InMemEnv},
    message::Message,
    pval::Command,
    replica::Replica,
};

mod paxos;
use log::{Level, Metadata, Record};
use log::{LevelFilter, SetLoggerError};

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

static LOGGER: SimpleLogger = SimpleLogger;

pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Trace))
}

fn main() {
    init();
    let n_acceptors = 3;
    let n_replicas = 2;
    let n_leaders = 2;
    let n_requests = 10;

    let mut env = InMemEnv::new(Channel::new_default);

    for i in 1..n_acceptors + 1 {
        let id = ProcessId::new(format!("Acceptor:{}", i));
        env.register(
            id.clone(),
            paxos::env::ProcessType::Acceptor,
            Acceptor::new(id.clone()),
        );
    }

    for i in 1..n_leaders + 1 {
        let id = ProcessId::new(format!("Leader:{}", i));
        env.register(
            id.clone(),
            paxos::env::ProcessType::Leader,
            Leader::new(id.clone()),
        );
    }

    for i in 1..n_replicas + 1 {
        let id = ProcessId::new(format!("Replica:{}", i));
        env.register(
            id.clone(),
            paxos::env::ProcessType::Replica,
            Replica::new(id.clone()),
        );
    }

    for i in 1..n_requests + 1 {
        let s = env.sender();
        let client = ProcessId::new(format!("Client:{}", i));

        for j in 1..n_replicas + 1 {
            let id = ProcessId::new(format!("Replica:{}", j));
            s.send(
                &id,
                &Message::Request(
                    client.clone(),
                    Command::new_from_str(
                        client.clone(),
                        format!("Request:{}", i),
                        format!("Op:{}", i),
                    ),
                ),
            )
        }
    }

    thread::sleep(Duration::from_secs(10));
}

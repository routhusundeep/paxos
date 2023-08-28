use std::{thread, time::Duration};

use crate::paxos::{
    acceptor::Acceptor,
    env::{Env, ProcessId, Router},
    leader::Leader,
    local::InMemEnv,
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
    init().unwrap();
    let n_acceptors = 3;
    let n_replicas = 2;
    let n_leaders = 2;
    let n_requests = 10;

    let mut env = InMemEnv::new(|| {
        let (s, r) = crossbeam::channel::unbounded();
        return (r, s);
    });

    for i in 1..n_acceptors + 1 {
        let id = ProcessId::new();
        env.register(
            id.clone(),
            paxos::env::ProcessType::Acceptor,
            Acceptor::new(id.clone()),
        );
    }

    for i in 1..n_leaders + 1 {
        let id = ProcessId::new();
        env.register(
            id.clone(),
            paxos::env::ProcessType::Leader,
            Leader::new(id.clone()),
        );
    }

    for i in 1..n_replicas + 1 {
        let id = ProcessId::new();
        env.register(
            id.clone(),
            paxos::env::ProcessType::Replica,
            Replica::new(id.clone()),
        );
    }

    for i in 1..n_requests + 1 {
        let s = env.router();
        let client = ProcessId::new();

        for j in env.cluster().replicas().iter() {
            s.send(
                j,
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

mod logger {
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
}

mod tests {
    use crate::{
        acceptor::Acceptor,
        env::{Env, ProcessId, Router},
        leader::Leader,
        local::InMemEnv,
        message::Message,
        pval::Command,
        replica::Replica,
        zmq::ZMQEnv,
    };
    use once_cell::sync::Lazy;
    use std::{
        net::{IpAddr, Ipv4Addr},
        thread,
        time::Duration,
    };

    // static ENV: Lazy<
    //     InMemEnv<crossbeam::channel::Receiver<Message>, crossbeam::channel::Sender<Message>>,
    // > = Lazy::new(|| {
    //     InMemEnv::new(|| {
    //         let (s, r) = crossbeam::channel::unbounded();
    //         return (r, s);
    //     })
    // });

    static ENV: Lazy<
        ZMQEnv<crossbeam::channel::Receiver<Message>, crossbeam::channel::Sender<Message>>,
    > = Lazy::new(|| {
        ZMQEnv::new(|| {
            let (s, r) = crossbeam::channel::unbounded();
            return (r, s);
        })
    });

    #[test]
    fn test() {
        super::logger::init().unwrap();

        let n_acceptors = 3;
        let n_replicas = 2;
        let n_leaders = 2;
        let n_requests = 100;

        let local_host = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let port = 6060;
        let id = ProcessId::new(local_host, port, ENV.new_id());

        thread::spawn(move || {
            let addr = id.addr_sender();
            ENV.poller.start(&addr);
        });

        thread::sleep(Duration::from_millis(100));

        for _ in 1..n_acceptors + 1 {
            let id = ProcessId::new(local_host, port, ENV.new_id());
            ENV.register(
                id.clone(),
                crate::env::ProcessType::Acceptor,
                Acceptor::new(id.clone()),
            );
        }

        for _ in 1..n_leaders + 1 {
            let id = ProcessId::new(local_host, port, ENV.new_id());
            ENV.register(
                id.clone(),
                crate::env::ProcessType::Leader,
                Leader::new(id.clone()),
            );
        }

        for _ in 1..n_replicas + 1 {
            let id = ProcessId::new(local_host, port, ENV.new_id());
            ENV.register(
                id.clone(),
                crate::env::ProcessType::Replica,
                Replica::new(id.clone()),
            );
        }

        for i in 1..n_requests + 1 {
            let s = ENV.router();
            let client = ProcessId::new(local_host, port, ENV.new_id());

            for j in ENV.cluster().replicas().iter() {
                s.send(
                    j,
                    Message::Request(
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

        thread::sleep(Duration::from_secs(10000));
    }
}

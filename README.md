# paxos

Implement multi paxos in Rust . It is just a prototype intending to learn and internalize the algorithm, not meant to substitute perfectly competent libraries out there. This [paper](https://www.cs.cornell.edu/home/rvr/Paxos/paxos.pdf) is used heavily for inspiration and is the primary source.

## Features:
### Basic algorithm
Fully functional multi paxos implementation.

### State reduction in acceptor
Acceptor state do not grow with number of messages. will only store the latest accepted PValue for each slot.

### Decision tracking in leader from colocated nodes
Leader tracks the decided commands, it reduces the number of proposals retries vastly.

### Support network
Happens through a combination of in memory queues and sockets. [ZMQ](https://zeromq.org/get-started/) is used for the socket communication with protobuf for the serde. TCP is the only used protocol, can use multicast if needed. 

## Planned: 
### Failure detection
Hearbeat can be implemented, but it will take some time to simulate failures for testing.

### Garbage collection on Acceptor
Should be easy to add by introducing a new message type which will be sent by the replicas when they apply commands. Leader can keep track of it and eventually send it to acceptor once done.

### Leases for leader
Probably requires the biggest change. So will postpone it until the rest of the features are added.

References:
* [Paxos made simple](https://github.com/papers-we-love/papers-we-love/blob/main/distributed_systems/paxos-made-simple.pdf)
* [Understanding Paxos](https://understandingpaxos.wordpress.com/)
* [Paxos made moderately complex](https://www.cs.cornell.edu/home/rvr/Paxos/paxos.pdf)

# paxos

Implement multi paxos in Rust . It is just a prototype intending to learn and internalize the algorithm, not meant to substitute perfectly competent libraries out there. This [paper](https://www.cs.cornell.edu/home/rvr/Paxos/paxos.pdf) is used heavily for inspiration and is the primary source.

I need to figure a way to test it.

## Features:
### basic algorithm
Fully functional multi paxos implementation.

### state reduction in acceptor
Acceptor state do not grow exponentially with number of messages. will only store the latest accepted PValue for each slot.

## Planned: 
### support on TCP network
Currently message communication happens through in-memory channels, will need to extend it to network. 

### failure detection
Will go with a basic heartbeats for now, maybe accrual detection if I feel enterprising.

### Use colocation
The cluster should have the location added to it, should use a combination of TCP and Channel for message delivery. 

### garbage collection on leader
should be easy to add by introducing a new message type which will be sent by the replicas when they apply a command.

### Support generic state machine at replica
Right now, the replica is just a log. refactoring it into generic state machine would be helpful.

### leases for leader
Probably requires the biggest change. So will postpone it until the rest of the features are added.

References:
* [Paxos made simple](https://github.com/papers-we-love/papers-we-love/blob/main/distributed_systems/paxos-made-simple.pdf)
* [Understanding Paxos](https://understandingpaxos.wordpress.com/)
* [Paxos made moderately complex](https://www.cs.cornell.edu/home/rvr/Paxos/paxos.pdf)

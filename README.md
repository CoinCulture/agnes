# Agnes

Agnes is a BFT state-machine replication engine written in Rust.
It uses the Tendermint consensus algorithm.

## Code Structure

The code is designed to be highly modular. Each module has a minimal number of
concerns. An explicit goal is for algorithms and data structures to be as
decoupled as possible from the structure of data sent on the wire to peers.
This should greatly facilitate testing by reducing the size of the object
graphs necessary to test core componentry. For instance, signature
validation and proposer selection should not be required for testing 
the logic of the consensus state machine.

## Consensus State Machine

The Consensus State Machine is an implementation of the Tendermint algorithm as specified in the
[paper](https://arxiv.org/pdf/1807.04938.pdf). The state machine structure was inspired by
the blog post, [Pretty State Machine Patterns in Rust](https://hoverbear.org/2016/10/12/rust-state-machine-pattern/) 
and a [variation derived from it](https://www.reddit.com/r/rust/comments/57ccds/pretty_state_machine_patterns_in_rust/d8rhwq4/).
It can be seen as a Rust implemention of [ADR-30 from the Tendermint Core
project](https://github.com/tendermint/tendermint/pull/2696).

There are three basic types: 

- State: the internal consensus state
- Event: a consensus event, like receiving a polka or a timeout
- Message: outputs from a transition, like a vote to send to peers, or a timeout
  to schedule

`enum` and `match` are used to specify which events can be applied to which
states. When an event is applied to the state, the state is updated and/or a message
is returned.

As much of the state machine as possible is captured in the type system.
Rather than include functions for things like checking the proposer and the
validity of proposed values, distinct events are used to distinguish when the given
consensus instance is the proposer and when the proposed value is valid or not. 
This implies that the consumer of the state machine performs these checks and passes
the appropriate events.

The height of the state machine does not change.
Once a decision is output, the consumer can initialize a new State at a new height.

The consumer is expected to handle all aspects of sending
and receiving messages to peers and determining when a set of received messages
(including those returned by the state machine) constitutes an event.
It must also managed the scheduling and receipt of timeouts.


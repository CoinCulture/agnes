# Tendermint Consensus State Machine

This is an implementation of the Tendermint Consensus State Machine as specified in the
[paper](https://arxiv.org/pdf/1807.04938.pdf).

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


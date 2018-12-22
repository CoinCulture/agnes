# Tendermint Consensus State Machine

Here we implement the Tendermint Consensus State Machine as specified in the
[paper](https://arxiv.org/pdf/1807.04938.pdf).

We have three basic types: 

- State: the internal consensus state
- Event: a consensus event, like receiving a polka or a timeout
- Message: outputs from a transition, like a vote to send to peers, or a timeout
  to schedule

We use `enum` and `match` to specify which events can be applied to which
states. When an event is applied to the state, the state is updated and/or a message
is returned.

We capture as much as possible in the type system of the state machine.
Rather than include functions for things like checking the proposer and the
validity of proposed values, we use distinct events for when we are the proposer
and for when a proposed value is valid or not. This implies that the consumer of
this state machine is performing these checks and passing the appropriate
events.

We only operate on one height at a time. Once a decision is output, the consumer
can initialize a new height.

The consumer is expected to handle all aspects of sending
and receiving messages to peers and determining when a set of received messages
(including those returned by the state machine) constitutes an event.
It must also managed the scheduling and receipt of timeouts.


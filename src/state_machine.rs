//---------------------------------------------------------------------
// State

// Value is what we want to agree on.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Value {}

// RoundValue contains a Value and associated Round.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct RoundValue {
    round: i64,
    value: Value,
}

// Step is the step of the consensus in the round.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Step {
    NewRound,
    Propose,
    Prevote,
    Precommit,
    Commit,
}

// State is the state of the consensus state machine.
#[derive(Copy, Clone, Debug)]
pub struct State {
    height: i64,
    round: i64,
    step: Step,
    locked: Option<RoundValue>,
    valid: Option<RoundValue>,
}

impl State {
    // new creates a new State at the given height.
    pub fn new(height: i64) -> State {
        State {
            height: height,
            round: 0,
            step: Step::NewRound,
            locked: None,
            valid: None,
        }
    }

    // set_round sets the State to step NewRound at the given round.
    fn set_round(self, round: i64) -> State {
        State {
            round: round,
            step: Step::NewRound,
            ..self
        }
    }

    // next_step progresses the State to the next step,
    // stopping at precommit. To progress to Commit,
    // call commit_step(); to reset to NewRound, call
    // set_round().
    fn next_step(self) -> State {
        let step = match self.step {
            Step::NewRound => Step::Propose,
            Step::Propose => Step::Prevote,
            Step::Prevote => Step::Precommit,
            _ => self.step,
        };
        State { step: step, ..self }
    }

    // commit_step sets State to the Commit step.
    // No more state transitions can take place.
    fn commit_step(self) -> State {
        State {
            step: Step::Commit,
            ..self
        }
    }

    // set_locked sets the locked value and round.
    fn set_locked(self, value: Value) -> State {
        let round = self.round;
        let locked = Some(RoundValue { round, value });
        State { locked, ..self }
    }

    // set_valid sets the valid value and round.
    fn set_valid(self, value: Value) -> State {
        let round = self.round;
        let valid = Some(RoundValue { round, value });
        State { valid, ..self }
    }
}

//---------------------------------------------------------------------
// Inputs (Events)

// RoundEvent contains an event and its round.
// When applied successfully, it causes a state transition
// to occur or a message to be returned.
pub struct RoundEvent {
    round: i64,
    event: Event,
}

// Event is a type of event. It carries any relevant data.
pub enum Event {
    NewRound,                // Start a new round, not as proposer.
    NewRoundProposer(Value), // Start a new round and propose the Value.
    Proposal(i64, Value),    // Receive a proposal with possible pol_round.
    ProposalInvalid,         // Receive an invalid proposal.
    PolkaAny,                // Receive +2/3 prevotes for anything.
    PolkaNil,                // Receive +2/3 prevotes for nil.
    PolkaValue(Value),       // Receive +2/3 prevotes for Value.
    PrecommitAny,            // Receive +2/3 precommits for anything.
    PrecommitValue(Value),   // Receive +2/3 precommits for Value.
    RoundSkip,               // Receive +1/3 votes from a higher round.
    TimeoutPropose,          // Timeout waiting for proposal.
    TimeoutPrevote,          // Timeout waiting for prevotes.
    TimeoutPrecommit,        // Timeout waiting for precommits.
}

//---------------------------------------------------------------------
// Outputs (Messages)

// Message is the output of the state machine - proposals/votes
// to send to peers, timeouts to schedule, and an ultimate decision value.
#[derive(Debug, PartialEq)]
pub enum Message {
    NewRound(i64),        // Move to the new round.
    Proposal(Proposal),   // Broadcast the proposal.
    Prevote(Vote),        // Broadcast the prevote.
    Precommit(Vote),      // Broadcast the precommit.
    Timeout(Timeout),     // Schedule the timeout.
    Decision(RoundValue), // Decide the value.
}

impl Message {
    fn proposal(round: i64, value: Value, pol_round: i64) -> Message {
        Message::Proposal(Proposal {
            round,
            value,
            pol_round,
        })
    }
    fn prevote(round: i64, value: Option<Value>) -> Message {
        Message::Prevote(Vote { round, value })
    }
    fn precommit(round: i64, value: Option<Value>) -> Message {
        Message::Precommit(Vote { round, value })
    }
    fn timeout(round: i64, step: Step) -> Message {
        Message::Timeout(Timeout { round, step })
    }
    fn decision(round: i64, value: Value) -> Message {
        Message::Decision(RoundValue { round, value })
    }
}

// Proposal proposes a value in a round.
// pol_round is -1 or the last round this value got a polka.
#[derive(Debug, PartialEq)]
pub struct Proposal {
    round: i64,
    value: Value,
    pol_round: i64,
}

// Vote is a vote for a value in a round.
#[derive(Debug, PartialEq)]
pub struct Vote {
    round: i64,
    value: Option<Value>,
}

// Timeout is used to schedule timeouts at different steps in the round.
#[derive(Debug, PartialEq)]
pub struct Timeout {
    round: i64,
    step: Step,
}

//---------------------------------------------------------------------
// State Transition Function

impl State {
    // convenience fn to check if a proposal's pol_round is valid
    fn valid_vr(self, vr: i64) -> bool {
        vr >= -1 && vr < self.round
    }
}

// next transitions the state machine. It takes a state and an input event
// and returns an updated state and output message.
// Valid transitions result in at least a change to the state and/or an output message.
// Commented numbers refer to line numbers in the spec paper.
pub fn next(s: State, event: RoundEvent) -> (State, Option<Message>) {
    let eqr = s.round == event.round;
    match (s.step, event.event) {
        // From NewRound. Event must be for current round.
        (Step::NewRound, Event::NewRoundProposer(v)) if eqr => propose(s, v), // 11/14
        (Step::NewRound, Event::NewRound) if eqr => schedule_timeout_propose(s), // 11/20

        // From Propose. Event must be for current round.
        (Step::Propose, Event::Proposal(vr, v)) if eqr && s.valid_vr(vr) => prevote(s, vr, v), // 22, 28
        (Step::Propose, Event::ProposalInvalid) if eqr => prevote_nil(s), // 22/25, 28/31
        (Step::Propose, Event::TimeoutPropose) if eqr => prevote_nil(s),  // 57

        // From Prevote. Event must be for current round.
        (Step::Prevote, Event::PolkaAny) if eqr => schedule_timeout_prevote(s), // 34
        (Step::Prevote, Event::PolkaNil) if eqr => precommit_nil(s),            // 44
        (Step::Prevote, Event::PolkaValue(v)) if eqr => precommit(s, v), // 36/37 - NOTE: only once?
        (Step::Prevote, Event::TimeoutPrevote) if eqr => precommit_nil(s), // 61

        // From Precommit. Event must be for current round.
        (Step::Precommit, Event::PolkaValue(v)) if eqr => set_valid_value(s, v), // 36/42 - NOTE: only once?

        // From Commit. No more state transitions.
        (Step::Commit, _) => (s, None),

        // From all (except Commit). Various round guards.
        (_, Event::PrecommitAny) if eqr => schedule_timeout_precommit(s), // 47
        (_, Event::TimeoutPrecommit) if eqr => round_skip(s, event.round + 1), // 65
        (_, Event::RoundSkip) if s.round < event.round => round_skip(s, event.round), // 55
        (_, Event::PrecommitValue(v)) => commit(s, event.round, v),       // 49
        _ => (s, None),
    }
}

//---------------------------------------------------------------------
// Propose

// We're the proposer - propose the valid value if it exists,
// otherwise propose the given value.
// 11/14
fn propose(s: State, v: Value) -> (State, Option<Message>) {
    let s = s.next_step();
    let (value, pol_round) = match s.valid {
        Some(v) => (v.value, v.round),
        None => (v, -1),
    };
    (s, Some(Message::proposal(s.round, value, pol_round)))
}

//---------------------------------------------------------------------
// Prevote

// Received a complete proposal - prevote the value,
// unless we're locked on something else at a higher round.
// 22, 28
fn prevote(s: State, vr: i64, proposed: Value) -> (State, Option<Message>) {
    let s = s.next_step();
    let value = match s.locked {
        Some(locked) if locked.round <= vr => Some(proposed), // unlock and prevote
        Some(locked) if locked.value == proposed => Some(proposed), // already locked on value
        Some(_) => None, // we're locked on a higher round with a different value, prevote nil
        None => Some(proposed), // not locked, prevote the value
    };
    (s, Some(Message::prevote(s.round, value)))
}

// Received a complete proposal for an empty or invalid value, or timed out - prevote nil.
// 22/25, 28/31, 57
fn prevote_nil(s: State) -> (State, Option<Message>) {
    let s = s.next_step();
    (s, Some(Message::prevote(s.round, None)))
}

//---------------------------------------------------------------------
// Precommit

// Received a polka for a value - precommit the value.
// 36
// NOTE: only one of this and set_valid_value should be called once in a round
fn precommit(s: State, v: Value) -> (State, Option<Message>) {
    let s = s.set_locked(v).set_valid(v).next_step();
    (s, Some(Message::precommit(s.round, Some(v))))
}

// Received a polka for nil or timed out of prevote - precommit nil.
// 44, 61
fn precommit_nil(s: State) -> (State, Option<Message>) {
    let s = s.next_step();
    (s, Some(Message::precommit(s.round, None)))
}

//---------------------------------------------------------------------
// Schedule timeouts

// We're not the proposer - schedule timeout propose.
// 11/20
fn schedule_timeout_propose(s: State) -> (State, Option<Message>) {
    let s = s.next_step();
    (s, Some(Message::timeout(s.round, Step::Propose)))
}

// We received a polka for any - schedule timeout prevote.
// 34
// NOTE: this should only be called once in a round, per the spec,
// but it's harmless to schedule more timeouts
fn schedule_timeout_prevote(s: State) -> (State, Option<Message>) {
    (s, Some(Message::timeout(s.round, Step::Prevote)))
}

// We received +2/3 precommits for any - schedule timeout precommit.
// 47
fn schedule_timeout_precommit(s: State) -> (State, Option<Message>) {
    (s, Some(Message::timeout(s.round, Step::Precommit)))
}

//---------------------------------------------------------------------
// Set the valid value.

// We received a polka for a value after we already precommited.
// Set the valid value and current round.
// 36/42
// NOTE: only one of this and precommit should be called once in a round
fn set_valid_value(s: State, v: Value) -> (State, Option<Message>) {
    (s.set_valid(v), None)
}

//---------------------------------------------------------------------
// New round or height

// We finished a round (timeout precommit) or received +1/3 votes
// from a higher round. Move to the higher round.
// 65
fn round_skip(s: State, r: i64) -> (State, Option<Message>) {
    (s.set_round(r), Some(Message::NewRound(r)))
}

// We received +2/3 precommits for a value - commit and decide that value!
// 49
fn commit(s: State, r: i64, v: Value) -> (State, Option<Message>) {
    (s.commit_step(), Some(Message::decision(r, v)))
}

//---------------------------------------------------------------------
// Test

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn happy_case() {
        let val = Value {};
        let v = Some(val);
        let s = State::new(1);
        let (s, m) = next(s, RoundEvent {
            round: 0,
            event: Event::NewRoundProposer(val),
        });
        assert_eq!(m.unwrap(), Message::proposal(0, val, -1));
        let (s, m) = next(s, RoundEvent {
            round: 0,
            event: Event::Proposal(-1, val),
        });
        assert_eq!(m.unwrap(), Message::prevote(0, v));
        let (s, m) = next(s, RoundEvent {
            round: 0,
            event: Event::PolkaValue(val),
        });
        assert_eq!(m.unwrap(), Message::precommit(0, v));
        let (s, m) = next(s, RoundEvent {
            round: 0,
            event: Event::PrecommitValue(val),
        });
        assert_eq!(m.unwrap(), Message::decision(0, val));

        assert_eq!(s.step, Step::Commit);
    }
}

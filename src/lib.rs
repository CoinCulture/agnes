// Value is what we want to agree on.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Value {}

// RoundValue contains a Value and associated Round.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct RoundValue {
    round: i64,
    value: Value,
}

impl RoundValue {
    pub fn new(round: i64, value: Value) -> RoundValue {
        RoundValue { round, value }
    }
}

// State is the state of the consensus state machine.
#[derive(Copy, Clone, Debug)]
pub struct State {
    height: i64,
    round: i64,
    step: RoundStep,
    locked: Option<RoundValue>,
    valid: Option<RoundValue>,
}

impl State {
    fn set_round(self, round: i64) -> State {
        State {
            round: round,
            ..self
        }
    }

    fn set_step(self, step: RoundStep) -> State {
        State { step: step, ..self }
    }

    fn set_locked(self, locked: Value) -> State {
        State {
            locked: Some(RoundValue::new(self.round, locked)),
            ..self
        }
    }

    fn set_valid(self, valid: Value) -> State {
        State {
            valid: Some(RoundValue::new(self.round, valid)),
            ..self
        }
    }
}

// RoundStep is the step of the consensus in the round.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum RoundStep {
    NewRound,
    Propose,
    Prevote,
    Precommit,
    Commit,
}

// Event causes a state transition or message.
pub struct Event {
    round: i64,
    typ: EventType,
}

// EventType is a type of event, with any additional data
pub enum EventType {
    NewRound,                  // Start a new round, not as proposer
    NewRoundProposer(Value),   // Start a new round and propose the Value
    Proposal(Value),           // Receive a proposal
    ProposalInvalid,           // Receive an invalid proposal
    ProposalPolka(i64, Value), // Receive a proposal with a polka
    PolkaAny,                  // Receive +2/3 prevotes for anything
    PolkaNil,                  // Receive +2/3 prevotes for nil
    PolkaValue(Value),         // Receive +2/3 prevotes for Value
    PrecommitAny,              // Receive +2/3 precommits for anything
    PrecommitValue(Value),     // Receive +2/3 precommits for Value
    RoundSkip,                 // Receive +1/3 votes from a higher round
    TimeoutPropose,            // Timeout waiting for proposal
    TimeoutPrevote,            // Timeout waiting for prevotes
    TimeoutPrecommit,          // Timeout waiting for precommits
}

// Message is the output of the state machine - proposals/votes
// to send to peers, timeouts to schedule, and an ultimate decision value.
#[derive(Debug, PartialEq)]
pub enum Message {
    NewRound(i64),
    Proposal(Proposal),
    Prevote(Vote),
    Precommit(Vote),
    Timeout(Timeout),
    Decision(RoundValue),
}

// Proposal proposes a value in a round.
// pol_round is -1 or the last round this value got a polka.
#[derive(Debug, PartialEq)]
pub struct Proposal {
    round: i64,
    value: Value,
    pol_round: i64,
}

impl Proposal {
    fn new(round: i64, value: Value, pol_round: i64) -> Proposal {
        Proposal {
            round: round,
            value: value,
            pol_round: pol_round,
        }
    }
}

// Vote is a vote for a value in a round.
#[derive(Debug, PartialEq)]
pub struct Vote {
    round: i64,
    value: Option<Value>,
}

impl Vote {
    fn new(round: i64, value: Option<Value>) -> Vote {
        Vote {
            round: round,
            value: value,
        }
    }
}

// Timeout is used to schedule timeouts at different steps in the round.
#[derive(Debug, PartialEq)]
pub struct Timeout {
    round: i64,
    step: RoundStep,
}

impl Timeout {
    fn new(round: i64, step: RoundStep) -> Timeout {
        Timeout {
            round: round,
            step: step,
        }
    }
}

impl State {
    // new creates a new State at the given height.
    pub fn new(height: i64) -> State {
        State {
            height: height,
            round: 0,
            step: RoundStep::NewRound,
            locked: None,
            valid: None,
        }
    }

    // next progresses the state machine. It returns an updated State
    // and an optional message.
    pub fn next(self, event: Event) -> (State, Option<Message>) {
        let (s, round, eround) = (self, self.round, event.round);
        let eqr = round == eround;
        match (s.step, event.typ) {
            // no round guards
            (RoundStep::NewRound, EventType::NewRoundProposer(v)) => propose(s, eround, v), // 11/14
            (RoundStep::NewRound, EventType::NewRound) => schedule_timeout_propose(s, eround), // 11/20

            // must equal current round
            (RoundStep::Propose, EventType::Proposal(v)) if eqr => prevote(s, v), // 22
            (RoundStep::Propose, EventType::ProposalInvalid) if eqr => prevote_nil(s), // 22/25, 28/31
            (RoundStep::Propose, EventType::ProposalPolka(vr, v)) if eqr => prevote_polka(s, vr, v), // 28
            (RoundStep::Propose, EventType::TimeoutPropose) if eqr => prevote_nil(s), // 57
            (RoundStep::Prevote, EventType::PolkaAny) if eqr => schedule_timeout_prevote(s), // 34
            (RoundStep::Prevote, EventType::PolkaNil) if eqr => precommit_nil(s),     // 44
            (RoundStep::Prevote, EventType::PolkaValue(v)) if eqr => precommit(s, v), // 36/37 - NOTE: only once?
            (RoundStep::Prevote, EventType::TimeoutPrevote) if eqr => precommit_nil(s), // 61
            (RoundStep::Precommit, EventType::PolkaValue(v)) if eqr => set_valid_value(s, v), // 36/42 - NOTE: only once?
            (_, EventType::PrecommitAny) if eqr => schedule_timeout_precommit(s),             // 47
            (_, EventType::TimeoutPrecommit) if eqr => round_skip(s, eround + 1),             // 65

            // must be from higher round
            (_, EventType::RoundSkip) if round < eround => round_skip(s, eround), // 55

            // no round guards
            (_, EventType::PrecommitValue(v)) => commit(s, eround, v), // 49
            _ => (s, None),
        }
    }
}

//---------------------------------------------------------------------
// propose

// we're the proposer. decide a propsal.
// 11/14
fn propose(s: State, r: i64, v: Value) -> (State, Option<Message>) {
    let s = s.set_round(r).set_step(RoundStep::Propose);
    let (value, round) = match s.valid {
        Some(v) => (v.value, v.round),
        None => (v, -1),
    };
    (s, Some(Message::Proposal(Proposal::new(r, value, round))))
}

//---------------------------------------------------------------------
// prevote

// received a complete proposal with new value - prevote
// 22
fn prevote(s: State, proposed: Value) -> (State, Option<Message>) {
    let s = s.set_step(RoundStep::Prevote);
    let value = match s.locked {
        Some(locked) if proposed != locked.value => None, // locked on something else
        _ => Some(proposed),
    };
    (s, Some(Message::Prevote(Vote::new(s.round, value))))
}

// received a complete proposal with old (polka) value - prevote
// 28
fn prevote_polka(s: State, vr: i64, proposed: Value) -> (State, Option<Message>) {
    let s = s.set_step(RoundStep::Prevote);
    let value = match s.locked {
        Some(locked) if locked.round <= vr => Some(proposed), // unlock and prevote
        Some(locked) if locked.value == proposed => Some(proposed), // already locked on value
        _ => None,                                            // otherwise, prevote nil
    };
    (s, Some(Message::Prevote(Vote::new(s.round, value))))
}

// received a complete proposal for an empty or invalid value, or timed out.
// 22, 57
fn prevote_nil(s: State) -> (State, Option<Message>) {
    let s = s.set_step(RoundStep::Prevote);
    (s, Some(Message::Prevote(Vote::new(s.round, None))))
}

//---------------------------------------------------------------------
// precommit

// 44, 61
fn precommit_nil(s: State) -> (State, Option<Message>) {
    let s = s.set_step(RoundStep::Precommit);
    (s, Some(Message::Precommit(Vote::new(s.round, None))))
}

// 36
// NOTE: only one of this and set_valid_value should be called once in a round
fn precommit(s: State, v: Value) -> (State, Option<Message>) {
    let s = s.set_locked(v).set_valid(v).set_step(RoundStep::Precommit);
    (s, Some(Message::Precommit(Vote::new(s.round, Some(v)))))
}

//---------------------------------------------------------------------
// schedule timeouts

// we're not the proposer. schedule timeout propose
// 11/20
fn schedule_timeout_propose(s: State, r: i64) -> (State, Option<Message>) {
    let s = s.set_round(r).set_step(RoundStep::Propose);
    (s, Some(Message::Timeout(Timeout::new(s.round, s.step))))
}

// 34
// NOTE: this should only be called once in a round, per the spec,
// but it's harmless to schedule more timeouts
fn schedule_timeout_prevote(s: State) -> (State, Option<Message>) {
    (
        s,
        Some(Message::Timeout(Timeout::new(s.round, RoundStep::Prevote))),
    )
}

// 47
fn schedule_timeout_precommit(s: State) -> (State, Option<Message>) {
    (
        s,
        Some(Message::Timeout(Timeout::new(
            s.round,
            RoundStep::Precommit,
        ))),
    )
}

//---------------------------------------------------------------------
// set the valid block

// 36/42
// NOTE: only one of this and precommit should be called once in a round
fn set_valid_value(s: State, v: Value) -> (State, Option<Message>) {
    let s = s.set_valid(v);
    (s, None)
}

//---------------------------------------------------------------------
// new round or height

// 65
fn round_skip(s: State, r: i64) -> (State, Option<Message>) {
    (s, Some(Message::NewRound(r)))
}

// 49
fn commit(s: State, r: i64, v: Value) -> (State, Option<Message>) {
    let s = s.set_step(RoundStep::Commit);
    (s, Some(Message::Decision(RoundValue::new(r, v))))
}

//---------------------------------------------------------------------
// test

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn happy_case() {
        let val = Value {};
        let v = Some(val);
        let s = State::new(1);
        let (s, m) = s.next(Event {
            round: 0,
            typ: EventType::NewRoundProposer(val),
        });
        assert_eq!(m.unwrap(), Message::Proposal(Proposal::new(0, val, -1)));
        let (s, m) = s.next(Event {
            round: 0,
            typ: EventType::Proposal(val),
        });
        assert_eq!(m.unwrap(), Message::Prevote(Vote::new(0, v)));
        let (s, m) = s.next(Event {
            round: 0,
            typ: EventType::PolkaValue(val),
        });
        assert_eq!(m.unwrap(), Message::Precommit(Vote::new(0, v)));
        let (s, m) = s.next(Event {
            round: 0,
            typ: EventType::PrecommitValue(val),
        });
        assert_eq!(m.unwrap(), Message::Decision(RoundValue::new(0, val)));

        assert_eq!(s.step, RoundStep::Commit);
    }
}

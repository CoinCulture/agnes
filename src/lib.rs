
// Value is the value the consensus seeks agreement on.
#[derive(Copy, Clone, PartialEq)]
struct Value{}

// RoundValue contains a Value and the round it was set.
#[derive(Copy, Clone, PartialEq)]
struct RoundValue{
    round: i64,
    value: Value
}

// State is the state of the consensus.
#[derive(Copy, Clone)]
struct State{
    height: i64,
    round: i64,
    step: RoundStep,
    locked: Option<RoundValue>,
    valid: Option<RoundValue>,
}

impl State{
    fn update_round(self, round: i64) -> State{
        State{
            round: round,
            ..self
        }
    }

    fn update_step(self, step: RoundStep) -> State{
        State{
            step: step,
            ..self
        }
    }
}

// ValueManager gets and validates values.
trait ValueManager{
    fn get_value(&self) -> Value;
    fn validate(&self, v: Value) -> bool;
}

// StateWrapper contains the State. It also contains a ValueManager 
// for proposing a new value and validating received values.
#[derive(Copy, Clone)]
struct StateWrapper<V>
    where V: ValueManager
{ 
    state: State,
    value_manager: V,
}

// RoundStep is the step of the consensus in the round.
#[derive(Copy, Clone)]
enum RoundStep {
    NewRound,
    Propose,
    Prevote,
    Precommit,
    Commit,
}

// Event causes a state transition.
enum Event {
    NewRound(i64),
    NewRoundProposer(i64),
    Proposal(i64, Value),
    ProposalPolka(i64, i64, Value),
    PolkaAny(i64),
    PolkaNil(i64),
    PolkaValue(i64, Value),
    CommitAny(i64),
    CommitNil(i64),
    CommitValue(i64, Value),
    CertValue(i64, Value),
    TimeoutPropose(i64),
    TimeoutPrevote(i64),
    TimeoutPrecommit(i64),
}

// Message is returned.
enum Message {
    Proposal(Proposal),
    Prevote(Vote),
    Precommit(Vote),
    Decision(Vote),
    Timeout(Timeout),
}

struct Proposal{
    round: i64,
    value: Value,
    pol_round: i64,
}

impl Proposal{
    fn new(round: i64, value: Value, pol_round: i64) -> Proposal{
        Proposal{
            round: round,
            value: value,
            pol_round: pol_round,
        }
    }
}

struct Vote{
    round: i64,
    value: Option<Value>,
}

impl Vote {
    fn new(round: i64, value: Option<Value>) -> Vote{
        Vote{
            round: round,
            value: value,
        }
    }
}

struct Timeout{
    round: i64,
    step: RoundStep,
}

impl Timeout{
    fn new(round: i64, step: RoundStep) -> Timeout{
        Timeout{
            round: round,
            step: step,
        }
    }

}

impl<V> StateWrapper<V>
    where V: ValueManager
{
    fn new(height: i64, value_manager: V) -> StateWrapper<V>{
        StateWrapper{
            state: State{
                height: height,
                round: 0,
                step: RoundStep::NewRound,
                locked: None,
                valid: None,
            },
            value_manager: value_manager,
        }
    }

    fn with_state(self, state: State) -> StateWrapper<V>{
        StateWrapper{
            state: state,
            ..self
        }
    }

    fn next(self, event: Event) -> (StateWrapper<V>, Option<Message>) {
        let s = self.state;
        let (s, m) = match (s.step, event) {
            (RoundStep::NewRound, Event::NewRoundProposer(r)) => {   self.handle_new_round_proposer(r) } // 11/14
            (RoundStep::NewRound, Event::NewRound(r)) => {   self.handle_new_round(r) } // 11/20
            (RoundStep::Propose, Event::Proposal(r, v)) => {   self.handle_proposal(r, v) } // 22
            (RoundStep::Propose, Event::ProposalPolka(r, vr, v)) => {  self.handle_proposal_polka(r, vr, v) } // 28
            (RoundStep::Propose, Event::TimeoutPropose(r)) => {  handle_timeout_propose(s, r) } // 57
            (RoundStep::Prevote, Event::PolkaAny(r)) => {  handle_polka_any(s, r) } // 34
            (RoundStep::Prevote, Event::PolkaNil(r)) => {  handle_polka_nil(s, r) } // 44
            (RoundStep::Prevote, Event::PolkaValue(r, v)) => {  handle_polka_value(s, r, v) } // 36
            (RoundStep::Prevote, Event::TimeoutPrevote(r)) => {  handle_timeout_prevote(s, r) } // 61
            (RoundStep::Precommit, Event::PolkaValue(r, v)) => {  handle_polka_value(s, r, v) } // 36/42
            (_,                    Event::CommitAny(r)) => {  handle_commit_any(s, r) } // 47
            (_,                    Event::CommitValue(r, v)) => {  handle_commit_value(s, r, v) } // 49
            (_,                    Event::CertValue(r, v)) => {  handle_cert_value(s, r, v) } // 55
            (_,                    Event::TimeoutPrecommit(r)) => {  handle_timeout_precommit(s, r) } // 65
            _ => { (s, None) }
        };
        (self.with_state(s), m)
    }

    // we're the proposer. decide a propsal.
    // 11/14
    fn handle_new_round_proposer(&self, r: i64) -> (State, Option<Message>) {
        let s = self.state.update_round(r).update_step(RoundStep::Propose);
        let (proposal_value, valid_round) = match s.valid {
            Some(v) => { (v.value, v.round) }
            None    => { (self.value_manager.get_value(), -1) }
        };
        (s, Some(Message::Proposal(Proposal::new(r, proposal_value, valid_round))))
    }


    // we're not the proposer. schedule timeout propose
    // 11/20
    fn handle_new_round(&self, r: i64) -> (State, Option<Message>) {
        let s = self.state.update_round(r).update_step(RoundStep::Propose);
        (s, Some(Message::Timeout(Timeout::new(s.round, s.step))))
    }

    // received a complete proposal with new value - prevote
    // 22
    fn handle_proposal(&self, r: i64, proposed_value: Value) -> (State, Option<Message>){
        let s = self.state.update_step(RoundStep::Prevote);
        let prevote_value = match self.value_manager.validate(proposed_value) {
            false => { None } // its not valid, prevote nil
            true => match s.locked {
                Some(v) if proposed_value != v.value => { None } // locked but the vals dont match, prevote nil
                _ => { Some(proposed_value) } // otherwise, prevote the value
            }
        };
        (s, Some(Message::Prevote(Vote::new(s.round, prevote_value))))
    }

    // received a complete proposal with old (polka) value - prevote
    // 28
    fn handle_proposal_polka(&self, r: i64, vr: i64, proposed_value: Value) -> (State, Option<Message>) {
        let s = self.state.update_step(RoundStep::Prevote);
        let prevote_value = match self.value_manager.validate(proposed_value) {
            false => { None } // its not valid, prevote nil
            true => match s.locked {
                Some(v) if v.round <= vr => { Some(proposed_value) } // unlock and prevote
                Some(v) if v.value == proposed_value  => { Some(proposed_value) } // already locked on value
                _ => { None } // otherwise, prevote nil
            }
        };
        (s, Some(Message::Prevote(Vote::new(s.round, prevote_value))))
    }
}

// timed out of propose - prevote nil
// 57
fn handle_timeout_propose(s: State, r: i64) -> (State, Option<Message>) {
    if s.round == r {
        let s = s.update_step(RoundStep::Prevote);
        return (s, Some(Message::Prevote(Vote::new(r, None))))
    }
    (s, None)
}

// 34
// TODO: this should only execute once per round
fn handle_polka_any(s: State, r: i64) -> (State, Option<Message>) {
    (s, Some(Message::Timeout(Timeout::new(r, RoundStep::Prevote))))
}

// 44
fn handle_polka_nil(s: State, r: i64) -> (State, Option<Message>) {
    (s, None)
}

// 36
fn handle_polka_value(s: State, r: i64, v: Value) -> (State, Option<Message>) {
    (s, None)
}

// 61
fn handle_timeout_prevote(s: State, r: i64) -> (State, Option<Message>) {
    (s, None)
}

// 47
fn handle_commit_any(s: State, r: i64) -> (State, Option<Message>) {
    (s, None)
}

// 49
fn handle_commit_value(s: State, r: i64, v: Value) -> (State, Option<Message>) {
    (s, None)
}

// 55
fn handle_cert_value(s: State, r: i64, v: Value) -> (State, Option<Message>) {
    (s, None)
}

// 65
fn handle_timeout_precommit(s: State, r: i64) -> (State, Option<Message>) {
    (s, None)
}




fn main() {
    println!("Hello, world!");
}

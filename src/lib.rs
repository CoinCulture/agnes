
// Value is what the consensus reaches agreement on.
#[derive(Copy, Clone, PartialEq)]
struct Value{
    value: i64,
}


// ValueManager gets and validates values.
trait ValueManager{
    fn get_value(&self) -> Value;
    fn validate(&self, v: Value) -> bool;
}

// State is the state of the consensus.
#[derive(Copy, Clone)]
struct State{
    height: i64,
    round: i64,
    step: RoundStep,
    locked_value: Option<Value>,
    locked_round: i64,
    valid_value: Option<Value>,
    valid_round: i64,
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
    NewHeight(i64),
    NewRound(i64, i64),
    NewRoundProposer(i64, i64),
    Proposal(i64, i64, Value),
    ProposalPolka(i64, i64, i64, Value),
    PolkaAny(i64, i64),
    PolkaNil(i64, i64),
    PolkaValue(i64, i64, Value),
    CommitAny(i64, i64),
    CommitNil(i64, i64),
    CommitValue(i64, i64, Value),
    CertValue(i64, i64, Value),
    TimeoutPropose(i64, i64),
    TimeoutPrevote(i64, i64),
    TimeoutPrecommit(i64, i64),
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
    height: i64,
    round: i64,
    value: Value,
    pol_round: i64,
}

struct Vote{
    height: i64,
    round: i64,
    value: Option<Value>,
}

struct Timeout{
    height: i64,
    round: i64,
    step: RoundStep,
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
                locked_value: None,
                locked_round: -1,
                valid_value: None,
                valid_round: -1,
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
            (RoundStep::NewRound, Event::NewRoundProposer(h, r)) => {   handle_new_round_proposer(&self, h, r) } // 11/14
            (RoundStep::NewRound, Event::NewRound(h, r)) => {   handle_new_round(s, h, r) } // 11/20
            (RoundStep::Propose, Event::Proposal(h, r, v)) => {   handle_proposal(&self, h, r, v) } // 22
            (RoundStep::Propose, Event::ProposalPolka(h, r, vr, v)) => {  handle_proposal_polka(s, h, r, vr, v) } // 28
            (RoundStep::Propose, Event::TimeoutPropose(h, r)) => {  handle_timeout_propose(s, h, r) } // 57
            (RoundStep::Prevote, Event::PolkaAny(h, r)) => {  handle_polka_any(s, h, r) } // 34
            (RoundStep::Prevote, Event::PolkaNil(h, r)) => {  handle_polka_nil(s, h, r) } // 44
            (RoundStep::Prevote, Event::PolkaValue(h, r, v)) => {  handle_polka_value(s, h, r, v) } // 36
            (RoundStep::Prevote, Event::TimeoutPrevote(h, r)) => {  handle_timeout_prevote(s, h, r) } // 61
            (RoundStep::Precommit, Event::PolkaValue(h, r, v)) => {  handle_polka_value(s, h, r, v) } // 36/42
            (_,                    Event::CommitAny(h, r)) => {  handle_commit_any(s, h, r) } // 47
            (_,                    Event::CommitValue(h, r, v)) => {  handle_commit_value(s, h, r, v) } // 49
            (_,                    Event::CertValue(h, r, v)) => {  handle_cert_value(s, h, r, v) } // 55
            (_,                    Event::TimeoutPrecommit(h, r)) => {  handle_timeout_precommit(s, h, r) } // 65
            _ => { (s, None) }
        };
        (self.with_state(s), m)
    }
}

// we're the proposer. decide a propsal.
// 11/14
fn handle_new_round_proposer<V>(sw: &StateWrapper<V>, h: i64, r: i64) -> (State, Option<Message>) 
    where V: ValueManager
{
    // update to step propose
    let s = State{
        round: r,
        step: RoundStep::Propose,
        ..sw.state
    };
    // decide proposal
    let proposal_value = match s.valid_value{
        Some(v) => { v }
        None    => { sw.value_manager.get_value() }
    };
    let proposal = Proposal{
        height: h,
        round: r,
        value: proposal_value,
        pol_round: s.valid_round,
    };
    (s, Some(Message::Proposal(proposal)))
}

// we're not the proposer. schedule timeout propose
// 11/20
fn handle_new_round(s: State, h: i64, r: i64) -> (State, Option<Message>) {
    // update to step propose
    let s = State{
        round: r,
        step: RoundStep::Propose,
        ..s
    };
    (s, Some(Message::Timeout(Timeout{
        height: s.height,
        round: s.round,
        step: s.step,
    })))
}

// received a complete proposal - prevote
// 22
fn handle_proposal<V>(sw: &StateWrapper<V>, h: i64, r: i64, proposed_value: Value) -> (State, Option<Message>)
    where V: ValueManager
{
    // update to step prevote
    let s = State{
        step: RoundStep::Prevote,
        ..sw.state
    };
    let prevote_value = match sw.value_manager.validate(proposed_value) {
        false => { None } // if its not valid, prevote nil
        true => match s.locked_value {
            Some(v) if proposed_value != v => { None } // if we're locked but the vals dont match, prevote nil
            _ => { Some(proposed_value) } // otherwise, prevote the value
        }
    };
    (s, Some(Message::Prevote(Vote{
        height: s.height,
        round: s.round,
        value: prevote_value,
    })))
}

fn handle_proposal_polka(s: State, h: i64, r: i64, vr: i64, v: Value) -> (State, Option<Message>) {
    (s, None)
}

fn handle_timeout_propose(s: State, h: i64, r: i64) -> (State, Option<Message>) {
    (s, None)
}

fn handle_polka_any(s: State, h: i64, r: i64) -> (State, Option<Message>) {
    (s, None)
}

fn handle_polka_nil(s: State, h: i64, r: i64) -> (State, Option<Message>) {
    (s, None)
}

fn handle_polka_value(s: State, h: i64, r: i64, v: Value) -> (State, Option<Message>) {
    (s, None)
}

fn handle_timeout_prevote(s: State, h: i64, r: i64) -> (State, Option<Message>) {
    (s, None)
}

fn handle_commit_any(s: State, h: i64, r: i64) -> (State, Option<Message>) {
    (s, None)
}

fn handle_commit_value(s: State, h: i64, r: i64, v: Value) -> (State, Option<Message>) {
    (s, None)
}

fn handle_cert_value(s: State, h: i64, r: i64, v: Value) -> (State, Option<Message>) {
    (s, None)
}

fn handle_timeout_precommit(s: State, h: i64, r: i64) -> (State, Option<Message>) {
    (s, None)
}



fn main() {
    println!("Hello, world!");
}

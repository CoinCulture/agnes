
// Value is what the consensus reaches agreement on.
#[derive(Copy, Clone)]
struct Value{}

// State is the state of the consensus.
#[derive(Copy, Clone)]
struct State{
    Height: i64,
    Round: i64,
    Step: RoundStep,
    LockedValue: Option<Value>,
    LockedRound: i64,
    ValidValue: Option<Value>,
    ValidRound: i64,
}

// StateWrapper contains the State, along with closures
// for getting the current proposer and proposing a new value.
struct StateWrapper<P, V>
    where P: Fn(i64) -> i64,
          V: Fn() -> Value
{ 
    State: State,
    GetProposer: P,
    GetValue: V,
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

struct Proposal{}
struct Vote{}
struct Timeout{}

impl<P,V> StateWrapper<P, V>
    where P: Fn(i64) -> i64,
          V: Fn() -> Value
{
    fn new(height: i64, get_proposer: P, get_value: V) -> StateWrapper<P,V>{
        StateWrapper{
            State: State{
                Height: height,
                Round: 0,
                Step: RoundStep::NewRound,
                LockedValue: None,
                LockedRound: -1,
                ValidValue: None,
                ValidRound: -1,
            },
            GetProposer: get_proposer,
            GetValue: get_value,
        }
    }

    fn with_state(self, state: State) -> StateWrapper<P,V>{
        StateWrapper{
            State: state,
            GetProposer: self.GetProposer,
            GetValue: self.GetValue,
        }
    }

    fn next(self, event: Event) -> (StateWrapper<P,V>, Option<Message>) {
        let s = self.State;
        let (s, m) = match (s.Step, event) {
            (RoundStep::NewRound, Event::NewRound(h, r)) => {   handle_new_round(s, h, r) } // 11
            (RoundStep::Propose, Event::Proposal(h, r, v)) => {   handle_proposal(s, h, r, v) } // 22
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

fn handle_new_round(s: State, h: i64, r: i64) -> (State, Option<Message>) {
    (s, None)
}

fn handle_proposal(s: State, h: i64, r: i64, v: Value) -> (State, Option<Message>) {
    (s, None)
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

use super::round_votes as rv;
use super::round_votes::Thresh;
use super::state_machine as sm;
use super::{Proposal, Value, Vote, VoteType};

// Executor executes valid consensus messages.
struct Executor {
    votes: rv::RoundVotes,
    state: sm::State,

    our_weight: i64,
}

// Message is a validated consensus message.
// Sequeunces of messages lead to state transitions.
// Messages may come from peers or be generated internally.
enum Message {
    Proposal(Proposal),
    Vote(Vote, i64),
    Timeout(sm::Timeout),
}

impl Executor {
    pub fn new(height: i64, our_weight: i64, total_weight: i64) -> Executor {
        Executor {
            votes: rv::RoundVotes::new(height, 0, total_weight),
            state: sm::State::new(height),
            our_weight,
        }
    }

    // Apply a message. This includes proposals and votes received from
    // peers, or timeouts generated internally.
    // Some messages generate events that are applied to the state machine.
    pub fn apply(&mut self, msg: Message) {
        // process msg. if it returns an event, apply to state machine
        if let (round, Some(event)) = self.process_msg(msg) {
            self.state = self.apply_event(sm::RoundEvent { round, event });
        }
    }

    fn get_proposal(&self, r: i64) -> Option<Value> {
        Some(Value {})
    } // TODO: use a closure

    // for proposals and votes, add to data store and return triggered event, if any.
    // for timeouts, just convert to event.
    fn process_msg(&mut self, msg: Message) -> (i64, Option<sm::Event>) {
        let (round, event) = match msg {
            Message::Proposal(p) => (p.round, Some(sm::Event::Proposal(p.pol_round, p.value))),
            Message::Vote(v, weight) => {
                let thresh = self.votes.add_vote(v, weight);
                let event = match (v.typ, thresh) {
                    (_, Thresh::Init) => None,
                    (VoteType::Prevote, Thresh::Any) => Some(sm::Event::PolkaAny),
                    (VoteType::Prevote, Thresh::Nil) => Some(sm::Event::PolkaNil),
                    (VoteType::Prevote, Thresh::Value(v)) => Some(sm::Event::PolkaValue(v)),
                    (VoteType::Precommit, Thresh::Any) => Some(sm::Event::PrecommitAny),
                    (VoteType::Precommit, Thresh::Nil) => None,
                    (VoteType::Precommit, Thresh::Value(v)) => Some(sm::Event::PrecommitValue(v)),
                };
                (v.round, event)
            }
            Message::Timeout(t) => {
                let event = match t.step {
                    sm::Step::Propose => Some(sm::Event::TimeoutPropose),
                    sm::Step::Prevote => Some(sm::Event::TimeoutPrevote),
                    sm::Step::Precommit => Some(sm::Event::TimeoutPrecommit),
                    _ => None,
                };
                (t.round, event)
            }
        };
        (round, event)
    }

    // apply an event to the state machine, calling process_msg on any
    // returned messages. calls apply_event recursively if processing the returned
    // messages results in more events. returns an updated state.
    fn apply_event(&mut self, event: sm::RoundEvent) -> sm::State {
        let s = self.state;
        let (s, msg) = s.apply(event);

        let msg = match msg {
            Some(msg) => msg,
            None => return s,
        };

        let event = match msg {
            sm::Message::NewRound(round) => {
                let proposal = self.get_proposal(round);
                let event = match proposal {
                    Some(p) => sm::Event::NewRoundProposer(p),
                    None => sm::Event::NewRound,
                };
                Some((round, Some(event)))
            }
            sm::Message::Proposal(p) => {
                let (round, event) = self.process_msg(Message::Proposal(p));
                Some((round, event))
            }
            sm::Message::Vote(v) => {
                let (round, event) = self.process_msg(Message::Vote(v, self.our_weight));
                Some((round, event))
            }
            sm::Message::Timeout(t) => {
                // TODO: schedule timeout
                None
            }
            sm::Message::Decision(v) => {
                // commit v
                // TODO: go to next height
                None
            }
        };

        match event {
            Some((round, Some(event))) => self.apply_event(sm::RoundEvent { round, event }),
            _ => s,
        }
    }
}

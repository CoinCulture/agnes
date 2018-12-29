use super::round_votes as rv;
use super::round_votes::Thresh;
use super::state_machine as sm;
use super::{Vote, VoteType};

// VoteExecutor executes valid votes at a given height.
// It adds votes to the set of votes and applies any
// resulting events to the state machine.
struct VoteExecutor {
    votes: rv::RoundVotes, // TODO: more rounds
    state: sm::State,
}

impl VoteExecutor {
    pub fn new(height: i64, total_weight: i64) -> VoteExecutor {
        let votes = rv::RoundVotes::new(height, 0, total_weight); // TODO more rounds
        let state = sm::State::new(height);
        VoteExecutor { votes, state }
    }

    // Apply a vote. If it triggers an event, apply the event to the state machine,
    // update the state, and return any result.
    pub fn apply(&mut self, vote: Vote, weight: i64) -> Option<sm::Message> {
        let thresh = self.votes.add_vote(vote, weight);
        match VoteExecutor::to_event(vote.typ, thresh) {
            None => None,
            Some(event) => {
                let (s, msg) = self.state.apply(vote.round, event);
                self.state = s;
                msg
            }
        }
    }

    fn to_event(typ: VoteType, thresh: Thresh) -> Option<sm::Event> {
        match (typ, thresh) {
            (_, Thresh::Init) => None,
            (VoteType::Prevote, Thresh::Any) => Some(sm::Event::PolkaAny),
            (VoteType::Prevote, Thresh::Nil) => Some(sm::Event::PolkaNil),
            (VoteType::Prevote, Thresh::Value(v)) => Some(sm::Event::PolkaValue(v)),
            (VoteType::Precommit, Thresh::Any) => Some(sm::Event::PrecommitAny),
            (VoteType::Precommit, Thresh::Nil) => None,
            (VoteType::Precommit, Thresh::Value(v)) => Some(sm::Event::PrecommitValue(v)),
        }
    }
}

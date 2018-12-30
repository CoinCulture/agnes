use super::round_votes as rv;
use super::round_votes::Thresh;
use super::state_machine as sm;
use super::{Vote, VoteType};

// VoteExecutor adds the vote and returns any event.
// TODO: better name, doesn't execute anymore
pub struct VoteExecutor {
    votes: rv::RoundVotes, // TODO: more rounds
}

impl VoteExecutor {
    pub fn new(height: i64, total_weight: i64) -> VoteExecutor {
        let votes = rv::RoundVotes::new(height, 0, total_weight); // TODO more rounds
        VoteExecutor { votes }
    }

    // Apply a vote. If it triggers an event, apply the event to the state machine,
    // returning the new state and any resulting message.
    pub fn apply(&mut self, vote: Vote, weight: i64) -> Option<sm::Event> {
        let thresh = self.votes.add_vote(vote, weight);
        VoteExecutor::to_event(vote.typ, thresh)
    }

    // map a vote type and threshold to a state machine event.
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

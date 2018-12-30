use super::state_machine as sm;
use super::vote_executor as ve;
use super::{Proposal, Vote, VoteType};

struct HeightVotes {}
struct ValidatorSet {}

struct ConsensusExecutor {
    height_votes: HeightVotes,
    validator_set: ValidatorSet,

    vote_executor: ve::VoteExecutor,
    state: sm::State,
}

enum Message {
    Proposal(Proposal),
    Vote(Vote),
    Timeout(sm::Timeout),
}

impl ConsensusExecutor {
    // execute the message in full. may result in multiple state transitions.
    pub fn execute(&mut self, msg: Message) {
        let msg = match self.apply_msg(msg) {
            None => return,
            Some(m) => m,
        };

        match msg {
            sm::Message::NewRound(round) => {
                // check if we're the proposer
            }
            sm::Message::Proposal(p) => {
                // sign the proposal
                // call execute
            }
            sm::Message::Vote(v) => {
                // sign the vote
                // call execute
            }
            sm::Message::Timeout(t) => {
                // schedule the timeout
            }
            sm::Message::Decision(d) => {
                // update the state
            }
        }
    }
}

impl ConsensusExecutor {
    // apply a single consensus message against the state
    pub fn apply_msg(&mut self, msg: Message) -> Option<sm::Message> {
        match msg {
            Message::Proposal(p) => {
                // TODO: check for invalid proposal
                let event = sm::Event::Proposal(p.pol_round, p.value);
                self.apply_event(p.round, event)
            }
            Message::Vote(v) => {
                // TODO: get weight
                let weight = 1;
                let event = match self.vote_executor.apply(v, weight) {
                    None => return None,
                    Some(event) => event,
                };
                self.apply_event(v.round, event)
            }
            Message::Timeout(t) => {
                let event = match t.step {
                    sm::TimeoutStep::Propose => sm::Event::TimeoutPropose,
                    sm::TimeoutStep::Prevote => sm::Event::TimeoutPrevote,
                    sm::TimeoutStep::Precommit => sm::Event::TimeoutPrecommit,
                };
                self.apply_event(t.round, event)
            }
        }
    }

    // apply the event, update the state.
    fn apply_event(&mut self, round: i64, event: sm::Event) -> Option<sm::Message> {
        let (s, msg) = self.state.apply(round, event);
        self.state = s;
        msg
    }
}

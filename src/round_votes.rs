use super::{Value, Vote, VoteType};

//-------------------------------------------------------------------------
// Tally votes of the same type (eg. prevote or precommit)

// ValueWeight represents a value and the weight of votes for it.
struct ValueWeight {
    value: Value,
    weight: i64,
}

// VoteCount tallys votes of the same type.
// Votes are for nil or for some value.
//(TODO: handle multiple values)
struct VoteCount {
    nil: i64,           // weight of votes for nil
    value: ValueWeight, // weight of votes for the value
    total: i64,
}

// Thresh represents the different quorum thresholds.
#[derive(Debug, PartialEq)]
pub enum Thresh {
    Init,         // no quorum
    Any,          // quorum of votes but not for the same value
    Nil,          // quorum for nil
    Value(Value), // quorum for the value
}

// is_quorum returns true if value > (2/3)*total.
fn is_quorum(value: i64, total: i64) -> bool {
    3 * value > 2 * total
}

impl VoteCount {
    fn new(total: i64) -> VoteCount {
        VoteCount {
            nil: 0,
            value: ValueWeight {
                value: Value {}, // TODO
                weight: 0,
            },
            total,
        }
    }

    // Add vote to internal counters and return the highest threshold.
    fn add_vote(&mut self, vote: Vote, weight: i64) -> Thresh {
        match vote.value {
            Some(v) => {
                // TODO: handle multi values
                self.value.weight += weight;
                self.value.value = v;
            }
            None => self.nil += weight,
        }

        if is_quorum(self.value.weight, self.total) {
            Thresh::Value(self.value.value)
        } else if is_quorum(self.nil, self.total) {
            Thresh::Nil
        } else if is_quorum(self.value.weight + self.nil, self.total) {
            Thresh::Any
        } else {
            Thresh::Init
        }
    }
}

//-------------------------------------------------------------------------
// RoundVotes

// RoundVotes tracks all the votes for a single round
pub struct RoundVotes {
    height: i64,
    round: i64,

    prevotes: VoteCount,
    precommits: VoteCount,
}

impl RoundVotes {
    pub fn new(height: i64, round: i64, total: i64) -> RoundVotes {
        RoundVotes {
            height,
            round,
            prevotes: VoteCount::new(total),
            precommits: VoteCount::new(total),
        }
    }

    pub fn add_vote(&mut self, vote: Vote, weight: i64) -> Thresh {
        match vote.typ {
            VoteType::Prevote => self.prevotes.add_vote(vote, weight),
            VoteType::Precommit => self.precommits.add_vote(vote, weight),
        }
    }
}

//---------------------------------------------------------------------
// Test

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_votes() {
        let v = Value {};
        let val = Some(v);
        let total = 4;
        let mut round_votes = RoundVotes::new(1, 0, total);
        let weight = 1;

        // add a vote. nothing changes.
        let vote = Vote::new_prevote(0, val);
        let thresh = round_votes.add_vote(vote, weight);
        assert_eq!(thresh, Thresh::Init);

        // add it again, nothing changes.
        let thresh = round_votes.add_vote(vote, weight);
        assert_eq!(thresh, Thresh::Init);

        // add a vote for nil, get Thresh::Any
        let vote_nil = Vote::new_prevote(0, None);
        let thresh = round_votes.add_vote(vote_nil, weight);
        assert_eq!(thresh, Thresh::Any);

        // add vote for value, get Thresh::Value
        let thresh = round_votes.add_vote(vote, weight);
        assert_eq!(thresh, Thresh::Value(v));
    }
}

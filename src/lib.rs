#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Value {}

// Proposal proposes a value in a round.
// pol_round is -1 or the last round this value got a polka.
#[derive(Debug, PartialEq)]
pub struct Proposal {
    round: i64,
    value: Value,
    pol_round: i64,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum VoteType {
    Prevote,
    Precommit,
}

// Vote is a vote for a value in a round.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vote {
    typ: VoteType,
    round: i64,
    value: Option<Value>,
}

impl Vote {
    pub fn new_prevote(round: i64, value: Option<Value>) -> Vote {
        Vote {
            typ: VoteType::Prevote,
            round,
            value,
        }
    }

    pub fn new_precommit(round: i64, value: Option<Value>) -> Vote {
        Vote {
            typ: VoteType::Precommit,
            round,
            value,
        }
    }
}

pub mod round_votes;
pub mod state_machine;

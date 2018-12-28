// Value is what the consensus algorithm seeks agreement on.
// TODO: it should probably be a Trait - currently it's empty.
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
        let typ = VoteType::Prevote;
        Vote { typ, round, value }
    }

    pub fn new_precommit(round: i64, value: Option<Value>) -> Vote {
        let typ = VoteType::Precommit;
        Vote { typ, round, value }
    }
}

pub mod round_votes;
pub mod state_machine;
pub mod vote_executor;

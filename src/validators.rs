//----------------------------------
// Validator

// Validator is a public key and voting power
pub struct Validator {
    pub public_key: Vec<u8>, // TODO: trait?
    pub voting_power: i64,
}

impl Validator {
    pub fn hash(&self) -> Vec<u8> {
        Vec::new() // TODO
    }

    pub fn address(&self) -> Vec<u8> {
        self.public_key // TODO
    }
}

//--------------------------------

// ValidatorSet contains a list of validators sorted by address.
pub struct ValidatorSet {
    validators: Vec<Validator>,
}

impl ValidatorSet {
    pub fn new(vals: Vec<Validator>) -> ValidatorSet {
        ValidatorSet::sort(vals);
        let val_set = ValidatorSet { validators: vals };
    }

    pub fn add(&mut self, val: Validator) {
        self.validators.push(val);
        ValidatorSet::sort(self.validators);
    }

    pub fn update(&mut self, val: Validator) {
        // find val in list
        // update voting power
    }

    pub fn remove(&mut self, val: Validator) {
        // find val in list
        // remove
    }

    // in place sort a list of validators
    fn sort(vals: &mut Vec<Validator>) {
        vals.sort_unstable_by(|v1, v2| {
            let (v1_addr, v2_addr) = (v1.address(), v2.address());
            v1_addr.cmp(v2_addr)
        });
        vals.dedup();
    }
}

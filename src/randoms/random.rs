use bevy::prelude::*;
use rand::{
    Rng, SeedableRng, distr::uniform::SampleRange, distr::uniform::SampleUniform, rngs::StdRng,
};

#[derive(Resource)]
pub struct Random {
    pub rng: StdRng,
}

impl Default for Random {
    fn default() -> Self {
        Self {
            rng: StdRng::seed_from_u64(123456),
        }
    }
}

impl Random {
    pub fn set_seed(&mut self, seed: u64) {
        self.rng = StdRng::seed_from_u64(seed);
    }

    pub fn random_range<T, R>(&mut self, range: R) -> T
    where
        T: SampleUniform,
        R: SampleRange<T>,
    {
        self.rng.random_range(range)
    }

    pub fn random_bool(&mut self, probability: f64) -> bool {
        self.rng.random_bool(probability)
    }
}

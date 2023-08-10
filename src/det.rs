use crate::{Crn, CrnSim, Reaction, State};

const MAX_POINTS: usize = 100000;

/// A deterministic CRN. In a sense this is the "limiting" behavior of a stochastic CRN as the number of species are scaled to infinity.
pub type DetCrn = Crn<f64>;

impl DetCrn {
    /// Simulates a single timestep.
    pub fn step(&mut self, dt: f64) {
        let k1 = self.state.species_rates(&self.rxns);
        let k2 = (&self.state + &(&k1 * (dt / 2.0))).species_rates(&self.rxns);
        let k3 = (&self.state + &(&k2 * (dt / 2.0))).species_rates(&self.rxns);
        let k4 = (&self.state + &(&k3 * dt)).species_rates(&self.rxns);

        let delta = &(&(&k1 + &(&k2 * 3.0)) + &(&(&k3 * 3.0) + &k4)) * (dt / 6.0);

        self.state = &self.state + &delta;

        self.state.time += dt;
    }

    /// Simulates a number of steps with a given timestep. Returns a collection of individual species' history.
    pub fn simulate_data(&mut self, steps: usize, dt: f64) -> Vec<Vec<(f64, f64)>> {
        let ratio = (steps / MAX_POINTS).max(1);
        let mut species: Vec<Vec<(f64, f64)>> =
            vec![Vec::with_capacity(steps.min(MAX_POINTS)); self.state.species.len()];
        for j in 0..steps {
            if j % ratio == 0 {
                // println!("{j}");
                for (i, s) in self.state.species.iter().enumerate() {
                    species[i].push((self.state.time, *s));
                }
            }
            self.step(dt);
        }
        species
    }
}

impl CrnSim for DetCrn {
    fn reactions(&self) -> &[Reaction] {
        &self.rxns
    }

    fn state(&self) -> State<f64> {
        self.state.clone()
    }

    fn simulate_history(&mut self, t: f64, dt: f64) -> Result<Vec<State<f64>>, crate::Error> {
        let steps = (t / dt).ceil() as usize;
        let mut result: Vec<State<f64>> = Vec::with_capacity(steps);
        for _ in 0..steps {
            result.push(self.state.clone());
            self.step(dt);
        }
        Ok(result)
    }

    fn reset(&mut self) {
        self.state = self.init_state.clone();
    }
}

impl From<String> for DetCrn {
    fn from(s: String) -> Self {
        Self::parse(&s).unwrap()
    }
}

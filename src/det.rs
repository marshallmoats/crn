use crate::{Crn, State};

const MAX_POINTS: usize = 100000;

/// A deterministic CRN. In a sense this is the "limiting" behavior of a stochastic CRN as the amounts of each species are scaled to infinity.
pub type DetCrn = Crn<f64>;

impl DetCrn {
    /// Simulates a single timestep.
    pub fn step(&mut self, dt: f64) {
        let k1 = self.state.species_rates(&self.rxns);
        let k2 = (&self.state + &(&k1 * (dt / 2.0))).species_rates(&self.rxns);
        let k3 = (&self.state + &(&k2 * (dt / 2.0))).species_rates(&self.rxns);
        let k4 = (&self.state + &(&k3 * dt)).species_rates(&self.rxns);

        let delta = &(&(&k1 + &(&k2 * 2.0)) + &(&(&k3 * 2.0) + &k4)) * (dt / 6.0);

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

impl DetCrn {
    /// Simulates for a given amount of time. Returns a collection of individual species' history.
    pub fn simulate_history(&mut self, t: f64, dt: f64) -> Result<Vec<State<f64>>, crate::Error> {
        let steps = (t / dt).ceil() as usize;
        let mut result: Vec<State<f64>> = Vec::with_capacity(steps);
        for _ in 0..steps {
            result.push(self.state.clone());
            self.step(dt);
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use approx::{assert_relative_eq, assert_abs_diff_eq};

    use crate::DetCrn;

    #[test]
    fn test() {
        const T: f64 = 1.0;
        let mut crn = DetCrn::parse("A = 1; A -> ;").unwrap();
        crn.simulate_history(T, 0.001).unwrap();
        assert_relative_eq!(crn.state.species[0], (-T).exp(), max_relative = 0.001);
    }

    #[test]
    fn test2() {
        const T: f64 = 10.0;
        let mut crn = DetCrn::parse("A = 0; -> A;").unwrap();
        crn.simulate_history(T, 0.001).unwrap();
        assert_abs_diff_eq!(crn.state.species[0], T, epsilon = 0.001);
    }

    #[test]
    fn test3() {
        const T: f64 = 10.0;
        let mut crn = DetCrn::parse("A = 1; A -> 2A;").unwrap();
        crn.simulate_history(T, 0.001).unwrap();
        assert_abs_diff_eq!(crn.state.species[0], T.exp(), epsilon = 0.001);
    }

    #[test]
    fn test4() {
        const T: f64 = 10.0;
        let mut crn = DetCrn::parse("A = 1; A -> B;").unwrap();
        crn.simulate_history(T, 0.001).unwrap();
        assert_abs_diff_eq!(crn.state.species[1], 1.0 - (-T).exp(), epsilon = 0.001);
    }
}
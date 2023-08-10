use std::{collections::HashMap, fmt::Display};

use itertools::Itertools;

use crate::{Crn, Reaction, State, C};

const MAX_POINTS: usize = 100000;

/// A deterministic CRN. In a sense this is the "limiting" behavior of a stochastic CRN as the number of species are scaled to infinity.
pub type DetCrn = C<f64>;

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

impl Display for DetCrn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let reactants_to_string = move |reactants: &HashMap<usize, i32>| -> String {
            if reactants.is_empty() {
                return String::new();
            }
            let mut result = reactants
                .iter()
                .sorted()
                .fold(String::new(), |acc, (i, count)| {
                    if *count == 1 {
                        acc + self.names.get_by_left(i).unwrap() + " + "
                    } else {
                        acc + &format!("{}{} + ", count, self.names.get_by_left(i).unwrap())
                    }
                });
            result.truncate(result.len() - 3);
            result
        };

        let mut result = String::new();

        for (i, ct) in self.state.species.iter().enumerate() {
            result.push_str(&format!(
                "{} = {};\n",
                self.names.get_by_left(&i).unwrap(),
                ct
            ));
        }

        for rxn in self.rxns.iter() {
            result.push_str(&format!(
                "{} -> {} : {};\n",
                reactants_to_string(&rxn.reactants),
                reactants_to_string(&rxn.products),
                rxn.rate
            ));
        }
        write!(f, "{}", result)
    }
}

impl Crn for DetCrn {
    fn reactions(&self) -> &[Reaction] {
        &self.rxns
    }

    fn state(&self) -> State<f64> {
        self.state.clone()
    }

    fn simulate_history(&mut self, t: f64, dt: f64) -> Result<Vec<State<f64>>, crate::Error> {
        let steps = (t / dt).ceil() as usize;
        let mut species: Vec<State<f64>> = Vec::with_capacity(steps);
        for _ in 0..steps {
            species.push(self.state.clone());
            self.step(dt);
        }
        Ok(species)
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

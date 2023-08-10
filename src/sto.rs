use rand::Rng;

use crate::{Crn, Reaction, C, state::State};

// const MAX_POINTS: usize = 100000;

use std::fmt::Debug;

use thiserror::Error;

/// A simulation can fail because no more reactions are possible, or because of numerical instability.
#[derive(Error, Debug)]
pub enum Error {
    /// No reactions are possible from the current state.
    #[error("CRN has reached terminal state")]
    TerminalState,
    /// The simulation has become numerically unstable.
    #[error("Insufficient precision for accurate simulation")]
    InsufficientPrecision,
}

/// A stochastic CRN. This is simulated using the Gillespie algorithm. Stochastic CRNs are essentially a type of continuous-time Markov chain.
pub type StoCrn = C<i32>;

impl StoCrn {
    /// Simulate one reaction. Uses `rates` to avoid repeated allocations.
    pub fn step(&mut self, rates: &mut [f64]) -> Result<(), Error> {
        let mut rate = 0.0;

        self.rxns.iter().enumerate().for_each(|(idx, rxn)| {
            let cur_rate = self.state.rate(rxn);
            rates[idx] = cur_rate;
            rate += cur_rate;
        });

        if rate == 0.0 {
            return Err(Error::TerminalState);
        }

        let mut rng = rand::thread_rng();
        // the random number is in (0, 1], so the ln is negative or zero and this is really an addition
        self.state.time -= (1.0 - rng.gen::<f64>()).ln() / rate;
        let j = rng.gen::<f64>() * rate;
        let mut sum = 0.0;

        for (idx, cur_rate) in rates.iter().enumerate() {
            sum += cur_rate;
            if j < sum {
                self.state.apply(&self.rxns[idx]);
                return Ok(());
            }
        }
        Err(Error::InsufficientPrecision)
    }

    /// Simulate a number of reactions.
    pub fn steps(&mut self, steps: usize) -> Result<(), Error> {
        let mut rates = vec![0.0; self.rxns.len()];
        for _ in 0..steps {
            self.step(&mut rates)?;
        }
        Ok(())
    }

    // pub fn simulate_history(&mut self, steps: usize) -> Result<Vec<Vec<(f64, f64)>>, Error> {
    //     let mut res = vec![Vec::with_capacity(steps.min(MAX_POINTS)); self.state.species.len()];

    //     let mut rates = vec![0.0; self.rxns.len()];

    //     if steps > MAX_POINTS {
    //         let ratio = steps / MAX_POINTS;
    //         // println!("ratio: {}", ratio);
    //         for i in 0..steps {
    //             if i % ratio == 0 {
    //                 for (j, s) in self.state.species.iter().enumerate() {
    //                     res[j].push((self.state.time, *s as f64));
    //                 }
    //             }
    //             match self.step(&mut rates) {
    //                 Ok(_) => {}
    //                 Err(Error::TerminalState) => break,
    //                 Err(e) => return Err(e),
    //             }
    //         }
    //     } else {
    //         for _ in 0..steps {
    //             for (j, s) in self.state.species.iter().enumerate() {
    //                 res[j].push((self.state.time, *s as f64));
    //             }
    //             match self.step(&mut rates) {
    //                 Ok(_) => {}
    //                 Err(Error::TerminalState) => break,
    //                 Err(e) => return Err(e),
    //             }
    //         }
    //     }

    //     Ok(res)
    // }
}

impl Crn for StoCrn {
    fn reactions(&self) -> &[Reaction] {
        &self.rxns
    }

    fn state(&self) -> State<f64> {
        let state = State {
            species: self.state.species.iter().map(|x| *x as f64).collect(),
            time: self.state.time,
        };
        state
    }

    fn simulate_history(&mut self, t: f64, _dt: f64) -> Result<Vec<State<f64>>, Error> {
        let mut result = Vec::new();

        let mut rates = vec![0.0; self.rxns.len()];
        while self.state.time < t {
            if self.step(&mut rates).is_err() {
                break;
            }
            let species = self.state.species.iter().map(|x| *x as f64).collect();
            result.push(State {
                species,
                time: self.state.time,
            });
        }
        Ok(result)
    }

    fn reset(&mut self) {
        self.state = self.init_state.clone();
    }
}

impl From<String> for StoCrn {
    fn from(s: String) -> Self {
        StoCrn::parse(&s).unwrap()
    }
}

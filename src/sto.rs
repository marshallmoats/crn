use rand::Rng;

use crate::{state::State, Crn};

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
pub type StoCrn = Crn<i32>;

impl StoCrn {
    /// Simulate one reaction. Uses `rates` to avoid repeated allocations.
    fn step(&mut self, rates: &mut [f64]) -> Result<(), Error> {
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

    /// Simulates for a given amount of time. Returns a collection of individual species' history.
    pub fn simulate_history(&mut self, t: f64) -> Result<Vec<State<f64>>, Error> {
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
}

#[cfg(test)]
mod tests {
    use crate::StoCrn;

    #[test]
    fn test() {
        const N: i32 = 100;
        let mut crn = StoCrn::parse(&format!("A = {N}; A -> ;")).unwrap();
        let mut rates = vec![0.0; crn.rxns.len()];
        for i in (0..N).rev() {
            crn.step(&mut rates).unwrap();
            assert_eq!(crn.state.species[0], i);
        }
    }

    #[test]
    fn test2() {
        let mut crn = StoCrn::parse("A = 1; B = 1; A + B -> C; C -> A + B;").unwrap();
        let mut rates = vec![0.0; crn.rxns.len()];
        crn.step(&mut rates).unwrap();
        assert_eq!(crn.state.species[0], 0);
        assert_eq!(crn.state.species[1], 0);
        assert_eq!(crn.state.species[2], 1);
        crn.step(&mut rates).unwrap();
        assert_eq!(crn.state.species[0], 1);
        assert_eq!(crn.state.species[1], 1);
        assert_eq!(crn.state.species[2], 0);
    }
}
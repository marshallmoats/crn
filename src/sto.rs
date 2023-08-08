use itertools::Itertools;
use rand::Rng;
use std::collections::HashMap;

use crate::{Reaction, State};

const MAX_POINTS: usize = 100000;

use std::fmt::Debug;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("CRN has reached terminal state")]
    TerminalState,
    #[error("Insufficient precision for accurate simulation")]
    InsufficientPrecision,
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct StoCrn {
    pub state: State<i32>,
    pub init_state: State<i32>,
    pub rxns: Vec<Reaction>,
    pub names: bimap::BiHashMap<usize, String>,
}

impl StoCrn {
    pub fn reset(&mut self) {
        self.state = self.init_state.clone();
    }

    pub fn single_step(&mut self) -> Result<(), Error> {
        self.step(&mut vec![0.0; self.rxns.len()])
    }

    // reuses the rates vector to avoid reallocating every step
    // if you only need one step, use single_step
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

    pub fn steps(&mut self, steps: usize) -> Result<(), Error> {
        let mut rates = vec![0.0; self.rxns.len()];
        for _ in 0..steps {
            self.step(&mut rates)?;
        }
        Ok(())
    }

    pub fn simulate_history(&mut self, steps: usize) -> Result<Vec<Vec<(f64, f64)>>, Error> {
        let mut res = vec![Vec::with_capacity(steps.min(MAX_POINTS)); self.state.species.len()];

        let mut rates = vec![0.0; self.rxns.len()];

        if steps > MAX_POINTS {
            let ratio = steps / MAX_POINTS;
            println!("ratio: {}", ratio);
            for i in 0..steps {
                if i % ratio == 0 {
                    for (j, s) in self.state.species.iter().enumerate() {
                        res[j].push((self.state.time, *s as f64));
                    }
                }
                match self.step(&mut rates) {
                    Ok(_) => {}
                    Err(Error::TerminalState) => break,
                    Err(e) => return Err(e),
                }
            }
        } else {
            for _ in 0..steps {
                for (j, s) in self.state.species.iter().enumerate() {
                    res[j].push((self.state.time, *s as f64));
                }
                match self.step(&mut rates) {
                    Ok(_) => {}
                    Err(Error::TerminalState) => break,
                    Err(e) => return Err(e),
                }
            }
        }

        Ok(res)
    }

    // pub fn simulate_data(
    //     &mut self,
    //     steps: usize,
    //     relative: bool,
    // ) -> Result<Vec<Vec<(f64, f64)>>, error::Error> {
    //     let hist = self.simulate_history(steps)?;

    //     Ok((0..self.state.species.len())
    //         .map(|i| {
    //             hist.iter()
    //                 .map(|state| {
    //                     // apparently this gets optimized heavily
    //                     // seems like it only checks relative once
    //                     if relative {
    //                         let total = state.species.iter().sum::<i32>() as f64;
    //                         (state.time, state.species[i] as f64 / total)
    //                     } else {
    //                         (state.time, state.species[i] as f64)
    //                     }
    //                 })
    //                 .collect::<Vec<(f64, f64)>>()
    //         })
    //         .collect())
    // }
}

impl ToString for StoCrn {
    fn to_string(&self) -> String {
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

        result
    }
}

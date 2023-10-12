//! # crn
//! A library for simulating chemical reaction networks.

#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use std::collections::HashMap;
use std::fmt::Display;
use std::str::FromStr;

pub use det::DetCrn;
use itertools::Itertools;
pub use state::State;
pub use sto::Error;
pub use sto::StoCrn;

/// Deterministic CRNs.
pub mod det;
/// Parsing CRNs from strings.
pub mod parse;
/// Some fun CRNs to play with.
pub mod presets;
/// State of a CRN.
pub mod state;
/// Stochastic CRNs.
pub mod sto;

/// A chemical reaction, with a rate parameter.
#[derive(Clone, Debug, PartialEq)]
pub struct Reaction {
    /// Reactants and their stoichiometric coefficients.
    pub reactants: HashMap<usize, i32>,
    /// Products and their stoichiometric coefficients.
    pub products: HashMap<usize, i32>,
    /// The change in a species' amount when this reaction occurs.
    pub delta: HashMap<usize, i32>,
    /// The rate parameter of this reaction.
    pub rate: f64,
}

impl Reaction {
    /// Create a new reaction from reactants, products, and a rate parameter.
    pub fn new(reactants: HashMap<usize, i32>, products: HashMap<usize, i32>, rate: f64) -> Self {
        Self {
            reactants: reactants.clone(),
            delta: {
                let mut hm = products.clone();

                for (species, count) in reactants {
                    if let Some(cur_count) = hm.get_mut(&species) {
                        *cur_count -= count;
                    } else {
                        hm.insert(species, -count);
                    }
                }
                hm
            },
            products,
            rate,
        }
    }
}

/// Shared behavior for stochastic and deterministic CRNs.
#[derive(Default, Clone)]
pub struct Crn<T> {
    /// The CRN's reactions.
    pub rxns: Vec<Reaction>,
    /// The CRN's current state.
    pub state: State<T>,
    /// The CRN's initial state, which it reverts to on a reset.
    pub init_state: State<T>,
    /// The name of each species.
    pub names: bimap::BiHashMap<usize, String>,
}

impl<T> Crn<T>
where
    T: Clone,
{
    /// Resets the CRN to its initial state.
    pub fn reset(&mut self) {
        self.state = self.init_state.clone();
    }
}

impl<T> Display for Crn<T>
where
    T: Display,
{
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

impl<T> From<String> for Crn<T>
where
    T: Clone + Default + FromStr,
    <T as FromStr>::Err: std::fmt::Debug,
{
    fn from(s: String) -> Self {
        Self::parse(&s).unwrap()
    }
}

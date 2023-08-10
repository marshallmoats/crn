//! # crn
//! A library for simulating chemical reaction networks.

use std::collections::HashMap;

pub use det::DetCrn;
pub use sto::Error;
pub use sto::StoCrn;

pub mod det;
pub mod parse;
pub mod presets;
pub mod sto;

#[derive(Clone, Debug, PartialEq)]
pub struct Reaction {
    reactants: HashMap<usize, i32>,
    products: HashMap<usize, i32>,
    delta: HashMap<usize, i32>,
    rate: f64,
}

impl Reaction {
    fn new(reactants: HashMap<usize, i32>, products: HashMap<usize, i32>, rate: f64) -> Self {
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

#[derive(Clone, Debug, Default, PartialEq)]
pub struct State<T> {
    pub species: Vec<T>,
    pub time: f64,
}

impl<T> State<T> {
    pub fn new(species: Vec<T>, time: f64) -> Self {
        Self { species, time }
    }
}

impl State<i32> {
    pub fn apply(&mut self, rxn: &Reaction) {
        for (i, d) in rxn.delta.iter() {
            self.species[*i] += d;
        }
    }

    pub fn applicable(&self, rxn: &Reaction) -> bool {
        rxn.reactants
            .iter()
            .all(|(species, count)| count <= &self.species[*species])
    }

    pub fn rate(&self, rxn: &Reaction) -> f64 {
        if self.applicable(rxn) {
            rxn.reactants
                .iter()
                .fold(rxn.rate, |mut cur, (species, count)| {
                    for i in (self.species[*species] - count + 1)..=self.species[*species] {
                        cur *= i as f64
                    }
                    cur
                })
        } else {
            0.0
        }
    }
}

impl State<f64> {
    pub fn rate(&self, rxn: &Reaction) -> f64 {
        rxn.reactants
            .iter()
            .fold(rxn.rate, |cur, (species, count)| {
                cur * self.species[*species].powi(*count)
            })
    }

    pub fn species_rates(&self, rxns: &[Reaction]) -> Self {
        let mut res = Self {
            species: vec![0.0; self.species.len()],
            time: 0.0,
        };
        rxns.iter().for_each(|rxn| {
            let rate = self.rate(rxn);
            for (species, change) in &rxn.delta {
                res.species[*species] += *change as f64 * rate;
            }
        });
        res
    }
}

impl<T> std::ops::Add for &State<T>
where
    T: std::ops::Add<Output = T> + Copy,
{
    type Output = State<T>;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            species: self
                .species
                .iter()
                .zip(rhs.species.iter())
                .map(|(a, b)| *a + *b)
                .collect(),
            time: self.time,
        }
    }
}

impl<T> std::ops::AddAssign for State<T>
where
    T: std::ops::AddAssign + Copy,
{
    fn add_assign(&mut self, rhs: Self) {
        self.species
            .iter_mut()
            .zip(rhs.species.iter())
            .for_each(|(a, b)| *a += *b);
    }
}

impl<T> std::ops::Mul<f64> for &State<T>
where
    T: std::ops::Mul<f64, Output = T> + Copy,
{
    type Output = State<T>;

    fn mul(self, rhs: f64) -> Self::Output {
        Self::Output {
            species: self.species.iter().map(|a| *a * rhs).collect(),
            time: self.time,
        }
    }
}

impl<T> std::ops::MulAssign<f64> for State<T>
where
    T: std::ops::MulAssign<f64> + Copy,
{
    fn mul_assign(&mut self, rhs: f64) {
        self.species.iter_mut().for_each(|a| *a *= rhs);
    }
}

pub trait Crn: ToString {
    fn reactions(&self) -> &[Reaction];
    fn state(&self) -> State<f64>;
    // fn simulate(&mut self, t: f64, dt: f64) -> Result<State<T>, Error>;
    fn simulate_history(&mut self, t: f64, dt: f64) -> Result<Vec<State<f64>>, Error>;
    fn reset(&mut self);
}

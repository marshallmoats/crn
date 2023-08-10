use crate::Reaction;

/// A state of a CRN. StoCrn uses integers, DetCrn uses floats.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct State<T> {
    /// Amount of each species. Will be an integer for stochastic CRNs, and a float for deterministic CRNs.
    pub species: Vec<T>,
    /// Current time.
    pub time: f64,
}

impl State<i32> {
    /// Applies a reaction, modifying the amounts of each species.
    pub fn apply(&mut self, rxn: &Reaction) {
        for (i, d) in rxn.delta.iter() {
            self.species[*i] += d;
        }
    }

    /// Returns true if the reaction is applicable to the current state.
    pub fn applicable(&self, rxn: &Reaction) -> bool {
        rxn.reactants
            .iter()
            .all(|(species, count)| count <= &self.species[*species])
    }

    /// Returns the rate at which this reaction is occurring -- if the reactants are more abundant, this will be higher. Note that this is scaled by the rate parameter of the reaction.
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
    /// Returns the rate at which this reaction is occurring -- if the reactants are more abundant, this will be higher. Note that this is scaled by the rate parameter of the reaction.
    pub fn rate(&self, rxn: &Reaction) -> f64 {
        rxn.reactants
            .iter()
            .fold(rxn.rate, |cur, (species, count)| {
                cur * self.species[*species].powi(*count)
            })
    }

    /// Given a set of reactions, returns the instantaneous rate of change of each species.
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
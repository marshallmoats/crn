use std::collections::HashMap;

use itertools::Itertools;

use crate::{
    parse::{parse_counts, parse_reactions, ParseError},
    Crn, Reaction, State,
};

const MAX_POINTS: usize = 100000;

#[derive(Default, Clone)]
pub struct DetCrn {
    pub rxns: Vec<Reaction>,
    pub state: State<f64>,
    pub init_state: State<f64>,
    pub names: bimap::BiHashMap<usize, String>,
}

impl DetCrn {
    pub fn new(
        rxns: Vec<Reaction>,
        state: State<f64>,
        names: bimap::BiHashMap<usize, String>,
    ) -> Self {
        Self {
            rxns,
            init_state: state.clone(),
            state,
            names,
        }
    }

    pub fn step(&mut self, dt: f64) {
        let k1 = self.state.species_rates(&self.rxns);
        let k2 = (&self.state + &(&k1 * (dt / 2.0))).species_rates(&self.rxns);
        let k3 = (&self.state + &(&k2 * (dt / 2.0))).species_rates(&self.rxns);
        let k4 = (&self.state + &(&k3 * dt)).species_rates(&self.rxns);

        let delta = &(&(&k1 + &(&k2 * 3.0)) + &(&(&k3 * 3.0) + &k4)) * (dt / 6.0);

        self.state = &self.state + &delta;

        self.state.time += dt;
    }

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

    pub fn parse(input: &str) -> Result<DetCrn, ParseError> {
        let (leftover_input, counts) = parse_counts(input).unwrap();
        let mut species_map: HashMap<&str, usize> = HashMap::new();
        let mut names = bimap::BiHashMap::<usize, String>::new();
        let mut x = Vec::<f64>::with_capacity(counts.len());
        for (i, (species, num)) in counts.iter().enumerate() {
            if species_map.contains_key(species) {
                return Err(ParseError::DuplicateDefinition(species.to_string()));
            } else {
                species_map.insert(species, i);
                names.insert(i, species.to_string());
                x.push(num.parse::<f64>().unwrap());
            }
        }

        let (_leftover_input, reactions) = parse_reactions(leftover_input).unwrap();

        let mut rxns = Vec::<Reaction>::with_capacity(reactions.len());

        for ((reactants, products), rate) in reactions {
            let mut reactant_map: HashMap<usize, i32> = HashMap::new();
            let mut product_map: HashMap<usize, i32> = HashMap::new();

            for (num, species) in reactants {
                let num: i32 = if num.is_empty() {
                    1
                } else {
                    num.parse().unwrap()
                };
                if !species_map.contains_key(species) {
                    let len = species_map.len();
                    species_map.insert(species, len);
                    names.insert(len, species.to_string());
                    x.push(0.0);
                    reactant_map.insert(len, num);
                } else {
                    reactant_map.insert(species_map[species], num);
                }
            }

            for (num, species) in products {
                let num: i32 = if num.is_empty() {
                    1
                } else {
                    num.parse().unwrap()
                };
                if !species_map.contains_key(species) {
                    let len = species_map.len();
                    species_map.insert(species, len);
                    names.insert(len, species.to_string());
                    x.push(0.0);
                    product_map.insert(len, num);
                } else {
                    product_map.insert(species_map[species], num);
                }
            }
            let rxn = Reaction::new(reactant_map, product_map, rate.unwrap_or(1.0));
            rxns.push(rxn);
        }

        let state = State::new(x, 0.0);
        Ok(DetCrn::new(rxns, state, names))
    }
}

impl ToString for DetCrn {
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

impl Crn for DetCrn {
    fn reactions(&self) -> &[Reaction] {
        &self.rxns
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

use std::collections::HashMap;

use nom::{
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric0, digit0, multispace0},
    combinator::{opt, recognize},
    multi::{many0, separated_list0},
    number::complete::double,
    sequence::{delimited, pair, separated_pair, terminated},
    IResult,
};

use crate::{state::State, Crn, Reaction};

/// Errors that can occur while parsing a CRN.
#[derive(Debug, Clone)]
pub enum ParseError {
    /// Species amount was defined twice.
    DuplicateDefinition(String),
}

/// Parse the name of a species.
fn species_name(input: &str) -> IResult<&str, &str> {
    delimited(
        multispace0,
        recognize(pair(alpha1, alphanumeric0)),
        multispace0,
    )(input)
}

/// Parse a species amount definition.
fn parse_count(input: &str) -> IResult<&str, (&str, &str)> {
    delimited(
        multispace0,
        terminated(
            separated_pair(
                species_name,
                separated_pair(multispace0, tag("="), multispace0),
                nom::number::complete::recognize_float,
            ),
            tag(";"),
        ),
        multispace0,
    )(input)
}

/// Parse multiple species amount definitions.
fn parse_counts(input: &str) -> IResult<&str, Vec<(&str, &str)>> {
    many0(parse_count)(input)
}

/// Parse a species with an optional stoichiometric coefficient.
fn parse_reactant(input: &str) -> IResult<&str, (&str, &str)> {
    delimited(multispace0, pair(digit0, species_name), multispace0)(input)
}

/// Parse multiple species with optional stoichiometric coefficients.
fn parse_reactants(input: &str) -> IResult<&str, Vec<(&str, &str)>> {
    delimited(
        multispace0,
        separated_list0(
            delimited(multispace0, tag("+"), multispace0),
            parse_reactant,
        ),
        multispace0,
    )(input)
}

/// Result of parsing a reaction.
type ReactionTokens<'a> = (
    (Vec<(&'a str, &'a str)>, Vec<(&'a str, &'a str)>),
    Option<f64>,
);

/// Parse a reaction with an optional rate parameter.
fn parse_reaction(input: &str) -> IResult<&str, ReactionTokens> {
    terminated(
        pair(
            separated_pair(parse_reactants, tag("->"), parse_reactants),
            opt(delimited(pair(tag(":"), multispace0), double, multispace0)),
        ),
        tag(";"),
    )(input)
}

/// Parse multiple reactions.
fn parse_reactions(input: &str) -> IResult<&str, Vec<ReactionTokens>> {
    many0(parse_reaction)(input)
}

impl<T> Crn<T>
where
    T: Default + std::clone::Clone + std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    /// Parse a CRN from a string.
    pub fn parse(input: &str) -> Result<Crn<T>, ParseError> {
        let (leftover_input, counts) = parse_counts(input).unwrap();
        let mut species_map: HashMap<&str, usize> = HashMap::new();
        let mut names = bimap::BiHashMap::<usize, String>::new();
        let mut x = Vec::<T>::with_capacity(counts.len());
        for (i, (species, num)) in counts.iter().enumerate() {
            if species_map.contains_key(species) {
                return Err(ParseError::DuplicateDefinition(species.to_string()));
            } else {
                species_map.insert(species, i);
                names.insert(i, species.to_string());
                x.push(num.parse::<T>().unwrap());
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
                    x.push(T::default());
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
                    x.push(T::default());
                    product_map.insert(len, num);
                } else {
                    product_map.insert(species_map[species], num);
                }
            }
            let rxn = Reaction::new(reactant_map, product_map, rate.unwrap_or(1.0));
            rxns.push(rxn);
        }

        let state = State {
            species: x,
            time: 0.0,
        };
        Ok(Self {
            init_state: state.clone(),
            rxns,
            state,
            names,
        })
    }
}

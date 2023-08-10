use nom::{
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric0, digit0, multispace0},
    combinator::{opt, recognize},
    multi::{many0, separated_list0},
    number::complete::double,
    sequence::{delimited, pair, separated_pair, terminated},
    IResult,
};

#[derive(Debug, Clone)]
pub enum ParseError {
    DuplicateSpecies(String),
    DuplicateReaction,
    DuplicateDefinition(String),
    InvalidSpecies,
    InvalidReaction,
    InvalidDefinition,
}

fn species_name(input: &str) -> IResult<&str, &str> {
    delimited(
        multispace0,
        recognize(pair(alpha1, alphanumeric0)),
        multispace0,
    )(input)
}

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

pub fn parse_counts(input: &str) -> IResult<&str, Vec<(&str, &str)>> {
    many0(parse_count)(input)
}

fn parse_reactant(input: &str) -> IResult<&str, (&str, &str)> {
    delimited(multispace0, pair(digit0, species_name), multispace0)(input)
}

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

type ReactionTokens<'a> = (
    (Vec<(&'a str, &'a str)>, Vec<(&'a str, &'a str)>),
    Option<f64>,
);

fn parse_reaction(input: &str) -> IResult<&str, ReactionTokens> {
    terminated(
        pair(
            separated_pair(parse_reactants, tag("->"), parse_reactants),
            opt(delimited(pair(tag(":"), multispace0), double, multispace0)),
        ),
        tag(";"),
    )(input)
}

pub fn parse_reactions(input: &str) -> IResult<&str, Vec<ReactionTokens>> {
    many0(parse_reaction)(input)
}

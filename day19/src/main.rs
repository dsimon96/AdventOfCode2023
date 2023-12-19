use std::{
    collections::HashMap,
    io::{stdin, BufRead, Lines},
};

use anyhow::Result;
use clap::{Parser, Subcommand};
use enum_map::{enum_map, Enum, EnumMap};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, char, digit1, one_of},
    combinator::map_res,
    multi::separated_list1,
    sequence::delimited,
    IResult,
};
use thiserror::Error;

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    part: Part,
}

#[derive(PartialEq, Eq, Subcommand)]
enum Part {
    Part1,
    Part2,
}

#[derive(Clone, Copy, Enum)]
enum Category {
    X,
    M,
    A,
    S,
}

#[derive(Debug, Error)]
#[error("`{0}` is not a valid category")]
struct ParseCategoryError(char);

impl TryFrom<char> for Category {
    type Error = ParseCategoryError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'x' => Ok(Category::X),
            'm' => Ok(Category::M),
            'a' => Ok(Category::A),
            's' => Ok(Category::S),
            _ => Err(ParseCategoryError(value)),
        }
    }
}

fn category(input: &str) -> IResult<&str, Category> {
    map_res(one_of("xmas"), Category::try_from)(input)
}

enum ComparisonType {
    Greater,
    Less,
}

#[derive(Debug, Error)]
#[error("`{0}` is not a valid comparison type")]
struct ParseComparisonTypeError(char);

impl TryFrom<char> for ComparisonType {
    type Error = ParseComparisonTypeError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '>' => Ok(ComparisonType::Greater),
            '<' => Ok(ComparisonType::Less),
            _ => Err(ParseComparisonTypeError(value)),
        }
    }
}

fn comparison_type(input: &str) -> IResult<&str, ComparisonType> {
    map_res(one_of("<>"), ComparisonType::try_from)(input)
}

type Value = usize;

fn value(input: &str) -> IResult<&str, usize> {
    map_res(digit1, str::parse)(input)
}

enum Rule {
    Comparison {
        category: Category,
        t: ComparisonType,
        v: Value,
        dest: String,
    },
    Default {
        dest: String,
    },
}

fn comparison_rule(input: &str) -> IResult<&str, Rule> {
    let (input, category) = category(input)?;
    let (input, t) = comparison_type(input)?;
    let (input, v) = value(input)?;
    let (input, _) = char(':')(input)?;
    let (input, dest) = alpha1(input)?;

    Ok((
        input,
        Rule::Comparison {
            category,
            t,
            v,
            dest: dest.to_owned(),
        },
    ))
}

fn default_rule(input: &str) -> IResult<&str, Rule> {
    let (input, dest) = alpha1(input)?;

    Ok((
        input,
        Rule::Default {
            dest: dest.to_owned(),
        },
    ))
}

fn rule(input: &str) -> IResult<&str, Rule> {
    alt((comparison_rule, default_rule))(input)
}

type Workflow = Vec<Rule>;

fn workflow(input: &str) -> IResult<&str, (String, Workflow)> {
    let (input, name) = alpha1(input)?;
    let (input, workflow) =
        delimited(char('{'), separated_list1(char(','), rule), char('}'))(input)?;

    Ok((input, (name.into(), workflow)))
}

type PartRatings = EnumMap<Category, Value>;

fn part_ratings(input: &str) -> IResult<&str, PartRatings> {
    let (input, _) = tag("{x=")(input)?;
    let (input, x) = value(input)?;
    let (input, _) = tag(",m=")(input)?;
    let (input, m) = value(input)?;
    let (input, _) = tag(",a=")(input)?;
    let (input, a) = value(input)?;
    let (input, _) = tag(",s=")(input)?;
    let (input, s) = value(input)?;
    let (input, _) = char('}')(input)?;

    Ok((
        input,
        enum_map! { Category::X => x, Category::M => m, Category::A => a, Category::S => s},
    ))
}

const ACCEPT: &str = "A";
const REJECT: &str = "R";
const TERMINAL_LABELS: &[&str] = &[ACCEPT, REJECT];
const INIT_LABEL: &str = "in";

fn parse_workflows(inp: &mut Lines<impl BufRead>) -> Result<HashMap<String, Workflow>> {
    let mut workflows = HashMap::new();

    while let Some(line) = inp.next() {
        let line = line?;
        if line.is_empty() {
            break;
        }
        let (_, (name, workflow)) = workflow(&line).map_err(|e| e.to_owned())?;

        workflows.insert(name, workflow);
    }

    Ok(workflows)
}

fn parse_parts(inp: &mut Lines<impl BufRead>) -> Result<Vec<PartRatings>> {
    let mut parts = Vec::new();
    while let Some(line) = inp.next() {
        let line = line?;
        let (_, part) = part_ratings(&line).map_err(|e| e.to_owned())?;
        parts.push(part);
    }

    Ok(parts)
}

fn process_one<'a>(workflow: &'a Workflow, part: &PartRatings) -> &'a str {
    for rule in workflow {
        match rule {
            Rule::Comparison {
                category,
                t: ComparisonType::Less,
                v,
                dest,
            } => {
                if part[*category] < *v {
                    return dest;
                }
            }
            Rule::Comparison {
                category,
                t: ComparisonType::Greater,
                v,
                dest,
            } => {
                if part[*category] > *v {
                    return dest;
                }
            }
            Rule::Default { dest } => return dest,
        }
    }

    unreachable!("Should have encountered a default rule")
}

fn process(
    workflows: HashMap<String, Workflow>,
    parts: Vec<PartRatings>,
) -> (Vec<PartRatings>, Vec<PartRatings>) {
    let mut labeled: Vec<(&str, PartRatings)> =
        parts.into_iter().map(|part| (INIT_LABEL, part)).collect();

    while !labeled
        .iter()
        .all(|(label, _)| TERMINAL_LABELS.contains(label))
    {
        labeled = labeled
            .into_iter()
            .map(|(label, part)| {
                if TERMINAL_LABELS.contains(&label) {
                    return (label, part);
                }
                let workflow = workflows.get(label).expect("Invalid label!");
                (process_one(workflow, &part), part)
            })
            .collect();
    }

    let (accept, reject): (Vec<_>, Vec<_>) =
        labeled.into_iter().partition(|(label, _)| *label == ACCEPT);

    (
        accept.into_iter().map(|(_, part)| part).collect(),
        reject.into_iter().map(|(_, part)| part).collect(),
    )
}

fn main() -> Result<()> {
    let _args = Args::parse();

    let mut inp = stdin().lines();

    let workflows = parse_workflows(&mut inp)?;
    let parts = parse_parts(&mut inp)?;

    let (accept, _) = process(workflows, parts);

    let res = accept
        .into_iter()
        .map(|part| part.values().sum::<Value>())
        .sum::<Value>();

    println!("{res}");

    Ok(())
}

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

#[derive(Debug, Clone, Copy, Enum, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug)]
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

type Workflows = HashMap<String, Workflow>;

fn parse_workflows(inp: &mut Lines<impl BufRead>) -> Result<Workflows> {
    let mut workflows = Workflows::new();

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

fn process(workflows: Workflows, parts: Vec<PartRatings>) -> (Vec<PartRatings>, Vec<PartRatings>) {
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

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
struct Interval {
    lower_bound_incl: Value,
    upper_bound_excl: Value,
}

const MIN_VAL: Value = 1;
const MAX_VAL: Value = 4001;

impl Default for Interval {
    fn default() -> Self {
        Self {
            lower_bound_incl: MIN_VAL,
            upper_bound_excl: MAX_VAL,
        }
    }
}

impl Interval {
    fn count(&self) -> Value {
        self.upper_bound_excl.saturating_sub(self.lower_bound_incl)
    }

    fn refine_if(&self, t: ComparisonType, v: Value) -> Self {
        match t {
            ComparisonType::Greater => Self {
                lower_bound_incl: v + 1,
                upper_bound_excl: self.upper_bound_excl,
            },
            ComparisonType::Less => Self {
                lower_bound_incl: self.lower_bound_incl,
                upper_bound_excl: v,
            },
        }
    }

    fn refine_else(&self, t: ComparisonType, v: Value) -> Self {
        let (t, v) = match t {
            ComparisonType::Greater => (ComparisonType::Less, v + 1),
            ComparisonType::Less => (ComparisonType::Greater, v - 1),
        };
        self.refine_if(t, v)
    }
}

#[derive(Debug, Clone)]
struct Accepted {
    map: EnumMap<Category, Interval>,
}

impl Accepted {
    fn all() -> Self {
        Self {
            map: enum_map! {
                _ => Interval { lower_bound_incl: MIN_VAL, upper_bound_excl: MAX_VAL }
            },
        }
    }

    fn none() -> Self {
        Self {
            map: enum_map! {
                _ => Interval { lower_bound_incl: MIN_VAL, upper_bound_excl: MIN_VAL }
            },
        }
    }

    fn count(&self) -> Value {
        self.map
            .values()
            .map(|intervals| intervals.count())
            .product()
    }

    fn split_comparison(&self, category: Category, t: ComparisonType, v: usize) -> (Self, Self) {
        let if_case = Self {
            map: enum_map! {
                c => if c == category { self.map[c].refine_if(t, v) } else {self.map[c]}
            },
        };

        let else_case = Self {
            map: enum_map! {
                c => if c == category { self.map[c].refine_else(t, v) } else {self.map[c]}
            },
        };

        (if_case, else_case)
    }
}

fn determine_accepted(workflows: &Workflows, s: &String, mut prior: Accepted) -> Vec<Accepted> {
    if *s == ACCEPT.to_string() {
        return vec![prior];
    } else if *s == REJECT.to_string() {
        return Vec::new();
    }
    let workflow = workflows.get(s).expect("Invalid workflow name");

    let mut res = Vec::new();
    for rule in workflow {
        let ((if_case, else_case), dep) = match rule {
            Rule::Comparison {
                category,
                t,
                v,
                dest,
            } => (prior.split_comparison(*category, *t, *v), dest),
            Rule::Default { dest } => ((prior, Accepted::none()), dest),
        };
        res.extend(determine_accepted(workflows, dep, if_case).into_iter());
        prior = else_case;
    }

    res
}

fn count_accepted(workflows: &Workflows) -> Value {
    let accepted = determine_accepted(workflows, &INIT_LABEL.to_string(), Accepted::all());

    accepted.into_iter().map(|accepted| accepted.count()).sum()
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut inp = stdin().lines();

    let workflows = parse_workflows(&mut inp)?;
    let res = match args.part {
        Part::Part1 => {
            let parts = parse_parts(&mut inp)?;
            let (accept, _) = process(workflows, parts);

            accept
                .into_iter()
                .map(|part| part.values().sum::<Value>())
                .sum::<Value>()
        }
        Part::Part2 => count_accepted(&workflows),
    };

    println!("{res}");

    Ok(())
}

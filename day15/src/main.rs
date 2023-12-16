use std::{collections::HashMap, io::stdin};

use anyhow::{ensure, Result};
use clap::{Parser, Subcommand};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, digit1},
    combinator::{map, map_res, recognize},
    sequence::{separated_pair, terminated},
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

#[derive(Debug, Error)]
enum InvalidInputError {
    #[error("Input contains non-ascii characters")]
    NonAscii,
}

fn hash(s: &str) -> Result<u8> {
    ensure!(s.is_ascii(), InvalidInputError::NonAscii);

    Ok(s.chars()
        .fold(0, |s, c| ((s as u32 + c as u32) * 17 % 256) as u8))
}

enum Operation {
    Set(u32),
    Delete,
}

struct Step {
    label: String,
    op: Operation,
}

fn num(input: &str) -> IResult<&str, u32> {
    map_res(recognize(digit1), str::parse)(input)
}

fn set_step(input: &str) -> IResult<&str, Step> {
    map(
        separated_pair(alphanumeric1, tag("="), num),
        |(label, val)| Step {
            label: label.to_owned(),
            op: Operation::Set(val),
        },
    )(input)
}

fn delete_step(input: &str) -> IResult<&str, Step> {
    map(terminated(alphanumeric1, tag("-")), |label: &str| Step {
        label: label.to_owned(),
        op: Operation::Delete,
    })(input)
}

fn step(input: &str) -> IResult<&str, Step> {
    alt((set_step, delete_step))(input)
}

fn main() -> Result<()> {
    let args = Args::parse();

    let res = match args.part {
        Part::Part1 => stdin()
            .lines()
            .map(|line| {
                line?
                    .split(',')
                    .map(hash)
                    .map(|x| x.map(u32::from))
                    .sum::<Result<u32>>()
            })
            .sum::<Result<u32>>()?,
        Part::Part2 => {
            let mut hashmap: HashMap<u8, Vec<(String, u32)>> = HashMap::new();
            for line in stdin().lines() {
                let line = line?;
                for token in line.split(',') {
                    let (_, step) = step(token).map_err(|e| e.to_owned())?;
                    let i = hash(&step.label)?;
                    let inner = hashmap.entry(i).or_default();
                    match step.op {
                        Operation::Set(val) => {
                            let mut found = false;
                            for (label, contents) in inner.iter_mut() {
                                if *label == step.label {
                                    *contents = val;
                                    found = true;
                                    break;
                                }
                            }
                            if !found {
                                inner.push((step.label, val));
                            }
                        }
                        Operation::Delete => inner.retain(|(label, _)| *label != step.label),
                    };
                }
            }

            hashmap
                .iter()
                .map(|(&i, inner)| {
                    inner
                        .iter()
                        .enumerate()
                        .map(|(j, &(_, contents))| (i as u32 + 1) * (j as u32 + 1) * contents)
                        .sum::<u32>()
                })
                .sum()
        }
    };

    println!("{res}");

    Ok(())
}

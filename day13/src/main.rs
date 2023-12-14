use std::{io::stdin, iter::from_fn};

use anyhow::Result;
use clap::{Parser, Subcommand};
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PatternSpace {
    Ash,
    Rocks,
}

#[derive(Debug, Error)]
#[error("`{0}` is an invalid grid space")]
struct ParsePatternError(char);

impl TryFrom<char> for PatternSpace {
    type Error = ParsePatternError;

    fn try_from(value: char) -> std::prelude::v1::Result<Self, Self::Error> {
        match value {
            '.' => Ok(PatternSpace::Ash),
            '#' => Ok(PatternSpace::Rocks),
            c => Err(ParsePatternError(c)),
        }
    }
}

type PatternRow = Vec<PatternSpace>;
type Pattern = Vec<PatternRow>;

fn patterns() -> impl Iterator<Item = Pattern> {
    let mut inp = stdin().lines();

    from_fn(move || {
        let mut cur = Pattern::new();
        while let Some(line) = inp.next() {
            let line = line.expect("Error reading input");
            if line.is_empty() {
                break;
            };

            cur.push(
                line.chars()
                    .map(|c| PatternSpace::try_from(c).expect("Error parsing input"))
                    .collect::<Vec<_>>(),
            );
        }

        if cur.is_empty() {
            return None;
        }
        Some(cur)
    })
}

fn summarize(patterns: impl Iterator<Item = Pattern>, part: Part) -> usize {
    let mut res = 0;
    for pattern in patterns {
        let rows = pattern.len();
        let cols = pattern.get(0).expect("Zero length Pattern").len();

        for i in 1..cols {
            let num_different = pattern
                .iter()
                .flat_map(|row| {
                    row[..i]
                        .iter()
                        .rev()
                        .zip(row[i..].iter())
                        .filter(|(x, y)| x != y)
                })
                .count();
            if (part == Part::Part1 && num_different == 0)
                || (part == Part::Part2 && num_different == 1)
            {
                res += i
            }
        }

        for i in 1..rows {
            let num_different = pattern[..i]
                .iter()
                .rev()
                .zip(pattern[i..].iter())
                .flat_map(|(rx, ry)| rx.iter().zip(ry.iter()).filter(|(x, y)| x != y))
                .count();
            if (part == Part::Part1 && num_different == 0)
                || (part == Part::Part2 && num_different == 1)
            {
                res += 100 * i
            }
        }
    }

    res
}

fn main() -> Result<()> {
    let args = Args::parse();

    let res = summarize(patterns(), args.part);
    println!("{res}");

    Ok(())
}

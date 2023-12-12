use std::{
    collections::HashSet,
    io::{stdin, Stdin},
};

use anyhow::Result;
use clap::{Parser, Subcommand};
use itertools::Itertools;
use thiserror::Error;

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    part: Part,
}

#[derive(Subcommand)]
enum Part {
    Part1,
    Part2,
}

#[derive(PartialEq, Eq)]
enum GridSpace {
    Empty,
    Galaxy,
}

#[derive(Debug, Error)]
#[error("`{0}` is not a valid grid space")]
struct InvalidCharError(char);

impl TryFrom<char> for GridSpace {
    type Error = InvalidCharError;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '.' => Ok(GridSpace::Empty),
            '#' => Ok(GridSpace::Galaxy),
            c => Err(InvalidCharError(c)),
        }
    }
}

type GridRow = Vec<GridSpace>;

fn grid_row(line: &str) -> Result<GridRow, InvalidCharError> {
    line.chars().map(GridSpace::try_from).collect()
}

type Grid = Vec<GridRow>;

fn grid(inp: Stdin) -> Result<Grid> {
    let mut res = Vec::new();
    for line in inp.lines() {
        let line = line?;
        res.push(grid_row(&line)?);
    }

    Ok(res)
}

fn main() -> Result<()> {
    let args = Args::parse();

    let grid = grid(stdin())?;
    let empty_rows = grid
        .iter()
        .enumerate()
        .filter_map(|(r, row)| {
            if row.iter().all(|space| *space == GridSpace::Empty) {
                Some(r)
            } else {
                None
            }
        })
        .collect::<HashSet<_>>();

    let empty_cols = (0..grid[0].len())
        .filter_map(|c| {
            if grid.iter().all(|row| row[c] == GridSpace::Empty) {
                Some(c)
            } else {
                None
            }
        })
        .collect::<HashSet<_>>();

    let galaxies = grid
        .iter()
        .enumerate()
        .flat_map(|(r, row)| {
            row.iter()
                .enumerate()
                .filter_map(|(c, space)| {
                    if *space == GridSpace::Galaxy {
                        Some((r, c))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let multiplier = match args.part {
        Part::Part1 => 2usize,
        Part::Part2 => 1000000usize,
    };

    let mut res: usize = 0;
    for (&(r0, c0), &(r1, c1)) in galaxies.iter().tuple_combinations() {
        let row_dist: usize = (r0.min(r1)..r0.max(r1))
            .map(|r| {
                if empty_rows.contains(&r) {
                    multiplier
                } else {
                    1
                }
            })
            .sum();

        let col_dist: usize = (c0.min(c1)..c0.max(c1))
            .map(|c| {
                if empty_cols.contains(&c) {
                    multiplier
                } else {
                    1
                }
            })
            .sum();

        res += row_dist + col_dist;
    }

    println!("{res}");

    Ok(())
}

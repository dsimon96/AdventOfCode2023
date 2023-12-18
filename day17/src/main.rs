use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
    io::{stdin, BufRead},
};

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

#[derive(Debug, Error)]
#[error("Non-numeric input")]
struct ParseValError;

type Row = Vec<u32>;

fn row(inp: &str) -> Result<Row> {
    inp.chars()
        .map(|c| Ok(c.to_digit(10).ok_or(ParseValError)?))
        .collect()
}

type Grid = Vec<Row>;

fn grid(inp: impl BufRead) -> Result<Grid> {
    inp.lines().map(|l| row(&l?)).collect()
}

type Coords = (usize, usize);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    fn turn_left(&self) -> Self {
        match self {
            Direction::North => Direction::West,
            Direction::South => Direction::East,
            Direction::East => Direction::North,
            Direction::West => Direction::South,
        }
    }

    fn turn_right(&self) -> Self {
        match self {
            Direction::North => Direction::East,
            Direction::South => Direction::West,
            Direction::East => Direction::South,
            Direction::West => Direction::North,
        }
    }
}

fn try_move(grid: &Grid, (r, c): &Coords, direction: &Direction) -> Option<Coords> {
    match direction {
        Direction::North if *r > 0 => Some((r - 1, *c)),
        Direction::South if *r < grid.len() - 1 => Some((r + 1, *c)),
        Direction::East if *c < grid[0].len() - 1 => Some((*r, c + 1)),
        Direction::West if *c > 0 => Some((*r, c - 1)),
        _ => None,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct State {
    coords: Coords,
    direction: Direction,
    run_length: usize,
}

fn heat_loss(grid: &Grid, min_run_length: usize, max_run_length: usize) -> u32 {
    let mut heap = BinaryHeap::new();
    let mut best: HashMap<State, u32> = HashMap::new();

    let state = State {
        coords: (0, 0),
        direction: Direction::East,
        run_length: 0,
    };
    best.insert(state, 0);
    heap.push(Reverse((0, state)));

    while let Some(Reverse((heat_loss, state))) = heap.pop() {
        if state.coords == (grid.len() - 1, grid[0].len() - 1) && state.run_length >= min_run_length
        {
            return heat_loss;
        } else if heat_loss > *best.get(&state).unwrap() {
            continue;
        }

        let mut possible = Vec::new();
        if state.run_length == 0 || state.run_length >= min_run_length {
            possible.push((state.direction.turn_left(), 1));
            possible.push((state.direction.turn_right(), 1));
        }
        if state.run_length < max_run_length {
            possible.push((state.direction, state.run_length + 1));
        }

        for (direction, run_length) in possible {
            let Some(coords) = try_move(grid, &state.coords, &direction) else {
                continue;
            };

            let heat_loss = heat_loss + grid[coords.0][coords.1];
            let state = State {
                coords,
                direction,
                run_length,
            };

            let best_for_state = best.entry(state).or_insert(u32::MAX);
            if heat_loss < *best_for_state {
                heap.push(Reverse((heat_loss, state)));
                *best_for_state = heat_loss;
            }
        }
    }

    unreachable!()
}

fn main() -> Result<()> {
    let args = Args::parse();

    let grid = grid(stdin().lock())?;
    let res = match args.part {
        Part::Part1 => heat_loss(&grid, 0, 3),
        Part::Part2 => heat_loss(&grid, 4, 10),
    };
    println!("{res}");

    Ok(())
}

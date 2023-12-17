use std::{
    collections::{HashSet, VecDeque},
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

enum Space {
    Empty,
    MirrorUpLeft,
    MirrorUpRight,
    SplitterHorizontal,
    SplitterVertical,
}

#[derive(Debug, Error)]
#[error("`{0}` is an invalid space")]
struct ParseSpaceError(char);

impl TryFrom<char> for Space {
    type Error = ParseSpaceError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '.' => Ok(Space::Empty),
            '\\' => Ok(Space::MirrorUpLeft),
            '/' => Ok(Space::MirrorUpRight),
            '-' => Ok(Space::SplitterHorizontal),
            '|' => Ok(Space::SplitterVertical),
            _ => Err(ParseSpaceError(value)),
        }
    }
}

type Row = Vec<Space>;

fn row(s: &str) -> Result<Row, ParseSpaceError> {
    s.chars().map(Space::try_from).collect()
}

type Grid = Vec<Row>;

fn grid(inp: impl BufRead) -> Result<Grid> {
    inp.lines().map(|line| Ok(row(&line?)?)).collect()
}

type Coords = (usize, usize);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

type State = (Coords, Direction);

fn successor_directions(space: &Space, direction: &Direction) -> Vec<Direction> {
    match (space, direction) {
        (Space::Empty, _) => Vec::from([*direction]),
        (Space::MirrorUpLeft, Direction::Up) => Vec::from([Direction::Left]),
        (Space::MirrorUpLeft, Direction::Down) => Vec::from([Direction::Right]),
        (Space::MirrorUpLeft, Direction::Left) => Vec::from([Direction::Up]),
        (Space::MirrorUpLeft, Direction::Right) => Vec::from([Direction::Down]),
        (Space::MirrorUpRight, Direction::Up) => Vec::from([Direction::Right]),
        (Space::MirrorUpRight, Direction::Down) => Vec::from([Direction::Left]),
        (Space::MirrorUpRight, Direction::Left) => Vec::from([Direction::Down]),
        (Space::MirrorUpRight, Direction::Right) => Vec::from([Direction::Up]),
        (Space::SplitterHorizontal, Direction::Up | Direction::Down) => {
            Vec::from([Direction::Left, Direction::Right])
        }
        (Space::SplitterHorizontal, Direction::Left | Direction::Right) => Vec::from([*direction]),
        (Space::SplitterVertical, Direction::Up | Direction::Down) => Vec::from([*direction]),
        (Space::SplitterVertical, Direction::Left | Direction::Right) => {
            Vec::from([Direction::Up, Direction::Down])
        }
    }
}

fn try_move(grid: &Grid, (r, c): Coords, direction: &Direction) -> Option<Coords> {
    match direction {
        Direction::Up if r > 0 => Some((r - 1, c)),
        Direction::Down if r < grid.len() - 1 => Some((r + 1, c)),
        Direction::Left if c > 0 => Some((r, c - 1)),
        Direction::Right if c < grid[0].len() - 1 => Some((r, c + 1)),
        _ => None,
    }
}

fn count_energized(grid: &Grid, init_state: State) -> usize {
    let mut queue = VecDeque::from([init_state]);
    let mut visited = HashSet::from([init_state]);
    let mut energized: HashSet<Coords> = HashSet::new();

    while let Some((coords, direction)) = queue.pop_front() {
        energized.insert(coords);
        for direction in successor_directions(&grid[coords.0][coords.1], &direction) {
            let Some(new_coords) = try_move(&grid, coords, &direction) else {
                continue;
            };

            let new_state: State = (new_coords, direction);
            if visited.contains(&new_state) {
                continue;
            }

            visited.insert(new_state);
            queue.push_back(new_state);
        }
    }

    energized.len()
}

fn main() -> Result<()> {
    let args = Args::parse();

    let grid = grid(stdin().lock())?;

    let res = match args.part {
        Part::Part1 => count_energized(&grid, ((0, 0), Direction::Right)),
        Part::Part2 => (0..grid.len())
            .flat_map(|r| {
                [
                    ((r, 0), Direction::Right),
                    ((r, grid[0].len() - 1), Direction::Left),
                ]
            })
            .chain((0..grid[0].len()).flat_map(|c| {
                [
                    ((0, c), Direction::Down),
                    ((grid.len() - 1, c), Direction::Up),
                ]
            }))
            .map(|init_state| count_energized(&grid, init_state))
            .max()
            .unwrap(),
    };

    println!("{res}");

    Ok(())
}

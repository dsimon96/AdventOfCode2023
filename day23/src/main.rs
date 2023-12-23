use std::{
    collections::VecDeque,
    io::{stdin, BufRead},
    ops::Index,
};

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
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

#[derive(Debug, Clone, Copy)]
enum Direction {
    N,
    E,
    S,
    W,
}

const ALL_DIRECTIONS: &[Direction] = &[Direction::N, Direction::E, Direction::S, Direction::W];

type Coord = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Coords {
    r: Coord,
    c: Coord,
}

#[derive(Debug)]
enum Space {
    Empty,
    Forest,
    Slope(Direction),
}

impl Space {
    fn available_directions(&self) -> &'static [Direction] {
        match self {
            Space::Empty => ALL_DIRECTIONS,
            Space::Forest => unreachable!(),
            Space::Slope(Direction::N) => &[Direction::N],
            Space::Slope(Direction::E) => &[Direction::E],
            Space::Slope(Direction::S) => &[Direction::S],
            Space::Slope(Direction::W) => &[Direction::W],
        }
    }
}

#[derive(Debug, Error)]
#[error("`{0}` is an invalid Space")]
struct ParseSpaceError(char);

impl TryFrom<char> for Space {
    type Error = ParseSpaceError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '.' => Ok(Space::Empty),
            '#' => Ok(Space::Forest),
            '^' => Ok(Space::Slope(Direction::N)),
            'v' => Ok(Space::Slope(Direction::S)),
            '>' => Ok(Space::Slope(Direction::E)),
            '<' => Ok(Space::Slope(Direction::W)),
            _ => Err(ParseSpaceError(value)),
        }
    }
}

#[derive(Debug)]
struct Input {
    map: Vec<Vec<Space>>,
}

impl Input {
    fn height(&self) -> usize {
        self.map.len()
    }

    fn width(&self) -> usize {
        self.map[0].len()
    }
}

impl Index<Coords> for Input {
    type Output = Space;

    fn index(&self, index: Coords) -> &Self::Output {
        &self.map[index.r][index.c]
    }
}

fn parse_row(inp: &str) -> Result<Vec<Space>, ParseSpaceError> {
    inp.chars().map(Space::try_from).collect()
}

fn parse_input(inp: impl BufRead) -> Result<Input> {
    let mut map = Vec::new();

    for line in inp.lines() {
        map.push(parse_row(&line?)?);
    }

    Ok(Input { map })
}

fn find_only_empty(row: &Vec<Space>) -> Result<usize> {
    let empty_spaces: Vec<Coord> = row
        .iter()
        .enumerate()
        .filter_map(|(c, space)| {
            if let Space::Empty = space {
                Some(c)
            } else {
                None
            }
        })
        .collect();

    let [space] = empty_spaces[..] else {
        bail!("There must be exactly one empty space");
    };
    Ok(space)
}

fn try_move(input: &Input, coords: Coords, dir: Direction) -> Option<Coords> {
    match dir {
        Direction::N if coords.r > 0 => Some(Coords {
            r: coords.r - 1,
            ..coords
        }),
        Direction::E if coords.c < input.width() - 1 => Some(Coords {
            c: coords.c + 1,
            ..coords
        }),
        Direction::S if coords.r < input.height() - 1 => Some(Coords {
            r: coords.r + 1,
            ..coords
        }),
        Direction::W if coords.c > 0 => Some(Coords {
            c: coords.c - 1,
            ..coords
        }),
        _ => None,
    }
}

fn find_longest_path(input: &Input, start: Coords, end: Coords) -> Option<usize> {
    let mut paths = VecDeque::from([vec![start]]);

    let mut hike_lengths = Vec::new();

    while let Some(path) = paths.pop_front() {
        let cur = *path.last().expect("Path should not be empty");
        if cur == end {
            hike_lengths.push(path.len() - 1);
            continue;
        }

        for &dir in input[cur].available_directions() {
            let Some(next) = try_move(input, cur, dir) else {
                continue;
            };
            if let Space::Forest = input[next] {
                continue;
            }
            if path.contains(&next) {
                continue;
            }
            let mut new_path = path.clone();
            new_path.push(next);
            paths.push_front(new_path);
        }
    }

    hike_lengths.into_iter().max()
}

fn main() -> Result<()> {
    let args = Args::parse();

    let input = parse_input(stdin().lock())?;

    let start = Coords {
        r: 0,
        c: find_only_empty(&input.map[0]).context("Couldn't find start space")?,
    };

    let end = Coords {
        r: input.map.len() - 1,
        c: find_only_empty(input.map.last().context("Empty input")?)
            .context("Couldn't find end space")?,
    };

    let res = match args.part {
        Part::Part1 => find_longest_path(&input, start, end).expect("No path found"),
        Part::Part2 => todo!(),
    };

    println!("{res}");

    Ok(())
}

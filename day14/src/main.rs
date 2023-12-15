use std::{
    collections::{hash_map::Entry, HashMap},
    io::{stdin, BufRead},
    mem::swap,
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
    Part2 {
        #[arg(default_value_t = 1000000000)]
        cycles: usize,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Space {
    RoundedRock,
    CubeShapedRock,
    Empty,
}

#[derive(Debug, Error)]
#[error("`{0}` is an invalid Space")]
struct ParseSpaceError(char);

impl TryFrom<char> for Space {
    type Error = ParseSpaceError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'O' => Ok(Self::RoundedRock),
            '#' => Ok(Self::CubeShapedRock),
            '.' => Ok(Self::Empty),
            _ => Err(ParseSpaceError(value)),
        }
    }
}

type Row = Vec<Space>;

fn row(line: &str) -> Result<Row, ParseSpaceError> {
    line.chars().map(Space::try_from).collect()
}

type Grid = Vec<Row>;

fn grid(inp: impl BufRead) -> Result<Grid> {
    inp.lines()
        .into_iter()
        .map(|line| Ok(row(&line?)?))
        .collect()
}

type Coords = (usize, usize);

fn all_rounded_rocks(grid: &Grid) -> Vec<Coords> {
    grid.iter()
        .enumerate()
        .flat_map(|(i, row)| {
            row.iter().enumerate().filter_map(move |(j, space)| {
                if let Space::RoundedRock = space {
                    Some((i, j))
                } else {
                    None
                }
            })
        })
        .collect()
}

#[derive(Clone, Copy)]
enum Direction {
    North,
    South,
    East,
    West,
}

fn get_coord(grid: &Grid, (r, c): Coords, direction: Direction) -> Option<Coords> {
    match direction {
        Direction::North => {
            if r == 0 {
                None
            } else {
                Some((r - 1, c))
            }
        }
        Direction::South => {
            if r + 1 == grid.len() {
                None
            } else {
                Some((r + 1, c))
            }
        }
        Direction::East => {
            if c + 1 == grid[0].len() {
                None
            } else {
                Some((r, c + 1))
            }
        }
        Direction::West => {
            if c == 0 {
                None
            } else {
                Some((r, c - 1))
            }
        }
    }
}

fn do_swap(grid: &mut Grid, (r, c): Coords, (nr, nc): Coords) {
    if c == nc {
        let (x, y) = grid.split_at_mut(r.max(nr));
        swap(&mut x[r.min(nr)][c], &mut y[0][c]);
    } else if r == nr {
        let (x, y) = grid[r].split_at_mut(c.max(nc));
        swap(&mut x[c.min(nc)], &mut y[0]);
    } else {
        unreachable!()
    }
}

fn roll(mut grid: Grid, direction: Direction) -> Grid {
    let mut rocks = all_rounded_rocks(&grid);
    rocks.sort_by_key(|&(row, col)| match direction {
        Direction::North => row,
        Direction::South => grid.len() - row,
        Direction::East => grid[0].len() - col,
        Direction::West => col,
    });

    for original_coords in rocks {
        let mut coords = original_coords;
        while let Some(new_coords) = get_coord(&grid, coords, direction) {
            let Space::Empty = grid[new_coords.0][new_coords.1] else {
                break;
            };
            coords = new_coords;
        }
        if coords != original_coords {
            do_swap(&mut grid, original_coords, coords);
        }
    }

    grid
}

fn get_total_load(grid: &Grid) -> usize {
    grid.iter()
        .enumerate()
        .map(|(i, row)| {
            (grid.len() - i)
                * row
                    .iter()
                    .filter(|space| {
                        if let Space::RoundedRock = space {
                            true
                        } else {
                            false
                        }
                    })
                    .count()
        })
        .sum()
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut grid = grid(stdin().lock())?;

    match args.part {
        Part::Part1 => {
            grid = roll(grid, Direction::North);
        }
        Part::Part2 { cycles } => {
            let mut i = 0;
            let mut seen: HashMap<Grid, usize> = HashMap::new();
            seen.insert(grid.to_owned(), 0);

            let cycle_length = loop {
                grid = roll(grid, Direction::North);
                grid = roll(grid, Direction::West);
                grid = roll(grid, Direction::South);
                grid = roll(grid, Direction::East);
                i += 1;

                match seen.entry(grid.to_owned()) {
                    Entry::Occupied(e) => {
                        break i - e.get();
                    }
                    Entry::Vacant(e) => e.insert(i),
                };
            };
            while i + cycle_length <= cycles {
                i += cycle_length;
            }
            while i < cycles {
                grid = roll(grid, Direction::North);
                grid = roll(grid, Direction::West);
                grid = roll(grid, Direction::South);
                grid = roll(grid, Direction::East);
                i += 1;
            }
        }
    }

    println!("{}", get_total_load(&grid));

    Ok(())
}

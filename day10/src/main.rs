use std::{
    collections::{HashMap, VecDeque},
    io::{stdin, Stdin},
};

use anyhow::Result;
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

#[derive(Clone, Copy, PartialEq)]
enum Direction {
    North,
    South,
    West,
    East,
}

impl Direction {
    fn is_vertical(&self) -> bool {
        match self {
            Direction::North => true,
            Direction::South => true,
            Direction::West => false,
            Direction::East => false,
        }
    }
}

impl Direction {
    fn opposite(&self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
            Direction::East => Direction::West,
        }
    }
}

#[derive(Debug, PartialEq)]
enum GridSpace {
    VerticalPipe,
    HorizontalPipe,
    NorthEastBend,
    NorthWestBend,
    SouthWestBend,
    SouthEastBend,
    Ground,
    Start,
}

impl GridSpace {
    fn possible_connections(&self) -> Vec<Direction> {
        match self {
            GridSpace::VerticalPipe => vec![Direction::North, Direction::South],
            GridSpace::HorizontalPipe => vec![Direction::West, Direction::East],
            GridSpace::NorthEastBend => vec![Direction::North, Direction::East],
            GridSpace::NorthWestBend => vec![Direction::North, Direction::West],
            GridSpace::SouthWestBend => vec![Direction::South, Direction::West],
            GridSpace::SouthEastBend => vec![Direction::South, Direction::East],
            GridSpace::Ground => vec![],
            GridSpace::Start => vec![
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East,
            ],
        }
    }
}

#[derive(Debug, Error)]
#[error("`{0}` is not a valid GridSpace")]
struct ParseGridSpaceError(char);

impl TryFrom<char> for GridSpace {
    type Error = ParseGridSpaceError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '|' => Ok(GridSpace::VerticalPipe),
            '-' => Ok(GridSpace::HorizontalPipe),
            'L' => Ok(GridSpace::NorthEastBend),
            'J' => Ok(GridSpace::NorthWestBend),
            '7' => Ok(GridSpace::SouthWestBend),
            'F' => Ok(GridSpace::SouthEastBend),
            '.' => Ok(GridSpace::Ground),
            'S' => Ok(GridSpace::Start),
            c => Err(ParseGridSpaceError(c)),
        }
    }
}

type GridRow = Vec<GridSpace>;
type Grid = Vec<GridRow>;

fn grid_row(line: &str) -> Result<GridRow, ParseGridSpaceError> {
    line.chars().map(|c| GridSpace::try_from(c)).collect()
}

#[derive(Debug, Error)]
#[error("Missing start position")]
struct MissingStartError;

fn find_start(grid: &Grid) -> Result<(usize, usize), MissingStartError> {
    for (i, row) in grid.iter().enumerate() {
        for (j, space) in row.iter().enumerate() {
            if let GridSpace::Start = space {
                return Ok((i, j));
            }
        }
    }

    Err(MissingStartError)
}

#[derive(Debug, Error)]
#[error("Tried to move out of bounds")]
struct OutOfBoundsError;

type Coords = (usize, usize);

fn try_move(grid: &Grid, coords: Coords, dir: Direction) -> Result<Coords, OutOfBoundsError> {
    let (r, c) = coords;
    let (dr, dc) = match dir {
        Direction::North => (-1, 0),
        Direction::South => (1, 0),
        Direction::West => (0, -1),
        Direction::East => (0, 1),
    };

    let r = r.checked_add_signed(dr).ok_or(OutOfBoundsError)?;
    let c = c.checked_add_signed(dc).ok_or(OutOfBoundsError)?;

    if r > grid.len() || c > grid[0].len() {
        return Err(OutOfBoundsError);
    }

    Ok((r, c))
}

fn grid(inp: Stdin) -> Result<Grid> {
    let mut grid = Grid::new();
    for line in inp.lines() {
        let line = line?;
        grid.push(grid_row(&line)?);
    }

    Ok(grid)
}

fn connections(grid: &Grid, coords: Coords) -> Result<Vec<Direction>> {
    let space = &grid[coords.0][coords.1];
    if let GridSpace::Start = space {
        let mut res = Vec::new();

        for dir in space.possible_connections() {
            let coords = try_move(&grid, coords, dir)?;
            if grid[coords.0][coords.1]
                .possible_connections()
                .contains(&dir.opposite())
            {
                res.push(dir)
            }
        }

        Ok(res)
    } else {
        Ok(space.possible_connections())
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let grid = grid(stdin())?;
    let start = find_start(&grid)?;

    let mut to_visit = VecDeque::new();
    let mut visited: HashMap<Coords, usize> = HashMap::new();
    to_visit.push_back((start, 0));
    visited.insert(start, 0);

    while let Some((coords, dist)) = to_visit.pop_front() {
        for dir in connections(&grid, coords)? {
            let coords = try_move(&grid, coords, dir)?;
            if visited.contains_key(&coords) {
                continue;
            }

            to_visit.push_back((coords, dist + 1));
            visited.insert(coords, dist + 1);
        }
    }

    let res = match args.part {
        Part::Part1 => visited.values().max().unwrap().to_owned(),
        Part::Part2 => {
            let mut total = 0;
            for (i, row) in grid.iter().enumerate() {
                let mut is_inside = false;
                let mut seen: Option<Direction> = None;

                for (j, _) in row.iter().enumerate() {
                    if visited.contains_key(&(i, j)) {
                        let connections: Vec<_> = connections(&grid, (i, j))?
                            .into_iter()
                            .filter(Direction::is_vertical)
                            .collect();

                        if connections.len() >= 2 {
                            is_inside = !is_inside;
                        } else if connections.len() == 0 {
                            continue;
                        } else {
                            (is_inside, seen) = match (seen, connections[0]) {
                                (None, dir) => (is_inside, Some(dir)),
                                (Some(Direction::North), Direction::North) => (is_inside, None),
                                (Some(Direction::North), Direction::South) => (!is_inside, None),
                                (Some(Direction::South), Direction::North) => (!is_inside, None),
                                (Some(Direction::South), Direction::South) => (is_inside, None),
                                _ => unreachable!(),
                            }
                        }
                    } else if is_inside {
                        total += 1;
                    }
                }
            }

            total
        }
    };

    println!("{res}");

    Ok(())
}

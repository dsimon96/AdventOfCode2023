use std::io::stdin;

use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use nom::{
    bytes::complete::{tag, take},
    character::complete::{anychar, char, digit1, multispace1},
    combinator::{map_res, recognize},
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

#[derive(Clone, Copy, Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

type Coords = (isize, isize);

impl From<Direction> for Coords {
    fn from(value: Direction) -> Self {
        match value {
            Direction::Up => (-1, 0),
            Direction::Down => (1, 0),
            Direction::Left => (0, -1),
            Direction::Right => (0, 1),
        }
    }
}

#[derive(Debug, Error)]
enum ParseError {
    #[error("`{0}` is not a valid direction")]
    Direction(char),
}

impl TryFrom<char> for Direction {
    type Error = ParseError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'U' => Ok(Direction::Up),
            'D' => Ok(Direction::Down),
            'L' => Ok(Direction::Left),
            'R' => Ok(Direction::Right),
            _ => Err(ParseError::Direction(value)),
        }
    }
}

fn direction(input: &str) -> IResult<&str, Direction> {
    map_res(anychar, Direction::try_from)(input)
}

fn num(input: &str) -> IResult<&str, usize> {
    map_res(recognize(digit1), str::parse)(input)
}

fn color(input: &str) -> IResult<&str, String> {
    let (input, vals) = delimited(tag("(#"), take(6usize), char(')'))(input)?;

    Ok((input, vals.to_owned()))
}

#[derive(Debug)]
struct Step {
    direction: Direction,
    length: usize,
    color: String,
}

fn step(input: &str) -> IResult<&str, Step> {
    let (input, direction) = direction(input)?;
    let (input, _) = multispace1(input)?;
    let (input, length) = num(input)?;
    let (input, _) = multispace1(input)?;
    let (input, color) = color(input)?;

    Ok((
        input,
        Step {
            direction,
            length,
            color,
        },
    ))
}

fn decode_direction(c: char) -> Result<Direction> {
    match c {
        '0' => Ok(Direction::Right),
        '1' => Ok(Direction::Down),
        '2' => Ok(Direction::Left),
        '3' => Ok(Direction::Up),
        _ => bail!("Failed to decode direction"),
    }
}

fn decode_color(s: &str) -> Result<(Direction, usize)> {
    let direction = decode_direction(s.chars().nth(5).unwrap())?;
    let meters = usize::from_str_radix(&s[..5], 16)?;

    Ok((direction, meters))
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut cur = (0, 0);
    let mut edge = vec![cur];
    let mut boundary_points = 0; // avoid double counting the origin

    for line in stdin().lines() {
        let (_, step) = step(&line?).map_err(|e| e.to_owned())?;

        let (direction, meters) = match args.part {
            Part::Part1 => (step.direction, step.length),
            Part::Part2 => decode_color(&step.color)?,
        };

        let (dr, dc) = Coords::from(direction);
        let end = (cur.0 + dr * meters as isize, cur.1 + dc * meters as isize);

        edge.push(end);
        cur = end;
        boundary_points += meters;
    }

    // shoelace formula
    let area = edge
        .windows(2)
        .map(|window| {
            let &[(x1, y1), (x2, y2)] = window else {
                unreachable!()
            };
            x1 * y2 - y1 * x2
        })
        .sum::<isize>()
        .abs()
        / 2;

    let res = area + boundary_points as isize / 2 + 1;

    println!("{res}");

    Ok(())
}

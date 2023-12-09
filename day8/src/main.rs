use std::{
    collections::HashMap,
    io::{stdin, Error, ErrorKind},
    mem::replace,
};

use clap::{Parser, Subcommand};
use nom::{
    bytes::complete::{tag, take},
    character::complete::anychar,
    combinator::{map, map_res},
    multi::many1,
    sequence::{delimited, separated_pair},
    IResult,
};
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

#[derive(Debug)]
enum Move {
    Left,
    Right,
}

#[derive(Error, Debug)]
enum ParseMoveError {
    #[error("Character `{0}` is not a valid move")]
    InvalidChar(char),
}

impl TryFrom<char> for Move {
    type Error = ParseMoveError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'L' => Ok(Move::Left),
            'R' => Ok(Move::Right),
            c => Err(ParseMoveError::InvalidChar(c)),
        }
    }
}

fn move_seq(input: &str) -> IResult<&str, Vec<Move>> {
    many1(map_res(anychar, Move::try_from))(input)
}
type Node = (String, (String, String));

fn label(input: &str) -> IResult<&str, String> {
    map(take(3usize), String::from)(input)
}

fn node(input: &str) -> IResult<&str, Node> {
    separated_pair(
        label,
        tag(" = "),
        delimited(tag("("), separated_pair(label, tag(", "), label), tag(")")),
    )(input)
}

type NodeMap = HashMap<String, (String, String)>;

fn find_length(
    moves: &Vec<Move>,
    node_map: &NodeMap,
    start: &str,
    end_pred: fn(&str) -> bool,
) -> usize {
    let mut steps: usize = 0;
    let mut cur = start;
    while !end_pred(cur) {
        let m = &moves[steps % moves.len()];
        let next = node_map.get(cur).expect("Invalid node");
        let next = match *m {
            Move::Left => next.0.as_str(),
            Move::Right => next.1.as_str(),
        };
        let _ = replace(&mut cur, next);
        steps += 1;
    }

    steps
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut inp = stdin().lines();
    let moves = inp.next().ok_or(Error::from(ErrorKind::UnexpectedEof))??;
    let (_, moves) = move_seq(&moves).map_err(|e| e.to_owned())?;
    let _ = inp.next().ok_or(Error::from(ErrorKind::UnexpectedEof))??;

    let mut node_map: NodeMap = NodeMap::new();

    for line in inp {
        let line = line?;
        let (_, (label, next)) = node(&line).map_err(|e| e.to_owned())?;

        node_map.insert(label, next);
    }

    let steps = match args.part {
        Part::Part1 => find_length(&moves, &node_map, "AAA", |s| s == "ZZZ"),
        Part::Part2 => {
            // The problem is constructed such that each node ending with A connects to a separate chain which contains only one node ending with Z.
            // Furthermore the path length from A to Z is the same as the cycle length.
            let path_lengths: Vec<usize> = node_map
                .keys()
                .filter(|&k| k.ends_with("A"))
                .map(|s| find_length(&moves, &node_map, s, |c| c.ends_with("Z")))
                .collect();

            path_lengths.into_iter().fold(1, num::integer::lcm)
        }
    };

    println!("{steps}");
    Ok(())
}

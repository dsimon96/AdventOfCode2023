use std::{
    collections::{HashMap, HashSet, VecDeque},
    io::stdin,
    rc::Rc,
};

use anyhow::Result;
use clap::{Parser, Subcommand};
use nom::{
    character::complete::{char, digit1},
    combinator::map_res,
    sequence::{separated_pair, tuple},
    IResult,
};

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

type Coord = usize;

#[derive(Debug, PartialEq, Eq, Hash)]
struct Range(Coord, Coord);

impl Range {
    fn iter(&self) -> impl Iterator<Item = Coord> {
        self.0..=self.1
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct Brick {
    z: Range, // default sort order is by initial height
    x: Range,
    y: Range,
}

impl Brick {
    fn horizontal_slice(&self) -> impl Iterator<Item = (Coord, Coord)> + '_ {
        self.x
            .iter()
            .flat_map(|x| self.y.iter().map(move |y| (x, y)))
    }
}

fn coord(input: &str) -> IResult<&str, Coord> {
    map_res(digit1, str::parse)(input)
}

fn coords3(input: &str) -> IResult<&str, (Coord, Coord, Coord)> {
    let (input, (x, _, y, _, z)) = tuple((coord, char(','), coord, char(','), coord))(input)?;

    Ok((input, (x, y, z)))
}

fn brick(input: &str) -> IResult<&str, Brick> {
    let (input, ((x1, y1, z1), (x2, y2, z2))) = separated_pair(coords3, char('~'), coords3)(input)?;

    Ok((
        input,
        Brick {
            z: Range(z1, z2),
            x: Range(x1, x2),
            y: Range(y1, y2),
        },
    ))
}

fn main() -> Result<()> {
    let args = Args::parse();

    // store bricks in a min-heap so that we can later iterate in ascending z1 order
    let mut bricks: Vec<Rc<Brick>> = Vec::new();
    for line in stdin().lines() {
        let (_, brick) = brick(&line?).map_err(|e| e.to_owned())?;
        bricks.push(brick.into());
    }
    bricks.sort_by(|lhs, rhs| lhs.z.0.cmp(&rhs.z.0));

    // (x, y) -> (highest z so far, brick that occupies that z)
    let mut heightmap: HashMap<(Coord, Coord), (usize, Rc<Brick>)> = HashMap::new();
    // A -> [Bricks which A supports]
    let mut supports: HashMap<Rc<Brick>, HashSet<Rc<Brick>>> = HashMap::new();
    // A -> [Bricks which A is supported by]
    let mut supported_by: HashMap<Rc<Brick>, HashSet<Rc<Brick>>> = HashMap::new();

    for brick in bricks.iter() {
        let mut max_height = 0;
        let mut support_set = HashSet::new();
        for point in brick.horizontal_slice() {
            if let Some((height, other)) = heightmap.get(&point) {
                if *height > max_height {
                    max_height = *height;
                    support_set = HashSet::from([other.clone()]);
                } else if *height == max_height {
                    support_set.insert(other.clone());
                }
            }
        }

        let Range(z1, z2) = brick.z;
        let new_z1 = max_height + 1;
        let new_z2 = z2 - z1 + new_z1;
        for point in brick.horizontal_slice() {
            heightmap.insert(point, (new_z2, brick.clone()));
        }

        for other in support_set.iter() {
            supports
                .entry(other.clone())
                .or_default()
                .insert(brick.clone());
        }

        supported_by.insert(brick.clone(), support_set);
    }

    let res = match args.part {
        Part::Part1 => bricks
            .iter()
            .filter(|&a| {
                let Some(others) = supports.get(a) else {
                    return true;
                };
                others.iter().all(|b| {
                    supported_by
                        .get(b)
                        .unwrap()
                        .iter()
                        .any(|c| !Rc::ptr_eq(a, c))
                })
            })
            .count(),
        Part::Part2 => bricks
            .iter()
            .map(|brick| {
                let mut count = 0;
                let mut hypothetical_supports = supported_by.clone();
                let mut fall_queue = VecDeque::from([brick.clone()]);

                while let Some(a) = fall_queue.pop_front() {
                    count += 1;
                    let Some(others) = supports.get(&a) else {
                        continue;
                    };
                    for other in others {
                        let remaining_supports = hypothetical_supports.get_mut(other).unwrap();
                        remaining_supports.remove(&a);
                        if remaining_supports.is_empty() {
                            fall_queue.push_back(other.clone());
                        }
                    }
                }

                count - 1 // the initial brick doesn't count
            })
            .sum(),
    };

    println!("{res}");
    Ok(())
}

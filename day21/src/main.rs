use std::{
    collections::{hash_map::Entry, HashMap, VecDeque},
    io::{stdin, BufRead},
};

use anyhow::bail;
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    part: Part,
    n: usize,
}

#[derive(Subcommand)]
enum Part {
    Part1,
    Part2,
}

type Coords = (isize, isize);
type Map = Vec<Vec<bool>>;

fn parse_input(input: impl BufRead) -> anyhow::Result<(Map, Coords)> {
    let mut map = Vec::new();
    let mut start = Coords::default();
    for (i, line) in input.lines().enumerate() {
        let line = line?;
        let mut row = Vec::new();
        for (j, c) in line.chars().enumerate() {
            row.push(match c {
                'S' => {
                    start = (i as isize, j as isize);
                    false
                }
                '.' => false,
                '#' => true,
                _ => bail!("Unrecognized character"),
            });
        }

        map.push(row);
    }

    Ok((map, start))
}

#[derive(Clone, Copy)]
enum Direction {
    North,
    South,
    East,
    West,
}

fn move_p1(map: &Map, (r, c): Coords, direction: Direction) -> Option<Coords> {
    match direction {
        Direction::North if r > 0 => Some((r - 1, c)),
        Direction::South if r < (map.len() - 1) as isize => Some((r + 1, c)),
        Direction::East if c < (map[0].len() - 1) as isize => Some((r, c + 1)),
        Direction::West if c > 0 => Some((r, c - 1)),
        _ => None,
    }
}

fn check_p1(map: &Map, (r, c): Coords) -> bool {
    map[r as usize][c as usize]
}

fn move_p2(_: &Map, (r, c): Coords, direction: Direction) -> Option<Coords> {
    match direction {
        Direction::North => Some((r - 1, c)),
        Direction::South => Some((r + 1, c)),
        Direction::East => Some((r, c + 1)),
        Direction::West => Some((r, c - 1)),
    }
}

fn check_p2(map: &Map, (r, c): Coords) -> bool {
    map[r.rem_euclid(map.len() as isize) as usize][c.rem_euclid(map[0].len() as isize) as usize]
}

fn floodfill(
    map: &Map,
    start: Coords,
    max_dist: usize,
    move_func: fn(&Map, Coords, Direction) -> Option<Coords>,
    check_func: fn(&Map, Coords) -> bool,
) -> HashMap<Coords, usize> {
    let mut distances: HashMap<Coords, usize> = HashMap::from([(start, 0)]);
    let mut to_visit = VecDeque::from([(start, 0)]);

    while let Some((coords, dist)) = to_visit.pop_front() {
        let new_dist = dist + 1;
        if new_dist > max_dist {
            continue;
        }
        for direction in [
            Direction::North,
            Direction::South,
            Direction::East,
            Direction::West,
        ] {
            let Some(new_coords) = move_func(map, coords, direction) else {
                continue;
            };
            if check_func(map, new_coords) {
                continue;
            }
            match distances.entry(new_coords) {
                Entry::Occupied(_) => continue,
                Entry::Vacant(v) => {
                    v.insert(new_dist);
                    to_visit.push_back((new_coords, new_dist));
                }
            }
        }
    }

    distances
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let (map, start) = parse_input(stdin().lock())?;

    let res = match args.part {
        Part::Part1 => floodfill(&map, start, args.n, move_p1, check_p1),
        Part::Part2 => floodfill(&map, start, args.n, move_p2, check_p2),
    }
    .values()
    .filter(|&v| v % 2 == args.n % 2)
    .count();

    println!("{res}");
    Ok(())
}

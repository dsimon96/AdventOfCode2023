use std::{
    collections::{HashMap, HashSet},
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
    fn available_directions(&self, part: &Part) -> impl Iterator<Item = Direction> + 'static {
        match (self, part) {
            (Space::Empty, _) | (Space::Slope(_), Part::Part2) => ALL_DIRECTIONS.iter().copied(),
            (Space::Forest, _) => unreachable!(),
            (Space::Slope(Direction::N), Part::Part1) => [Direction::N].iter().copied(),
            (Space::Slope(Direction::E), Part::Part1) => [Direction::E].iter().copied(),
            (Space::Slope(Direction::S), Part::Part1) => [Direction::S].iter().copied(),
            (Space::Slope(Direction::W), Part::Part1) => [Direction::W].iter().copied(),
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

    fn successors(&self, coords: Coords, part: &Part) -> impl Iterator<Item = Coords> + '_ {
        self[coords]
            .available_directions(part)
            .filter_map(move |dir| {
                let Some(coords) = try_move(self, coords, dir) else {
                    return None;
                };
                if let Space::Forest = self[coords] {
                    return None;
                }

                return Some(coords);
            })
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

#[derive(Debug)]
struct Graph {
    edges: HashMap<Coords, HashSet<Coords>>,
    edge_weights: HashMap<(Coords, Coords), usize>,
}

fn discover_graph(input: &Input, start: Coords, end: Coords, part: &Part) -> Graph {
    let mut nodes = HashSet::from([start, end]);
    let mut edges: HashMap<Coords, HashSet<Coords>> = HashMap::new();
    let mut edge_weights = HashMap::new();

    let mut to_explore = Vec::from([start]);
    while let Some(node) = to_explore.pop() {
        for mut cur in input.successors(node, part) {
            let mut steps = 1;
            let mut prev = node;
            let mut found_node = false;
            loop {
                if nodes.contains(&cur) {
                    found_node = true;
                    break;
                }
                let successors: Vec<_> = input
                    .successors(cur, part)
                    .filter(|&next| next != prev)
                    .collect();

                match successors[..] {
                    [] => break,
                    [next] => {
                        prev = cur;
                        cur = next;
                        steps += 1;
                    }
                    _ => {
                        found_node = true;
                        nodes.insert(cur);
                        to_explore.push(cur);
                        break;
                    }
                }
            }

            if found_node {
                edges.entry(node).or_default().insert(cur);
                edge_weights.insert((node, cur), steps);
            }
        }
    }

    Graph {
        edges,
        edge_weights,
    }
}

fn find_longest_path(graph: &Graph, start: Coords, end: Coords) -> Option<usize> {
    let mut paths = Vec::from([vec![start]]);

    let mut hike_lengths = Vec::new();
    while let Some(path) = paths.pop() {
        let cur = *path.last().expect("Path must be non-empty");
        if cur == end {
            hike_lengths.push(
                path.windows(2)
                    .map(|window| {
                        let &[x, y] = window else { unreachable!() };
                        graph
                            .edge_weights
                            .get(&(x, y))
                            .expect("Edges must have a corresponding weight")
                    })
                    .sum(),
            );
            continue;
        }
        for &next in graph.edges.get(&cur).expect("No out-edges found").iter() {
            if path.contains(&next) {
                continue;
            }
            let mut new_path = path.clone();
            new_path.push(next);
            paths.push(new_path)
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

    let graph = discover_graph(&input, start, end, &args.part);
    let res = find_longest_path(&graph, start, end).context("No path found")?;

    println!("{res}");

    Ok(())
}

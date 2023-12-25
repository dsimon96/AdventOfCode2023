use std::{collections::HashMap, io::stdin};

use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use nom::{
    bytes::complete::{tag, take},
    character::complete::space1,
    combinator::map,
    multi::separated_list1,
    sequence::separated_pair,
    IResult,
};
use pathfinding::directed::edmonds_karp::edmonds_karp_sparse;
use petgraph::{
    graph::{NodeIndex, UnGraph},
    visit::{Bfs, EdgeRef},
};

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    part: Part,
}

#[derive(Subcommand)]
enum Part {
    Part1,
}

#[derive(Debug)]
struct Input {
    nodes: HashMap<String, NodeIndex>,
    graph: UnGraph<String, ()>,
}

fn name(input: &str) -> IResult<&str, String> {
    map(take(3usize), String::from)(input)
}

fn parse_line(input: &str) -> IResult<&str, (String, Vec<String>)> {
    separated_pair(name, tag(": "), separated_list1(space1, name))(input)
}

fn parse_input() -> Result<Input> {
    let mut nodes = HashMap::new();
    let mut graph = UnGraph::new_undirected();

    for line in stdin().lines() {
        let (_, (v, ws)) = parse_line(&line?).map_err(|e| e.to_owned())?;

        let v = *nodes
            .entry(v)
            .or_insert_with_key(|v| graph.add_node(v.to_owned()));
        for w in ws {
            let w = *nodes
                .entry(w)
                .or_insert_with_key(|w| graph.add_node(w.to_owned()));
            graph.add_edge(v, w, ());
        }
    }

    Ok(Input { nodes, graph })
}

fn main() -> Result<()> {
    let args = Args::parse();
    let input = parse_input()?;

    let res = match args.part {
        Part::Part1 => {
            let nodes: Vec<_> = input.nodes.values().collect();
            let Some((source, others)) = nodes.split_first() else {
                bail!("Empty input")
            };

            let mut component_sizes = None;
            for sink in others {
                let caps: Vec<_> = input
                    .graph
                    .edge_references()
                    .flat_map(|edge| {
                        [
                            (edge.source(), edge.target()),
                            (edge.target(), edge.source()),
                        ]
                    })
                    .collect();
                let (_, _, cut) = edmonds_karp_sparse(
                    &nodes[..],
                    source,
                    sink,
                    caps.iter().map(|(a, b)| ((a, b), 1)),
                );

                if cut.len() <= 3 {
                    let mut residual = input.graph.clone();
                    for ((v, w), _) in cut {
                        residual.remove_edge(residual.find_edge(*v, *w).unwrap());
                    }

                    let mut component_size = 0;
                    let mut bfs = Bfs::new(&residual, **source);
                    while let Some(_) = bfs.next(&residual) {
                        component_size += 1;
                    }

                    component_sizes = Some((component_size, input.nodes.len() - component_size));
                    break;
                }
            }

            let Some((a, b)) = component_sizes else {
                bail!("Could not find a cut");
            };

            a * b
        }
    };

    println!("{res}");

    Ok(())
}

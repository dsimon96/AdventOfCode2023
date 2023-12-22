use std::{
    collections::{HashMap, VecDeque},
    io::{stdin, BufRead},
};

use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use multimap::MultiMap;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, char},
    combinator::map,
    multi::separated_list1,
    sequence::{preceded, separated_pair},
    IResult,
};

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    part: Part,
}

#[derive(PartialEq, Eq, Subcommand)]
enum Part {
    Part1 {
        #[arg(default_value_t = 1000)]
        n: usize,
    },
    Part2,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
enum Pulse {
    High,
    Low,
}

type ModuleId = String;

enum Module {
    FlipFlop { memory: bool },
    Conjunction { memory: HashMap<ModuleId, Pulse> },
    Broadcast,
}

struct Event {
    source: ModuleId,
    dest: ModuleId,
    pulse: Pulse,
}

impl Module {
    fn handle(&mut self, event: &Event) -> Option<Pulse> {
        match self {
            Module::FlipFlop { memory } => match event.pulse {
                Pulse::High => None,
                Pulse::Low => {
                    let res = if *memory { Pulse::Low } else { Pulse::High };
                    *memory = !*memory;

                    Some(res)
                }
            },
            Module::Conjunction { memory } => {
                let Some(input_memory) = memory.get_mut(&event.source) else {
                    panic!("Received a message from an unexpected source!")
                };
                *input_memory = event.pulse;

                if memory.values().all(|pulse| *pulse == Pulse::High) {
                    Some(Pulse::Low)
                } else {
                    Some(Pulse::High)
                }
            }
            Module::Broadcast => Some(event.pulse),
        }
    }
}

fn module_id(input: &str) -> IResult<&str, String> {
    map(alpha1, ToString::to_string)(input)
}

fn broadcaster(input: &str) -> IResult<&str, (ModuleId, Module)> {
    let (input, _) = tag("broadcaster")(input)?;

    Ok((input, ("broadcaster".into(), Module::Broadcast)))
}

fn flipflop(input: &str) -> IResult<&str, (ModuleId, Module)> {
    let (input, id) = preceded(char('%'), module_id)(input)?;
    Ok((input, (id, Module::FlipFlop { memory: false })))
}

fn conjunction(input: &str) -> IResult<&str, (ModuleId, Module)> {
    let (input, id) = preceded(char('&'), module_id)(input)?;
    Ok((
        input,
        (
            id,
            Module::Conjunction {
                memory: HashMap::new(),
            },
        ),
    ))
}

fn module(input: &str) -> IResult<&str, (ModuleId, Module)> {
    alt((broadcaster, flipflop, conjunction))(input)
}

fn dests(input: &str) -> IResult<&str, Vec<ModuleId>> {
    separated_list1(tag(", "), module_id)(input)
}

fn module_spec(input: &str) -> IResult<&str, ((ModuleId, Module), Vec<ModuleId>)> {
    separated_pair(module, tag(" -> "), dests)(input)
}

type ModuleRegistry = HashMap<ModuleId, Module>;
type ModuleConnections = MultiMap<ModuleId, ModuleId>;

fn parse_input(
    input: impl BufRead,
) -> Result<(ModuleRegistry, ModuleConnections, ModuleConnections)> {
    let mut registry = ModuleRegistry::new();
    let mut forward = ModuleConnections::new();
    let mut reverse = ModuleConnections::new();
    for line in input.lines() {
        let (_, ((id, module), dests)) = module_spec(&line?).map_err(|e| e.to_owned())?;
        registry.insert(id.clone(), module);
        forward.insert_many(id.clone(), dests.clone());
        reverse.extend(dests.into_iter().map(|dest| (dest, id.clone())));
    }

    for (id, module) in registry.iter_mut() {
        if let Module::Conjunction { memory } = module {
            memory.extend(
                reverse
                    .get_vec(id)
                    .expect("Module has no inputs")
                    .iter()
                    .map(|input| (input.clone(), Pulse::Low)),
            )
        }
    }

    Ok((registry, forward, reverse))
}

fn button_event(target: ModuleId) -> Event {
    Event {
        source: "button".into(),
        dest: target,
        pulse: Pulse::Low,
    }
}

fn determine_activation_period(
    registry: &mut ModuleRegistry,
    forward: &ModuleConnections,
    input: &ModuleId,
    output: &ModuleId,
    expected_pulse: Pulse,
) -> usize {
    let mut events = VecDeque::new();
    let mut count = 0;
    let mut received = false;
    while !received {
        events.push_back(button_event(input.clone()));
        count += 1;

        while let Some(event) = events.pop_front() {
            if event.dest == *output && event.pulse == expected_pulse {
                received = true;
                break;
            }
            let id = &event.dest;
            if let Some(pulse) = registry
                .get_mut(id)
                .and_then(|module| module.handle(&event))
            {
                events.extend(
                    forward
                        .get_vec(id)
                        .expect("Could not find outputs for module")
                        .iter()
                        .map(|dest| Event {
                            source: id.clone(),
                            dest: dest.clone(),
                            pulse,
                        }),
                )
            }
        }
    }

    count
}

fn main() -> Result<()> {
    let args = Args::parse();

    let (mut registry, forward, reverse) = parse_input(stdin().lock())?;

    let res = match args.part {
        Part::Part1 { n } => {
            let mut counts: HashMap<Pulse, usize> = HashMap::new();
            let mut events = VecDeque::new();
            for _ in 0..n {
                events.push_back(button_event("broadcaster".into()));

                while let Some(event) = events.pop_front() {
                    *counts.entry(event.pulse).or_default() += 1;
                    let id = &event.dest;
                    if let Some(pulse) = registry
                        .get_mut(id)
                        .and_then(|module| module.handle(&event))
                    {
                        events.extend(
                            forward
                                .get_vec(id)
                                .expect("Could not find outputs for module")
                                .iter()
                                .map(|dest| Event {
                                    source: id.clone(),
                                    dest: dest.clone(),
                                    pulse,
                                }),
                        )
                    }
                }
            }

            counts.values().product::<usize>()
        }
        Part::Part2 => {
            let Some(origins) = forward.get_vec("broadcaster") else {
                bail!("Could not find broadcast node's outputs");
            };

            let Some(dest) = reverse.get("rx") else {
                bail!("Could not find 'rx' node's input");
            };

            origins
                .iter()
                .map(|source| {
                    determine_activation_period(&mut registry, &forward, source, dest, Pulse::High)
                })
                .reduce(num::integer::lcm)
                .expect("No outputs of broadcast node")
        }
    };

    println!("{res}");
    Ok(())
}

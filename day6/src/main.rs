use std::{
    io::{stdin, Read},
    num::ParseIntError,
};

use anyhow::Result;
use clap::{Parser, Subcommand};
use nom::{
    bytes::complete::{tag, take_till},
    character::complete::{digit1, newline, space1},
    combinator::{map_res, recognize},
    multi::separated_list1,
    sequence::{preceded, terminated},
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

#[derive(Debug)]
struct Race {
    time: usize,
    distance: usize,
}

fn num(input: &str) -> IResult<&str, usize> {
    map_res(recognize(digit1), str::parse)(input)
}

fn num_list<'a>(input: &'a str, part: &Part) -> IResult<&'a str, Vec<usize>> {
    match part {
        Part::Part1 => terminated(preceded(space1, separated_list1(space1, num)), newline)(input),
        Part::Part2 => map_res(
            // drop the spaces and parse the remaining digits as a single number
            terminated(take_till(|c| c == '\n'), newline),
            |x: &str| -> Result<Vec<usize>, ParseIntError> {
                let s = String::from_iter(x.chars().filter(|&c| c != ' '));
                let mut res = Vec::new();

                res.push(s.parse()?);

                Ok(res)
            },
        )(input),
    }
}

fn times<'a>(input: &'a str, part: &Part) -> IResult<&'a str, Vec<usize>> {
    let (input, _) = tag("Time:")(input)?;
    num_list(input, part)
}

fn distances<'a>(input: &'a str, part: &Part) -> IResult<&'a str, Vec<usize>> {
    let (input, _) = tag("Distance:")(input)?;
    num_list(input, part)
}

fn races<'a>(input: &'a str, part: &Part) -> IResult<&'a str, Vec<Race>> {
    let (input, times) = times(input, &part)?;
    let (input, distances) = distances(input, &part)?;

    Ok((
        input,
        times
            .into_iter()
            .zip(distances.into_iter())
            .map(|(time, distance)| Race { time, distance })
            .collect(),
    ))
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut inp = String::new();
    let _ = stdin().read_to_string(&mut inp)?;
    let (_, races) = races(&inp, &args.part).map_err(|e| e.to_owned())?;

    let res: usize = races
        .into_iter()
        .map(|race| {
            (1..race.time)
                .filter_map(|hold_len| {
                    let dist = hold_len * (race.time - hold_len);
                    if dist > race.distance {
                        Some(())
                    } else {
                        None
                    }
                })
                .count()
        })
        .product();

    println!("{res}");
    Ok(())
}

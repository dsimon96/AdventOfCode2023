use std::{
    collections::{HashMap, HashSet},
    io::stdin,
};

use clap::{Parser, Subcommand};
use nom::{
    bytes::complete::tag,
    character::complete::{digit1, multispace0},
    combinator::{map_res, recognize},
    multi::separated_list1,
    sequence::{preceded, separated_pair},
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
struct ScratchCard {
    id: u32,
    winning_numbers: Vec<u32>,
    numbers: Vec<u32>,
}

fn num(input: &str) -> IResult<&str, u32> {
    preceded(multispace0, map_res(recognize(digit1), str::parse))(input)
}

fn scratch_card_header(input: &str) -> IResult<&str, u32> {
    let (input, _) = tag("Card")(input)?;
    let (input, id) = num(input)?;
    let (input, _) = tag(": ")(input)?;

    Ok((input, id))
}

fn number_seq(input: &str) -> IResult<&str, Vec<u32>> {
    separated_list1(tag(" "), num)(input)
}

fn scratch_card(input: &str) -> IResult<&str, ScratchCard> {
    let (input, id) = scratch_card_header(input)?;
    let (input, (winning_numbers, numbers)) =
        separated_pair(number_seq, tag(" | "), number_seq)(input)?;

    Ok((
        input,
        ScratchCard {
            id,
            winning_numbers,
            numbers,
        },
    ))
}

fn num_matches(card: &ScratchCard) -> usize {
    let winning_numbers: HashSet<u32> = HashSet::from_iter(card.winning_numbers.iter().copied());
    let numbers: HashSet<u32> = HashSet::from_iter(card.numbers.iter().copied());

    winning_numbers.intersection(&numbers).count()
}

fn main() {
    let args = Args::parse();

    let mut total = 0;
    match args.part {
        Part::Part1 => {
            for line in stdin().lines().map(Result::unwrap) {
                let card = scratch_card(&line).unwrap().1;

                let matches = num_matches(&card);
                if matches > 0 {
                    total += 1 << (matches - 1);
                }
            }
        }
        Part::Part2 => {
            let mut card_counts: HashMap<u32, u32> = HashMap::new();
            for line in stdin().lines().map(Result::unwrap) {
                let card = scratch_card(&line).unwrap().1;
                let copies = card_counts.get(&card.id).unwrap_or(&0) + 1;
                total += copies;

                let matches = num_matches(&card);
                for i in 0..matches {
                    *card_counts.entry(card.id + 1 + i as u32).or_default() += copies;
                }
            }
        }
    }

    println!("{total}")
}

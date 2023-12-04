use std::io::stdin;

use anyhow::Result;
use clap::{Parser, Subcommand};

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

fn get_val_p1(s: &str) -> u32 {
    let digits: Vec<u32> = s.chars().filter_map(|c| c.to_digit(10)).collect();
    digits.first().unwrap() * 10 + digits.last().unwrap()
}

const DIGIT_STRS: &[(&str, u32)] = &[
    ("one", 1),
    ("two", 2),
    ("three", 3),
    ("four", 4),
    ("five", 5),
    ("six", 6),
    ("seven", 7),
    ("eight", 8),
    ("nine", 9),
];

fn get_first_digit(s: &str) -> u32 {
    for (i, c) in s.chars().enumerate() {
        if let Some(d) = c.to_digit(10) {
            return d;
        }

        for (pattern, val) in DIGIT_STRS {
            if s[i..].starts_with(pattern) {
                return *val;
            }
        }
    }

    panic!("Line must contain a digit");
}

fn get_last_digit(s: &str) -> u32 {
    for (i, c) in s.chars().rev().enumerate() {
        if let Some(d) = c.to_digit(10) {
            return d;
        }

        for (pattern, val) in DIGIT_STRS {
            if s[..s.len() - i].ends_with(pattern) {
                return *val;
            }
        }
    }

    panic!("Line must contain a digit");
}

fn get_val_p2(s: &str) -> u32 {
    get_first_digit(s) * 10 + get_last_digit(s)
}

fn main() -> Result<()> {
    let args = Args::parse();

    let get_val_fn = match args.part {
        Part::Part1 => get_val_p1,
        Part::Part2 => get_val_p2,
    };

    let result: u32 = stdin().lines().map(|l| get_val_fn(&l.unwrap())).sum();

    println!("{result}");

    Ok(())
}

use std::{io::stdin, num::ParseIntError};

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

fn seq(input: &str) -> Result<Vec<i64>, ParseIntError> {
    input.split(" ").map(|s| s.parse::<i64>()).collect()
}

fn predict_next(seq: &Vec<i64>) -> i64 {
    if seq.iter().all(|&v| v == 0) {
        return 0;
    }

    let lower_order_prediction =
        predict_next(&seq.windows(2).map(|slice| slice[1] - slice[0]).collect());

    *seq.last().unwrap() + lower_order_prediction
}

fn predict_prev(seq: &Vec<i64>) -> i64 {
    if seq.iter().all(|&v| v == 0) {
        return 0;
    }

    let lower_order_prediction =
        predict_prev(&seq.windows(2).map(|slice| slice[1] - slice[0]).collect());

    *seq.first().unwrap() - lower_order_prediction
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut total: i64 = 0;
    for line in stdin().lines() {
        let line = line?;
        let seq = seq(&line)?;

        total += match args.part {
            Part::Part1 => predict_next(&seq),
            Part::Part2 => predict_prev(&seq),
        };
    }

    println!("{total}");
    Ok(())
}

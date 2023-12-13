use std::{io::stdin, iter::once, num::NonZeroUsize};

use clap::{Parser, Subcommand};
use memoize::memoize;
use nom::{
    character::complete::{char, digit1, one_of, space1},
    combinator::{map_res, recognize},
    multi::{many1, separated_list1},
    sequence::separated_pair,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum SpringCondition {
    Operational,
    Damaged,
    Unknown,
}

#[derive(Debug, Error)]
#[error("`{0}` is not a valid SpringCondition")]
struct SpringConditionParseError(char);

impl TryFrom<char> for SpringCondition {
    type Error = SpringConditionParseError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '.' => Ok(SpringCondition::Operational),
            '#' => Ok(SpringCondition::Damaged),
            '?' => Ok(SpringCondition::Unknown),
            c => Err(SpringConditionParseError(c)),
        }
    }
}

type Row = Vec<SpringCondition>;

fn row(input: &str) -> IResult<&str, Row> {
    many1(map_res(one_of(".#?"), SpringCondition::try_from))(input)
}
type GroupSize = NonZeroUsize;

fn group_size(input: &str) -> IResult<&str, GroupSize> {
    map_res(recognize(digit1), str::parse)(input)
}

fn group_sizes(input: &str) -> IResult<&str, Vec<GroupSize>> {
    separated_list1(char(','), group_size)(input)
}

#[derive(Debug)]
struct Record {
    row: Row,
    group_sizes: Vec<GroupSize>,
}

fn record(input: &str) -> IResult<&str, Record> {
    let (input, (row, group_sizes)) = separated_pair(row, space1, group_sizes)(input)?;

    Ok((input, Record { row, group_sizes }))
}

#[memoize]
fn num_arrangements(
    row: Vec<SpringCondition>,
    group_sizes: Vec<GroupSize>,
    start_constraint: Option<SpringCondition>,
) -> usize {
    match (row.split_first(), start_constraint) {
        // end of row, must also have reached end of groups
        (None, _) => group_sizes.is_empty().into(),

        // invalid constraint
        (_, Some(SpringCondition::Unknown)) => unreachable!(),

        // not matching constraint
        (Some((SpringCondition::Operational, _)), Some(SpringCondition::Damaged)) => 0,
        (Some((SpringCondition::Damaged, _)), Some(SpringCondition::Operational)) => 0,

        (Some((SpringCondition::Operational, row)), _) => {
            num_arrangements(row.to_vec(), group_sizes, None)
        }

        (Some((SpringCondition::Damaged, row)), _) => {
            let Some((&first, rest_sizes)) = group_sizes.split_first() else {
                return 0;
            };
            let remaining_in_first_group = first.get() - 1;
            match remaining_in_first_group {
                // must reach end of group
                0 => num_arrangements(
                    row.to_vec(),
                    rest_sizes.to_vec(),
                    Some(SpringCondition::Operational),
                ),

                // group must continue
                n => num_arrangements(
                    row.to_vec(),
                    once(GroupSize::try_from(n).unwrap())
                        .chain(rest_sizes.iter().copied())
                        .collect::<Vec<_>>(),
                    Some(SpringCondition::Damaged),
                ),
            }
        }

        (Some((SpringCondition::Unknown, row)), constraint) => {
            let mut res = 0;

            if let None | Some(SpringCondition::Operational) = constraint {
                // try replacing the first element with Operational
                res += num_arrangements(
                    once(SpringCondition::Operational)
                        .chain(row.iter().copied())
                        .collect::<Vec<_>>(),
                    group_sizes.clone(),
                    start_constraint,
                );
            }

            if let None | Some(SpringCondition::Damaged) = constraint {
                // try replacing the first element with Damaged
                res += num_arrangements(
                    once(SpringCondition::Damaged)
                        .chain(row.iter().copied())
                        .collect::<Vec<_>>(),
                    group_sizes.clone(),
                    start_constraint,
                );
            }

            res
        }
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut result = 0;
    for line in stdin().lines() {
        let line = line?;
        let (_, record) = record(&line).map_err(|e| e.to_owned())?;

        result += match args.part {
            Part::Part1 => num_arrangements(record.row, record.group_sizes, None),
            Part::Part2 => {
                // row is repeated 5 times, joined by 'Unknown'
                let row_len = record.row.len() * 5 + 4;
                let row = record
                    .row
                    .into_iter()
                    .chain(once(SpringCondition::Unknown))
                    .cycle()
                    .take(row_len)
                    .collect::<Vec<_>>();

                // group sizes are repeated 5 times
                let group_sizes_len = record.group_sizes.len() * 5;
                let group_sizes = record
                    .group_sizes
                    .into_iter()
                    .cycle()
                    .take(group_sizes_len)
                    .collect::<Vec<_>>();

                num_arrangements(row, group_sizes, None)
            }
        }
    }

    println!("{result}");
    Ok(())
}

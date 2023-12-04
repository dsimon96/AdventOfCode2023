use std::io::stdin;

use anyhow::Result;
use clap::{Parser, Subcommand};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::digit1,
    combinator::{map, map_res, recognize},
    multi::separated_list1,
    IResult,
};

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    part: Part,
}

#[derive(Subcommand)]
enum Part {
    Part1 {
        #[arg(default_value_t = 12)]
        red: u32,
        #[arg(default_value_t = 13)]
        green: u32,
        #[arg(default_value_t = 14)]
        blue: u32,
    },
    Part2,
}

#[derive(Debug, PartialEq)]
enum Color {
    Red,
    Green,
    Blue,
}

type Subset = Vec<(Color, u32)>;

#[derive(Debug)]
struct Game {
    id: u32,
    subsets: Vec<Subset>,
}

fn game_header(input: &str) -> IResult<&str, u32> {
    let (input, _) = tag("Game ")(input)?;
    let (input, id) = map_res(recognize(digit1), str::parse::<u32>)(input)?;
    let (input, _) = tag(": ")(input)?;

    Ok((input, id))
}

fn red(input: &str) -> IResult<&str, Color> {
    map(tag("red"), |_| Color::Red)(input)
}

fn green(input: &str) -> IResult<&str, Color> {
    map(tag("green"), |_| Color::Green)(input)
}

fn blue(input: &str) -> IResult<&str, Color> {
    map(tag("blue"), |_| Color::Blue)(input)
}

fn color_amt(input: &str) -> IResult<&str, (Color, u32)> {
    let (input, num) = map_res(recognize(digit1), str::parse::<u32>)(input)?;
    let (input, _) = tag(" ")(input)?;
    let (input, color) = alt((red, green, blue))(input)?;

    Ok((input, (color, num)))
}

fn color_amts(input: &str) -> IResult<&str, Vec<(Color, u32)>> {
    separated_list1(tag(", "), color_amt)(input)
}

fn subsets(input: &str) -> IResult<&str, Vec<Subset>> {
    separated_list1(tag("; "), color_amts)(input)
}

fn game(input: &str) -> IResult<&str, Game> {
    let (input, id) = game_header(input)?;
    let (input, subsets) = subsets(input)?;

    Ok((input, Game { id, subsets }))
}

fn game_is_possible(game: &Game, max_red: u32, max_green: u32, max_blue: u32) -> bool {
    game.subsets.iter().all(|subset| {
        subset.iter().all(|(color, num)| match color {
            Color::Red => *num <= max_red,
            Color::Green => *num <= max_green,
            Color::Blue => *num <= max_blue,
        })
    })
}

fn min_cubes(game: &Game) -> Vec<(Color, u32)> {
    let mut res = Vec::new();
    for c in [Color::Red, Color::Green, Color::Blue] {
        let max_num = game
            .subsets
            .iter()
            .map(|subset| {
                subset
                    .into_iter()
                    .filter_map(|(color, num)| if *color == c { Some(num) } else { None })
                    .sum::<u32>()
            })
            .max()
            .unwrap();

        res.push((c, max_num));
    }

    res
}

fn main() -> Result<()> {
    let args = Args::parse();

    let res: u32 = match args.part {
        Part::Part1 { red, green, blue } => stdin()
            .lines()
            .filter_map(|l| {
                let line = l.unwrap();
                let g = game(&line).unwrap().1;
                if game_is_possible(&g, red, green, blue) {
                    Some(g.id)
                } else {
                    None
                }
            })
            .sum(),
        Part::Part2 => stdin()
            .lines()
            .map(|l| {
                let line = l.unwrap();
                let g = game(&line).unwrap().1;
                let min_possible = min_cubes(&g);

                min_possible
                    .into_iter()
                    .map(|(_, num)| num)
                    .product::<u32>()
            })
            .sum(),
    };

    println!("{res}");
    Ok(())
}

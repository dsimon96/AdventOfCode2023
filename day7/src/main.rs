use std::{io::stdin, mem::replace};

use anyhow::Result;
use clap::{Parser, Subcommand};
use counter::Counter;
use nom::{
    character::complete::{anychar, digit1, space1},
    combinator::{map_res, recognize},
    multi::count,
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

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
enum Card {
    Joker,
    N2,
    N3,
    N4,
    N5,
    N6,
    N7,
    N8,
    N9,
    T,
    J,
    Q,
    K,
    A,
}

#[derive(Error, Debug)]
enum ParseCardError {
    #[error("Character `{0}` does not correspond to a Card")]
    InvalidChar(char),
}

impl TryFrom<char> for Card {
    type Error = ParseCardError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '2' => Ok(Card::N2),
            '3' => Ok(Card::N3),
            '4' => Ok(Card::N4),
            '5' => Ok(Card::N5),
            '6' => Ok(Card::N6),
            '7' => Ok(Card::N7),
            '8' => Ok(Card::N8),
            '9' => Ok(Card::N9),
            'T' => Ok(Card::T),
            'J' => Ok(Card::J),
            'Q' => Ok(Card::Q),
            'K' => Ok(Card::K),
            'A' => Ok(Card::A),
            c => Err(ParseCardError::InvalidChar(c)),
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum HandType {
    HighCard,
    OnePair,
    TwoPair,
    ThreeOfAKind,
    FullHouse,
    FourOfAKind,
    FiveOfAKind,
}

fn get_type(cards: &[Card; 5]) -> HandType {
    let num_jokers = cards.iter().filter(|&&c| c == Card::Joker).count();
    let counter: Counter<Card> = cards
        .iter()
        .copied()
        .filter(|&c| c != Card::Joker)
        .collect::<Counter<_>>();
    let mut counts: Vec<usize> = counter.values().copied().collect();
    counts.sort_unstable();
    if let Some(last) = counts.last_mut() {
        *last += num_jokers;
    } else {
        counts.push(num_jokers);
    }

    match counts[..] {
        [1, 1, 1, 1, 1] => HandType::HighCard,
        [1, 1, 1, 2] => HandType::OnePair,
        [1, 2, 2] => HandType::TwoPair,
        [1, 1, 3] => HandType::ThreeOfAKind,
        [2, 3] => HandType::FullHouse,
        [1, 4] => HandType::FourOfAKind,
        [5] => HandType::FiveOfAKind,
        _ => unreachable!(), // counts must add up to five
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Hand {
    t: HandType,
    cards: [Card; 5],
    bid: usize,
}

fn cards(input: &str) -> IResult<&str, Vec<Card>> {
    count(map_res(anychar, Card::try_from), 5)(input)
}

fn bid(input: &str) -> IResult<&str, usize> {
    map_res(recognize(digit1), str::parse)(input)
}

fn hand<'a>(input: &'a str, part: &Part) -> IResult<&'a str, Hand> {
    let (input, (cards, bid)) = separated_pair(cards, space1, bid)(input)?;

    let mut cards: [Card; 5] = cards[..].try_into().expect("Invalid hand length");

    if let Part::Part2 = part {
        for i in 0..5 {
            if cards[i] == Card::J {
                let _ = replace(&mut cards[i], Card::Joker);
            }
        }
    }

    Ok((
        input,
        Hand {
            t: get_type(&cards),
            cards,
            bid,
        },
    ))
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut hands: Vec<Hand> = Vec::new();
    for line in stdin().lines() {
        let line = line?;
        let (_, hand) = hand(&line, &args.part).map_err(|e| e.to_owned())?;
        hands.push(hand);
    }
    hands.sort_unstable();

    let res: usize = hands
        .into_iter()
        .enumerate()
        .map(|(i, hand)| (i + 1) * hand.bid)
        .sum();

    println!("{res}");
    Ok(())
}

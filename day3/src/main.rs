use std::{collections::HashSet, io::stdin};

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

type Grid = Vec<Vec<char>>;
type Coord = (usize, usize);

struct SymbolCoord {
    coord: Coord,
    symbol: char,
}

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
struct PartNumber {
    start: Coord,
    len: usize,
}

fn get_symbol_coords(grid: &Grid) -> Vec<SymbolCoord> {
    grid.iter()
        .enumerate()
        .flat_map(|(i, row)| {
            row.iter()
                .enumerate()
                .filter_map(|(j, &c)| {
                    if c.is_ascii_digit() || c == '.' {
                        None
                    } else {
                        Some(SymbolCoord {
                            coord: (i, j),
                            symbol: c,
                        })
                    }
                })
                .collect::<Vec<SymbolCoord>>()
        })
        .collect()
}

fn get_adjacent_part_numbers(grid: &Grid, coord: &Coord) -> HashSet<PartNumber> {
    let (r, c) = *coord;
    let mut res = HashSet::new();
    for dr in -1..2 {
        if r == 0 && dr < 0 {
            continue;
        } else if r == grid.len() - 1 && dr > 0 {
            continue;
        }

        let r = r.checked_add_signed(dr).unwrap();
        let row = &grid[r];

        for dc in -1..2 {
            if c == 0 && dc < 0 {
                continue;
            } else if c == row.len() - 1 && dc > 0 {
                continue;
            } else if dr == 0 && dc == 0 {
                continue;
            }

            let mut startc = c.checked_add_signed(dc).unwrap();
            let mut endc = startc;
            if !row[startc].is_ascii_digit() {
                continue;
            }

            while startc > 0 && row[startc - 1].is_ascii_digit() {
                startc -= 1;
            }

            while endc < row.len() - 1 && row[endc + 1].is_ascii_digit() {
                endc += 1;
            }

            res.insert(PartNumber {
                start: (r, startc),
                len: endc - startc + 1,
            });
        }
    }

    res
}

fn get_part_number_value(grid: &Grid, part_number: &PartNumber) -> u32 {
    let mut val: u32 = 0;
    let (r, c) = part_number.start;
    for dc in 0..part_number.len {
        val *= 10;
        val += grid[r][c + dc].to_digit(10).unwrap();
    }

    val
}

fn main() {
    let args = Args::parse();

    let grid: Grid = stdin()
        .lines()
        .map(Result::unwrap)
        .map(|l| l.chars().collect())
        .collect();

    let symbol_coords: Vec<SymbolCoord> = get_symbol_coords(&grid);

    let mut sum: u32 = 0;
    match args.part {
        Part::Part1 => {
            let mut part_numbers: HashSet<PartNumber> = HashSet::new();
            for SymbolCoord { coord, symbol: _ } in symbol_coords {
                for part_number in get_adjacent_part_numbers(&grid, &coord) {
                    if part_numbers.insert(part_number) {
                        sum += get_part_number_value(&grid, &part_number)
                    }
                }
            }
        }
        Part::Part2 => {
            for SymbolCoord { coord, symbol } in symbol_coords {
                if symbol != '*' {
                    continue;
                }
                let adjacent_part_numbers = get_adjacent_part_numbers(&grid, &coord);
                if adjacent_part_numbers.len() != 2 {
                    continue;
                }

                sum += adjacent_part_numbers
                    .iter()
                    .map(|part_number| get_part_number_value(&grid, &part_number))
                    .product::<u32>();
            }
        }
    }

    println!("{sum}");
}

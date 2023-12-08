use std::{
    collections::VecDeque,
    io::{stdin, BufRead, Lines},
    mem::replace,
};

use clap::{Parser, Subcommand};
use itertools::Itertools;
use nom::{
    bytes::complete::tag,
    character::complete::{digit1, space1},
    combinator::{map_res, recognize},
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
    Part1,
    Part2,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Range {
    start: usize,
    len: usize,
}

struct RangeMapEntry {
    dest_start: usize,
    source_start: usize,
    len: usize,
}

struct RangeMap {
    entries: Vec<RangeMapEntry>,
}

impl RangeMap {
    fn get(&self, num: usize) -> usize {
        for entry in self.entries.iter() {
            if num < entry.source_start {
                continue;
            }
            let offset = num - entry.source_start;
            if offset < entry.len {
                return entry.dest_start + offset;
            }
        }

        num
    }
}

struct Input {
    seeds: Vec<usize>,
    seed_to_soil: RangeMap,
    soil_to_fertilizer: RangeMap,
    fertilizer_to_water: RangeMap,
    water_to_light: RangeMap,
    light_to_temperature: RangeMap,
    temperature_to_humidity: RangeMap,
    humidity_to_location: RangeMap,
}

fn num(input: &str) -> IResult<&str, usize> {
    map_res(recognize(digit1), str::parse)(input)
}

fn seeds(input: &str) -> IResult<&str, Vec<usize>> {
    let (input, _) = tag("seeds: ")(input)?;
    separated_list1(space1, num)(input)
}

fn range_map_entry(input: &str) -> IResult<&str, RangeMapEntry> {
    let (input, dest_start) = num(input)?;
    let (input, _) = space1(input)?;
    let (input, source_start) = num(input)?;
    let (input, _) = space1(input)?;
    let (input, len) = num(input)?;

    Ok((
        input,
        RangeMapEntry {
            dest_start,
            source_start,
            len,
        },
    ))
}

fn range_map<B>(input: &mut Lines<B>) -> IResult<&mut Lines<B>, RangeMap>
where
    B: BufRead + std::fmt::Debug,
{
    input.next(); // skip header line

    let mut entries = Vec::new();
    loop {
        let line = input.next();
        if line.is_none() {
            break;
        }
        let line = line.unwrap().unwrap();
        if line.len() < 1 {
            break;
        }

        let (_, entry) = range_map_entry(&line).unwrap();

        entries.push(entry)
    }

    Ok((input, RangeMap { entries }))
}

fn get_input<B>(input: &mut Lines<B>) -> IResult<&mut Lines<B>, Input>
where
    B: BufRead + std::fmt::Debug,
{
    let (_, seeds) = seeds(&input.next().unwrap().unwrap()).unwrap();
    input.next();

    let (_, seed_to_soil) = range_map(input).unwrap();
    let (_, soil_to_fertilizer) = range_map(input).unwrap();
    let (_, fertilizer_to_water) = range_map(input).unwrap();
    let (_, water_to_light) = range_map(input).unwrap();
    let (_, light_to_temperature) = range_map(input).unwrap();
    let (_, temperature_to_humidity) = range_map(input).unwrap();
    let (_, humidity_to_location) = range_map(input).unwrap();

    Ok((
        input,
        Input {
            seeds,
            seed_to_soil,
            soil_to_fertilizer,
            fertilizer_to_water,
            water_to_light,
            light_to_temperature,
            temperature_to_humidity,
            humidity_to_location,
        },
    ))
}

fn apply(nums: Vec<usize>, map: &RangeMap) -> Vec<usize> {
    nums.into_iter().map(|num| map.get(num)).collect()
}

fn consolidate_ranges(mut ranges: Vec<Range>) -> Vec<Range> {
    ranges.sort_unstable();

    let mut res = VecDeque::new();

    let opt = ranges.pop();
    let mut next = if let Some(range) = opt {
        range
    } else {
        return ranges;
    };

    while let Some(mut cur) = ranges.pop() {
        if cur.start + cur.len == next.start {
            cur.len += next.len;
            let _ = replace(&mut next, cur);
        } else {
            let x = replace(&mut next, cur);
            res.push_front(x);
        }
    }
    res.push_front(next);
    res.into()
}

fn to_ranges(nums: Vec<usize>) -> Vec<Range> {
    let res = nums
        .into_iter()
        .tuples()
        .map(|(start, len)| Range { start, len })
        .collect();

    consolidate_ranges(res)
}

fn apply_range(mut ranges: Vec<Range>, map: &RangeMap) -> Vec<Range> {
    let mut res = Vec::new();

    while let Some(range) = ranges.pop() {
        let mut found_overlap: bool = false;
        for entry in &map.entries {
            if range.start < entry.source_start + entry.len
                && entry.source_start < range.start + range.len
            {
                found_overlap = true;

                // apply mapping to the overlapping portion, and add any remaining unmapped portions back to `ranges`
                let overlap_start = range.start.max(entry.source_start);
                let overlap_offset = overlap_start - entry.source_start;
                let range_end = range.start + range.len;
                let overlap_end = (range_end).min(entry.source_start + entry.len);

                res.push(Range {
                    start: entry.dest_start + overlap_offset,
                    len: overlap_end - overlap_start,
                });

                if overlap_start > range.start {
                    ranges.push(Range {
                        start: range.start,
                        len: overlap_start - range.start,
                    });
                }
                if overlap_end < range_end {
                    ranges.push(Range {
                        start: overlap_end,
                        len: range_end - overlap_end,
                    });
                }

                break;
            }
        }
        if !found_overlap {
            res.push(range)
        }
    }

    consolidate_ranges(res)
}

fn main() {
    let args = Args::parse();

    let (_, input) = get_input(&mut stdin().lines()).unwrap();

    let min_loc = match args.part {
        Part::Part1 => {
            let seeds = input.seeds;
            let soils = apply(seeds, &input.seed_to_soil);
            let fertilizers = apply(soils, &input.soil_to_fertilizer);
            let waters = apply(fertilizers, &input.fertilizer_to_water);
            let lights = apply(waters, &input.water_to_light);
            let temperatures = apply(lights, &input.light_to_temperature);
            let humidities = apply(temperatures, &input.temperature_to_humidity);
            let locations = apply(humidities, &input.humidity_to_location);

            *locations.iter().min().unwrap()
        }
        Part::Part2 => {
            let seeds = to_ranges(input.seeds);
            let soils = apply_range(seeds, &input.seed_to_soil);
            let fertilizers = apply_range(soils, &input.soil_to_fertilizer);
            let waters = apply_range(fertilizers, &input.fertilizer_to_water);
            let lights = apply_range(waters, &input.water_to_light);
            let temperatures = apply_range(lights, &input.light_to_temperature);
            let humidities = apply_range(temperatures, &input.temperature_to_humidity);
            let locations = apply_range(humidities, &input.humidity_to_location);

            locations.first().unwrap().start
        }
    };

    println!("{min_loc}")
}

use std::{io::stdin, ops::RangeInclusive};

use anyhow::Result;
use clap::{Parser, Subcommand};
use good_lp::{constraint, default_solver, variable, Expression, ProblemVariables, SolverModel};
use itertools::Itertools;
use nalgebra::{convert, vector, Matrix2, Vector2, Vector3, LU};
use nom::{
    character::complete::{char, digit1, multispace1},
    combinator::{map, map_res, recognize},
    sequence::{separated_pair, tuple},
    IResult,
};

#[derive(Debug, Parser)]
struct Args {
    #[command(subcommand)]
    part: Part,
}

type Num = i64;

fn num(input: &str) -> IResult<&str, Num> {
    let (input, test) = nom::combinator::opt(char('-'))(input)?;
    let (input, val) = map_res(recognize(digit1), str::parse::<Num>)(input)?;

    let val = if test.is_some() { -val } else { val };
    Ok((input, val))
}

#[derive(Debug, Subcommand)]
enum Part {
    Part1 {
        #[arg(default_value_t = 200000000000000)]
        lb: Num,
        #[arg(default_value_t = 400000000000000)]
        ub: Num,
    },
    Part2,
}

fn vec3(input: &str) -> IResult<&str, Vector3<Num>> {
    let (input, (x, _, _, y, _, _, z)) = tuple((
        num,
        char(','),
        multispace1,
        num,
        char(','),
        multispace1,
        num,
    ))(input)?;

    Ok((input, vector![x, y, z]))
}

#[derive(Debug)]
struct Hailstone {
    position: Vector3<Num>,
    velocity: Vector3<Num>,
}

fn hailstone(input: &str) -> IResult<&str, Hailstone> {
    map(
        separated_pair(vec3, tuple((multispace1, char('@'), multispace1)), vec3),
        |(position, velocity)| Hailstone { position, velocity },
    )(input)
}

fn parse_input() -> Result<Vec<Hailstone>> {
    let mut res = Vec::new();
    for line in stdin().lines() {
        let line = line?;
        let (_, hailstone) = hailstone(&line).map_err(|e| e.to_owned())?;
        res.push(hailstone);
    }

    Ok(res)
}

fn has_intersection(a: &Hailstone, b: &Hailstone, lb: Num, ub: Num) -> bool {
    let a_pos: Vector2<f64> = convert(a.position.xy());
    let b_pos: Vector2<f64> = convert(b.position.xy());
    let k = b_pos - a_pos;
    let a_vel: Vector2<f64> = convert(a.velocity.xy());
    let b_vel: Vector2<f64> = convert(b.velocity.xy());

    let m = Matrix2::from_columns(&[a_vel, -b_vel]);

    let Some(t) = LU::new(m).solve(&k) else {
        return false;
    };
    if t.x < 0. || t.y < 0. {
        return false;
    }

    let d = (t.x * a_vel) + a_pos;

    let range = (lb as f64)..=(ub as f64);
    range.contains(&d.x) && range.contains(&d.y)
}

fn main() -> Result<()> {
    let args = Args::parse();

    let hailstones = parse_input()?;

    let res = match args.part {
        Part::Part1 { lb, ub } => hailstones
            .iter()
            .tuple_combinations::<(_, _)>()
            .filter(|&(a, b)| has_intersection(a, b, lb, ub))
            .count(),
        Part::Part2 => {
            let mut vars = ProblemVariables::new();
            let px = vars.add(variable().integer().name("px"));
            let py = vars.add(variable().integer().name("py"));
            let pz = vars.add(variable().integer().name("pz"));
            let vx = vars.add(variable().integer().name("vx"));
            let vy = vars.add(variable().integer().name("vy"));
            let vz = vars.add(variable().integer().name("vz"));
            let ts = vars.add_vector(variable().integer().min(0), hailstones.len());

            let mut model = vars
                .minimise(ts.iter().sum::<Expression>())
                .using(default_solver);

            for (hailstone, t) in hailstones.into_iter().zip(ts) {
                let hpx = hailstone.position.x;
                let hpy = hailstone.position.y;
                let hpz = hailstone.position.z;
                let hvx = hailstone.velocity.x;
                let hvy = hailstone.velocity.y;
                let hvz = hailstone.velocity.z;
                // model.add_constraint(constraint::eq(hvx * t + hpx, vx * t + px));
                // model.add_constraint(constraint::eq(hvy * t + hpy, vy * t + py));
                // model.add_constraint(constraint::eq(hvz * t + hpz, vz * t + pz));
            }

            todo!()
        }
    };

    println!("{res}");

    Ok(())
}

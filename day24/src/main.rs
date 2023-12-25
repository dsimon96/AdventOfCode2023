use std::io::stdin;

use anyhow::{Result, bail};
use clap::{Parser, Subcommand};
use itertools::Itertools;
use nalgebra::{convert, vector, Matrix2, Vector2, Vector3, LU};
use nom::{
    character::complete::{char, digit1, multispace1},
    combinator::{map, map_res, recognize},
    sequence::{separated_pair, tuple},
    IResult,
};
use z3::{Config, Context, ast::{Int, Ast}, Solver, SatResult};

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
            let cfg = Config::new();
            let ctx = Context::new(&cfg);
            let solver = Solver::new(&ctx);
            let px = Int::new_const(&ctx, "px");
            let py = Int::new_const(&ctx, "py");
            let pz = Int::new_const(&ctx, "pz");
            let vx = Int::new_const(&ctx, "vx");
            let vy = Int::new_const(&ctx, "vy");
            let vz = Int::new_const(&ctx, "vz");

            for (i, hailstone) in hailstones.into_iter().enumerate() {
                let t = Int::new_const(&ctx, format!("t{i}"));
                solver.assert(&t.ge(&Int::from_i64(&ctx, 0)) );
                let hx = hailstone.velocity.x * &t + hailstone.position.x;
                let hy = hailstone.velocity.y * &t + hailstone.position.y;
                let hz = hailstone.velocity.z * &t + hailstone.position.z;
                let rx = &vx * &t + &px;
                let ry = &vy * &t + &py;
                let rz = &vz * &t + &pz;
                solver.assert(&hx._eq(&rx));
                solver.assert(&hy._eq(&ry));
                solver.assert(&hz._eq(&rz));
            }

            let SatResult::Sat = solver.check() else {
                bail!("Unsolvable!");
            };

            let model = solver.get_model().unwrap();
            let px = model.get_const_interp(&px).and_then(|ast| ast.as_i64()).unwrap();
            let py = model.get_const_interp(&py).and_then(|ast| ast.as_i64()).unwrap();
            let pz = model.get_const_interp(&pz).and_then(|ast| ast.as_i64()).unwrap();

            (px + py + pz).try_into()?
        }
    };

    println!("{res}");

    Ok(())
}

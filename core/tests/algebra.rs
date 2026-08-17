use kosmos_core::calculus::differentiate;
use kosmos_core::eval::Program;
use kosmos_core::expr::parse;
use kosmos_core::{SLOT_X, VARS};

const TABLE: &str = include_str!("../../cas/expected.txt");

const PROBES: [f64; 9] = [-2.3, -1.7, -0.9, -0.35, 0.21, 0.64, 1.3, 2.4, 3.7];

struct Row<'a> {
    method: &'a str,
    input: &'a str,
    output: &'a str,
}

fn rows() -> Vec<Row<'static>> {
    TABLE
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let mut parts = line.split(" | ");
            let method = parts.next().expect("method").trim();
            let input = parts.next().expect("input").trim();
            let output = parts.next().unwrap_or("").trim();
            Row {
                method,
                input,
                output,
            }
        })
        .collect()
}

fn program(source: &str) -> Program {
    let ast =
        parse(source, VARS).unwrap_or_else(|e| panic!("the engine cannot parse {source:?}: {e}"));
    Program::compile(&ast)
}

fn slope(source: &str) -> Program {
    let ast =
        parse(source, VARS).unwrap_or_else(|e| panic!("the engine cannot parse {source:?}: {e}"));
    Program::compile(&differentiate(&ast, SLOT_X as u32))
}

fn at(program: &Program, x: f64) -> f64 {
    let mut vars = vec![0.0; VARS.len()];
    vars[SLOT_X] = x;
    program.eval(&vars)
}

fn agree(left: &Program, right: &Program, label: &str) {
    let mut compared = 0;
    for x in PROBES {
        let a = at(left, x);
        let b = at(right, x);
        if !a.is_finite() || !b.is_finite() {
            continue;
        }
        assert!(
            (a - b).abs() <= 1e-6 * (1.0 + b.abs()),
            "{label} at x={x}\n  engine: {a}\n  algebra: {b}"
        );
        compared += 1;
    }
    assert!(compared >= 2, "{label}: nothing comparable was sampled");
}

/// Everything the OCaml module can print must be readable by the Rust parser.
///
/// The two languages carry their own lexer and printer, so this is the only
/// thing keeping the surface syntax from drifting apart.
#[test]
fn the_engine_can_read_every_answer() {
    let mut checked = 0usize;
    for row in rows() {
        if row.output.starts_with('!') || row.method == "solve" {
            continue;
        }
        program(row.output);
        checked += 1;
    }
    assert!(
        checked >= 30,
        "expected the full table, read {checked} rows"
    );
}

/// An antiderivative is only worth printing if differentiating it gets the
/// integrand back. The engine checks that with its own differentiator, so a
/// mistake in either implementation shows up here.
#[test]
fn every_integral_differentiates_back() {
    let mut checked = 0usize;
    for row in rows() {
        if row.method != "integral" || row.output.starts_with('!') {
            continue;
        }
        agree(
            &program(row.input),
            &slope(row.output),
            &format!("d/dx of the integral of {}", row.input),
        );
        checked += 1;
    }
    assert!(checked >= 15, "expected many integrals, checked {checked}");
}

/// Simplifying and differentiating must not change what a formula means.
#[test]
fn rewrites_keep_their_meaning() {
    let mut checked = 0usize;
    for row in rows() {
        if row.output.starts_with('!') {
            continue;
        }
        match row.method {
            "simplify" => agree(
                &program(row.input),
                &program(row.output),
                &format!("simplify {}", row.input),
            ),
            "derivative" => agree(
                &slope(row.input),
                &program(row.output),
                &format!("d/dx {}", row.input),
            ),
            _ => continue,
        }
        checked += 1;
    }
    assert!(checked >= 8, "expected rewrites, checked {checked}");
}

/// A Taylor polynomial has to hug the function it came from near its centre,
/// and the corpus only contains series taken about a point the engine can reach.
#[test]
fn every_series_hugs_its_function() {
    let mut checked = 0usize;
    for row in rows() {
        if row.method != "taylor" || row.output.starts_with('!') {
            continue;
        }
        let mut parts = row.input.split(" @ ");
        let source = parts.next().expect("source").trim();
        let about: f64 = parts
            .next()
            .expect("centre")
            .trim()
            .parse()
            .expect("centre");

        let exact = program(source);
        let series = program(row.output);

        for step in -2..=2 {
            let x = about + f64::from(step) * 0.08;
            let a = at(&exact, x);
            let b = at(&series, x);
            assert!(
                (a - b).abs() <= 1e-4 * (1.0 + a.abs()),
                "the series for {source} drifts at x={x}\n  exact: {a}\n  series: {b}"
            );
        }
        checked += 1;
    }
    assert!(checked >= 4, "expected several series, checked {checked}");
}

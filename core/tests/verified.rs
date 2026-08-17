use kosmos_core::calculus::differentiate;
use kosmos_core::eval::Program;
use kosmos_core::expr::parse;
use kosmos_core::{SLOT_X, VARS};

const CASES: &str = include_str!("verified-derivatives.txt");

fn slope_at(source: &str, x: f64) -> f64 {
    let ast = parse(source, VARS).unwrap_or_else(|e| panic!("cannot parse {source:?}: {e}"));
    let derivative = differentiate(&ast, SLOT_X as u32);
    let program = Program::compile(&derivative);

    let mut vars = vec![0.0; VARS.len()];
    vars[SLOT_X] = x;
    program.eval(&vars)
}

fn value_at(source: &str, x: f64) -> f64 {
    let ast = parse(source, VARS).unwrap_or_else(|e| panic!("cannot parse {source:?}: {e}"));
    let program = Program::compile(&ast);

    let mut vars = vec![0.0; VARS.len()];
    vars[SLOT_X] = x;
    program.eval(&vars)
}

/// The engine's differentiation must agree with the rules proved correct in
/// `proofs/Proofs/Deriv.lean`.
///
/// The expected numbers in `verified-derivatives.txt` are produced by evaluating
/// the *verified* `derive`, so a disagreement here means the shipped rules and
/// the proved rules have drifted apart — which no amount of hand-written test
/// cases would reliably catch.
#[test]
fn matches_the_lean_proofs() {
    let mut checked = 0usize;
    let mut cases = 0usize;
    let mut expression: Option<String> = None;

    for (number, line) in CASES.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some(rest) = line.strip_prefix("case ") {
            let (source, derivative) = rest
                .split_once(" => ")
                .unwrap_or_else(|| panic!("line {}: malformed case", number + 1));

            // The derivative Lean printed must itself parse, so the corpus stays
            // inside the syntax the engine actually accepts.
            parse(derivative, VARS).unwrap_or_else(|e| {
                panic!(
                    "line {}: Lean printed {derivative:?}, which the engine cannot parse: {e}",
                    number + 1
                )
            });

            expression = Some(source.to_string());
            cases += 1;
            continue;
        }

        if let Some(rest) = line.strip_prefix("at ") {
            let source = expression
                .as_ref()
                .unwrap_or_else(|| panic!("line {}: sample before any case", number + 1));

            let mut parts = rest.split_whitespace();
            let x: f64 = parts.next().unwrap().parse().unwrap();
            let expected: f64 = parts.next().unwrap().parse().unwrap();

            let actual = slope_at(source, x);

            // Lean prints six decimals, so the corpus itself is only that precise.
            let tolerance = 1e-5 * (1.0 + expected.abs());
            assert!(
                (actual - expected).abs() <= tolerance,
                "line {}: d/dx {source} at x={x}\n  engine: {actual}\n  proved: {expected}",
                number + 1
            );

            checked += 1;
            continue;
        }

        panic!("line {}: unrecognised line {line:?}", number + 1);
    }

    assert!(cases >= 20, "expected the full corpus, saw {cases} cases");
    assert!(checked >= 140, "expected many samples, checked {checked}");
}

/// The derivative Lean printed and the derivative the engine computes must agree
/// as functions, not merely at the sample points Lean happened to choose.
#[test]
fn agrees_with_the_printed_derivative_everywhere() {
    let mut compared = 0usize;

    for line in CASES.lines() {
        let Some(rest) = line.trim().strip_prefix("case ") else {
            continue;
        };
        let (source, printed) = rest.split_once(" => ").unwrap();

        for step in -40..=40 {
            let x = step as f64 * 0.1;
            let engine = slope_at(source, x);
            let proved = value_at(printed, x);

            if !engine.is_finite() || !proved.is_finite() {
                continue;
            }

            let tolerance = 1e-9 * (1.0 + proved.abs());
            assert!(
                (engine - proved).abs() <= tolerance,
                "d/dx {source} at x={x}\n  engine: {engine}\n  proved: {proved}"
            );
            compared += 1;
        }
    }

    assert!(compared > 1000, "only compared {compared} points");
}

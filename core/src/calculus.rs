use crate::eval::Program;
use crate::expr::{BinOp, Expr, Func};

pub fn simplify(expr: &Expr) -> Expr {
    match expr {
        Expr::Number(_) | Expr::Var(_) => expr.clone(),

        Expr::Neg(inner) => {
            let inner = simplify(inner);
            match inner {
                Expr::Number(n) => Expr::Number(-n),
                Expr::Neg(deeper) => *deeper,
                other => Expr::Neg(Box::new(other)),
            }
        }

        Expr::Binary(op, a, b) => {
            let a = simplify(a);
            let b = simplify(b);

            if let (Some(x), Some(y)) = (a.as_number(), b.as_number()) {
                let folded = match op {
                    BinOp::Add => x + y,
                    BinOp::Sub => x - y,
                    BinOp::Mul => x * y,
                    BinOp::Div => x / y,
                    BinOp::Pow => x.powf(y),
                    BinOp::Mod => x.rem_euclid(y),
                };
                if folded.is_finite() {
                    return Expr::Number(folded);
                }
            }

            match op {
                BinOp::Add => {
                    if a.is_number(0.0) {
                        return b;
                    }
                    if b.is_number(0.0) {
                        return a;
                    }
                }
                BinOp::Sub => {
                    if b.is_number(0.0) {
                        return a;
                    }
                    if a.is_number(0.0) {
                        return Expr::Neg(Box::new(b));
                    }
                }
                BinOp::Mul => {
                    if a.is_number(0.0) || b.is_number(0.0) {
                        return Expr::Number(0.0);
                    }
                    if a.is_number(1.0) {
                        return b;
                    }
                    if b.is_number(1.0) {
                        return a;
                    }
                    if a.is_number(-1.0) {
                        return Expr::Neg(Box::new(b));
                    }
                    if b.is_number(-1.0) {
                        return Expr::Neg(Box::new(a));
                    }
                }
                BinOp::Div => {
                    if b.is_number(1.0) {
                        return a;
                    }
                    if a.is_number(0.0) {
                        return Expr::Number(0.0);
                    }
                }
                BinOp::Pow => {
                    if b.is_number(1.0) {
                        return a;
                    }
                    if b.is_number(0.0) {
                        return Expr::Number(1.0);
                    }
                    if a.is_number(1.0) {
                        return Expr::Number(1.0);
                    }
                }
                BinOp::Mod => {}
            }

            Expr::binary(*op, a, b)
        }

        Expr::Call(func, args) => {
            let args: Vec<Expr> = args.iter().map(simplify).collect();
            if args.iter().all(|a| a.as_number().is_some()) {
                let values: Vec<f64> = args.iter().map(|a| a.as_number().unwrap()).collect();
                let program = Program::compile(&Expr::Call(*func, args.clone()));
                let _ = values;
                let value = program.eval(&[]);
                if value.is_finite() {
                    return Expr::Number(value);
                }
            }
            Expr::Call(*func, args)
        }
    }
}

pub fn differentiate(expr: &Expr, slot: u32) -> Expr {
    simplify(&derive(expr, slot))
}

fn mul(a: Expr, b: Expr) -> Expr {
    Expr::binary(BinOp::Mul, a, b)
}

fn div(a: Expr, b: Expr) -> Expr {
    Expr::binary(BinOp::Div, a, b)
}

fn add(a: Expr, b: Expr) -> Expr {
    Expr::binary(BinOp::Add, a, b)
}

fn sub(a: Expr, b: Expr) -> Expr {
    Expr::binary(BinOp::Sub, a, b)
}

fn pow(a: Expr, b: Expr) -> Expr {
    Expr::binary(BinOp::Pow, a, b)
}

fn call1(func: Func, a: Expr) -> Expr {
    Expr::Call(func, vec![a])
}

fn derive(expr: &Expr, slot: u32) -> Expr {
    if !expr.uses_var(slot) {
        return Expr::Number(0.0);
    }

    match expr {
        Expr::Number(_) => Expr::Number(0.0),

        Expr::Var(s) => Expr::Number(if *s == slot { 1.0 } else { 0.0 }),

        Expr::Neg(inner) => Expr::Neg(Box::new(derive(inner, slot))),

        Expr::Binary(op, a, b) => {
            let da = derive(a, slot);
            let db = derive(b, slot);
            match op {
                BinOp::Add => add(da, db),
                BinOp::Sub => sub(da, db),
                BinOp::Mul => add(mul(da, (**b).clone()), mul((**a).clone(), db)),
                BinOp::Div => div(
                    sub(mul(da, (**b).clone()), mul((**a).clone(), db)),
                    pow((**b).clone(), Expr::Number(2.0)),
                ),
                BinOp::Mod => da,
                BinOp::Pow => {
                    if !b.uses_var(slot) {
                        let reduced = sub((**b).clone(), Expr::Number(1.0));
                        mul(
                            mul((**b).clone(), pow((**a).clone(), simplify(&reduced))),
                            da,
                        )
                    } else {
                        let base = (**a).clone();
                        let exponent = (**b).clone();
                        let logarithmic = mul(db, call1(Func::Ln, base.clone()));
                        let power = mul(exponent.clone(), div(da, base.clone()));
                        mul(pow(base, exponent), add(logarithmic, power))
                    }
                }
            }
        }

        Expr::Call(func, args) => {
            let inner = args[0].clone();
            let di = derive(&inner, slot);

            let outer = match func {
                Func::Sin => call1(Func::Cos, inner.clone()),
                Func::Cos => Expr::Neg(Box::new(call1(Func::Sin, inner.clone()))),
                Func::Tan => div(
                    Expr::Number(1.0),
                    pow(call1(Func::Cos, inner.clone()), Expr::Number(2.0)),
                ),
                Func::Asin => div(
                    Expr::Number(1.0),
                    call1(
                        Func::Sqrt,
                        sub(Expr::Number(1.0), pow(inner.clone(), Expr::Number(2.0))),
                    ),
                ),
                Func::Acos => Expr::Neg(Box::new(div(
                    Expr::Number(1.0),
                    call1(
                        Func::Sqrt,
                        sub(Expr::Number(1.0), pow(inner.clone(), Expr::Number(2.0))),
                    ),
                ))),
                Func::Atan => div(
                    Expr::Number(1.0),
                    add(Expr::Number(1.0), pow(inner.clone(), Expr::Number(2.0))),
                ),
                Func::Sinh => call1(Func::Cosh, inner.clone()),
                Func::Cosh => call1(Func::Sinh, inner.clone()),
                Func::Tanh => sub(
                    Expr::Number(1.0),
                    pow(call1(Func::Tanh, inner.clone()), Expr::Number(2.0)),
                ),
                Func::Exp => call1(Func::Exp, inner.clone()),
                Func::Ln => div(Expr::Number(1.0), inner.clone()),
                Func::Log2 => div(
                    Expr::Number(1.0),
                    mul(inner.clone(), Expr::Number(std::f64::consts::LN_2)),
                ),
                Func::Log10 => div(
                    Expr::Number(1.0),
                    mul(inner.clone(), Expr::Number(std::f64::consts::LN_10)),
                ),
                Func::Sqrt => div(
                    Expr::Number(1.0),
                    mul(Expr::Number(2.0), call1(Func::Sqrt, inner.clone())),
                ),
                Func::Cbrt => div(
                    Expr::Number(1.0),
                    mul(
                        Expr::Number(3.0),
                        pow(call1(Func::Cbrt, inner.clone()), Expr::Number(2.0)),
                    ),
                ),
                Func::Abs => call1(Func::Sign, inner.clone()),
                Func::Sign | Func::Floor | Func::Ceil | Func::Round => Expr::Number(0.0),
                _ => return numeric_fallback(expr, slot),
            };

            mul(outer, di)
        }
    }
}

fn numeric_fallback(expr: &Expr, _slot: u32) -> Expr {
    let _ = expr;
    Expr::Number(f64::NAN)
}

pub struct Analysis {
    pub roots: Vec<f64>,
    pub extrema: Vec<(f64, f64, bool)>,
}

const SCAN_STEPS: usize = 4096;

fn bisect<F: Fn(f64) -> f64>(f: &F, mut lo: f64, mut hi: f64) -> Option<f64> {
    let mut flo = f(lo);
    let fhi = f(hi);
    if !flo.is_finite() || !fhi.is_finite() || flo * fhi > 0.0 {
        return None;
    }

    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        let fmid = f(mid);
        if !fmid.is_finite() {
            return None;
        }
        if fmid == 0.0 || (hi - lo).abs() < 1e-13 * (1.0 + mid.abs()) {
            return Some(mid);
        }
        if flo * fmid < 0.0 {
            hi = mid;
        } else {
            lo = mid;
            flo = fmid;
        }
    }
    Some(0.5 * (lo + hi))
}

pub fn find_roots<F: Fn(f64) -> f64>(f: F, from: f64, to: f64, limit: usize) -> Vec<f64> {
    let mut roots = Vec::new();
    let step = (to - from) / SCAN_STEPS as f64;
    let mut previous_x = from;
    let mut previous = f(from);

    for i in 1..=SCAN_STEPS {
        let x = from + step * i as f64;
        let value = f(x);

        if previous.is_finite() && value.is_finite() {
            let jump = (value - previous).abs();
            let scale = previous.abs().max(value.abs()).max(1.0);
            let is_pole = jump > scale * 8.0;

            if !is_pole {
                if value == 0.0 {
                    roots.push(x);
                } else if previous * value < 0.0 {
                    if let Some(root) = bisect(&f, previous_x, x) {
                        roots.push(root);
                    }
                }
            }
        }

        previous_x = x;
        previous = value;
        if roots.len() >= limit {
            break;
        }
    }

    roots.dedup_by(|a, b| (*a - *b).abs() < (to - from) * 1e-9);
    roots
}

pub fn integrate<F: Fn(f64) -> f64>(f: F, from: f64, to: f64, panels: usize) -> f64 {
    let panels = panels.max(2) & !1;
    let h = (to - from) / panels as f64;
    let mut total = f(from) + f(to);

    for i in 1..panels {
        let x = from + h * i as f64;
        let value = f(x);
        if !value.is_finite() {
            return f64::NAN;
        }
        total += value * if i % 2 == 1 { 4.0 } else { 2.0 };
    }

    total * h / 3.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{format, parse};

    const VARS: &[&str] = &["x", "y", "t"];

    fn d(source: &str) -> String {
        let expr = parse(source, VARS).unwrap();
        format(&differentiate(&expr, 0), VARS)
    }

    fn numeric(source: &str, at: f64) -> f64 {
        let expr = parse(source, VARS).unwrap();
        Program::compile(&differentiate(&expr, 0)).eval(&[at])
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-7
    }

    #[test]
    fn simple_derivatives() {
        assert_eq!(d("x"), "1");
        assert_eq!(d("5"), "0");
        assert_eq!(d("x^2"), "2*x");
        assert_eq!(d("3x"), "3");
        assert_eq!(d("x^3"), "3*x^2");
    }

    #[test]
    fn chain_and_product_rule() {
        assert!(close(numeric("sin(x^2)", 1.0), 2.0 * 1.0f64.cos()));
        assert!(close(
            numeric("x*sin(x)", 2.0),
            2.0f64.cos() * 2.0 + 2.0f64.sin()
        ));
        assert!(close(numeric("exp(2x)", 0.5), 2.0 * 1.0f64.exp()));
    }

    #[test]
    fn quotient_rule() {
        assert!(close(numeric("1/x", 2.0), -0.25));
        assert!(close(numeric("x/(x+1)", 1.0), 0.25));
    }

    #[test]
    fn variable_exponent() {
        assert!(close(numeric("x^x", 2.0), 4.0 * (2.0f64.ln() + 1.0)));
    }

    #[test]
    fn derivative_matches_finite_differences() {
        for source in ["sin(x)*cos(2x)", "sqrt(x^2+1)", "ln(x^2+3)", "tanh(x)"] {
            let expr = parse(source, VARS).unwrap();
            let value = Program::compile(&expr);
            let slope = Program::compile(&differentiate(&expr, 0));
            for at in [-1.7, -0.4, 0.6, 2.3] {
                let h = 1e-6;
                let numerical = (value.eval(&[at + h]) - value.eval(&[at - h])) / (2.0 * h);
                let symbolic = slope.eval(&[at]);
                assert!(
                    (numerical - symbolic).abs() < 1e-5,
                    "{source} at {at}: {numerical} vs {symbolic}"
                );
            }
        }
    }

    #[test]
    fn roots_are_found() {
        let expr = parse("x^2 - 4", VARS).unwrap();
        let program = Program::compile(&expr);
        let roots = find_roots(|x| program.eval(&[x]), -10.0, 10.0, 16);
        assert_eq!(roots.len(), 2);
        assert!(close(roots[0], -2.0));
        assert!(close(roots[1], 2.0));
    }

    #[test]
    fn poles_are_not_mistaken_for_roots() {
        let expr = parse("1/x", VARS).unwrap();
        let program = Program::compile(&expr);
        let roots = find_roots(|x| program.eval(&[x]), -5.0, 5.0, 16);
        assert!(roots.is_empty(), "found {roots:?}");
    }

    #[test]
    fn integration_is_accurate() {
        let expr = parse("sin(x)", VARS).unwrap();
        let program = Program::compile(&expr);
        let area = integrate(|x| program.eval(&[x]), 0.0, std::f64::consts::PI, 2000);
        assert!((area - 2.0).abs() < 1e-9, "{area}");
    }

    #[test]
    fn simplification_keeps_expressions_small() {
        let expr = parse("x^2 + 0*x + 1*x - 0", VARS).unwrap();
        assert_eq!(format(&simplify(&expr), VARS), "x^2 + x");
    }
}

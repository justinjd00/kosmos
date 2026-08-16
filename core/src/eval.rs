use crate::expr::{BinOp, Expr, Func};

pub const MAX_DEPTH: usize = 64;

#[derive(Debug, Clone, Copy)]
enum Op {
    Push(f64),
    Load(u32),
    Neg,
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    PowI(i32),
    Mod,
    Call1(Func),
    Call2(Func),
}

#[derive(Debug, Clone)]
pub struct Program {
    ops: Vec<Op>,
    depth: usize,
}

fn apply1(func: Func, a: f64) -> f64 {
    match func {
        Func::Sin => a.sin(),
        Func::Cos => a.cos(),
        Func::Tan => a.tan(),
        Func::Asin => a.asin(),
        Func::Acos => a.acos(),
        Func::Atan => a.atan(),
        Func::Sinh => a.sinh(),
        Func::Cosh => a.cosh(),
        Func::Tanh => a.tanh(),
        Func::Exp => a.exp(),
        Func::Ln => a.ln(),
        Func::Log2 => a.log2(),
        Func::Log10 => a.log10(),
        Func::Sqrt => a.sqrt(),
        Func::Cbrt => a.cbrt(),
        Func::Abs => a.abs(),
        Func::Sign => {
            if a > 0.0 {
                1.0
            } else if a < 0.0 {
                -1.0
            } else {
                0.0
            }
        }
        Func::Floor => a.floor(),
        Func::Ceil => a.ceil(),
        Func::Round => a.round(),
        _ => f64::NAN,
    }
}

fn apply2(func: Func, a: f64, b: f64) -> f64 {
    match func {
        Func::Atan2 => a.atan2(b),
        Func::Min => a.min(b),
        Func::Max => a.max(b),
        Func::Pow => a.powf(b),
        Func::Log => a.log(b),
        Func::Hypot => a.hypot(b),
        _ => f64::NAN,
    }
}

impl Program {
    pub fn compile(expr: &Expr) -> Program {
        let mut ops = Vec::new();
        emit(expr, &mut ops);
        let depth = stack_depth(&ops);
        Program { ops, depth }
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn len(&self) -> usize {
        self.ops.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    #[inline]
    pub fn eval(&self, vars: &[f64]) -> f64 {
        let mut stack = [0.0f64; MAX_DEPTH];
        let mut top = 0usize;

        for op in &self.ops {
            match *op {
                Op::Push(value) => {
                    stack[top] = value;
                    top += 1;
                }
                Op::Load(slot) => {
                    stack[top] = vars.get(slot as usize).copied().unwrap_or(f64::NAN);
                    top += 1;
                }
                Op::Neg => stack[top - 1] = -stack[top - 1],
                Op::Add => {
                    top -= 1;
                    stack[top - 1] += stack[top];
                }
                Op::Sub => {
                    top -= 1;
                    stack[top - 1] -= stack[top];
                }
                Op::Mul => {
                    top -= 1;
                    stack[top - 1] *= stack[top];
                }
                Op::Div => {
                    top -= 1;
                    stack[top - 1] /= stack[top];
                }
                Op::Pow => {
                    top -= 1;
                    stack[top - 1] = stack[top - 1].powf(stack[top]);
                }
                Op::PowI(exponent) => stack[top - 1] = stack[top - 1].powi(exponent),
                Op::Mod => {
                    top -= 1;
                    stack[top - 1] = stack[top - 1].rem_euclid(stack[top]);
                }
                Op::Call1(func) => stack[top - 1] = apply1(func, stack[top - 1]),
                Op::Call2(func) => {
                    top -= 1;
                    stack[top - 1] = apply2(func, stack[top - 1], stack[top]);
                }
            }
        }

        if top == 0 {
            f64::NAN
        } else {
            stack[top - 1]
        }
    }
}

fn emit(expr: &Expr, ops: &mut Vec<Op>) {
    match expr {
        Expr::Number(value) => ops.push(Op::Push(*value)),
        Expr::Var(slot) => ops.push(Op::Load(*slot)),
        Expr::Neg(inner) => {
            emit(inner, ops);
            ops.push(Op::Neg);
        }
        Expr::Binary(op, a, b) => {
            if *op == BinOp::Pow {
                if let Some(exponent) = integer_exponent(b) {
                    emit(a, ops);
                    ops.push(Op::PowI(exponent));
                    return;
                }
            }
            emit(a, ops);
            emit(b, ops);
            ops.push(match op {
                BinOp::Add => Op::Add,
                BinOp::Sub => Op::Sub,
                BinOp::Mul => Op::Mul,
                BinOp::Div => Op::Div,
                BinOp::Pow => Op::Pow,
                BinOp::Mod => Op::Mod,
            });
        }
        Expr::Call(func, args) => {
            for arg in args {
                emit(arg, ops);
            }
            ops.push(if args.len() == 2 {
                Op::Call2(*func)
            } else {
                Op::Call1(*func)
            });
        }
    }
}

fn integer_exponent(expr: &Expr) -> Option<i32> {
    let value = expr.as_number()?;
    if value.fract() == 0.0 && value.abs() <= 64.0 {
        Some(value as i32)
    } else {
        None
    }
}

fn stack_depth(ops: &[Op]) -> usize {
    let mut top = 0usize;
    let mut peak = 0usize;
    for op in ops {
        match op {
            Op::Push(_) | Op::Load(_) => {
                top += 1;
                peak = peak.max(top);
            }
            Op::Neg | Op::PowI(_) | Op::Call1(_) => {}
            _ => top = top.saturating_sub(1),
        }
    }
    peak
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::parse;

    const VARS: &[&str] = &["x", "y", "t"];

    fn eval(source: &str, values: &[f64]) -> f64 {
        let expr = parse(source, VARS).expect("parses");
        Program::compile(&expr).eval(values)
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-12
    }

    #[test]
    fn arithmetic() {
        assert!(close(eval("1 + 2 * 3", &[]), 7.0));
        assert!(close(eval("(1 + 2) * 3", &[]), 9.0));
        assert!(close(eval("2 ^ 3 ^ 2", &[]), 512.0));
        assert!(close(eval("10 / 4", &[]), 2.5));
        assert!(close(eval("7 % 3", &[]), 1.0));
    }

    #[test]
    fn variables_and_implicit_products() {
        assert!(close(eval("3x^2 + 2x + 1", &[2.0, 0.0, 0.0]), 17.0));
        assert!(close(eval("x*y", &[3.0, 4.0, 0.0]), 12.0));
        assert!(close(eval("2(x + y)", &[1.0, 2.0, 0.0]), 6.0));
    }

    #[test]
    fn functions() {
        assert!(close(eval("sin(0)", &[]), 0.0));
        assert!(close(eval("cos(0)", &[]), 1.0));
        assert!(close(eval("sqrt(16)", &[]), 4.0));
        assert!(close(eval("ln(e)", &[]), 1.0));
        assert!(close(eval("max(3, 7)", &[]), 7.0));
        assert!(close(eval("hypot(3, 4)", &[]), 5.0));
        assert!(close(eval("abs(-5)", &[]), 5.0));
    }

    #[test]
    fn unary_minus() {
        assert!(close(eval("-x^2", &[3.0]), -9.0));
        assert!(close(eval("(-x)^2", &[3.0]), 9.0));
        assert!(close(eval("-2^2", &[]), -4.0));
    }

    #[test]
    fn integer_powers_are_optimised() {
        let expr = parse("x^3", VARS).unwrap();
        let program = Program::compile(&expr);
        assert_eq!(program.len(), 2);
        assert!(close(program.eval(&[2.0]), 8.0));
        assert!(close(program.eval(&[-2.0]), -8.0));
    }

    #[test]
    fn stack_stays_within_bounds() {
        let deep = "1+".repeat(30) + "1";
        let expr = parse(&deep, VARS).unwrap();
        let program = Program::compile(&expr);
        assert!(program.depth() < MAX_DEPTH);
        assert!(close(program.eval(&[]), 31.0));
    }

    #[test]
    fn undefined_results_are_nan_not_panics() {
        assert!(eval("sqrt(-1)", &[]).is_nan());
        assert!(eval("ln(-1)", &[]).is_nan());
        assert!(eval("1/0", &[]).is_infinite());
    }
}

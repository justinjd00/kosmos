use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub message: String,
    pub at: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (at {})", self.message, self.at)
    }
}

impl ParseError {
    fn new(message: impl Into<String>, at: usize) -> Self {
        ParseError {
            message: message.into(),
            at,
        }
    }
}

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    Percent,
    LParen,
    RParen,
    Comma,
    Eof,
}

impl Token {
    fn starts_value(&self) -> bool {
        matches!(
            self,
            Token::Number(_) | Token::Ident(_) | Token::LParen | Token::Minus
        )
    }

    fn ends_value(&self) -> bool {
        matches!(self, Token::Number(_) | Token::Ident(_) | Token::RParen)
    }
}

#[derive(Debug, Clone)]
struct Lexeme {
    token: Token,
    at: usize,
}

fn lex(source: &str) -> ParseResult<Vec<Lexeme>> {
    let chars: Vec<char> = source.chars().collect();
    let mut out = Vec::new();
    let mut i = 0usize;

    while i < chars.len() {
        let c = chars[i];
        let at = i;

        if c.is_whitespace() {
            i += 1;
            continue;
        }

        if c.is_ascii_digit() || (c == '.' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit())
        {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            if i < chars.len() && (chars[i] == 'e' || chars[i] == 'E') {
                let save = i;
                i += 1;
                if i < chars.len() && (chars[i] == '+' || chars[i] == '-') {
                    i += 1;
                }
                if i < chars.len() && chars[i].is_ascii_digit() {
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        i += 1;
                    }
                } else {
                    i = save;
                }
            }
            let text: String = chars[start..i].iter().collect();
            let value = text
                .parse::<f64>()
                .map_err(|_| ParseError::new(format!("'{text}' is not a number"), start))?;
            out.push(Lexeme {
                token: Token::Number(value),
                at: start,
            });
            continue;
        }

        if c.is_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let text: String = chars[start..i].iter().collect();
            out.push(Lexeme {
                token: Token::Ident(text.to_ascii_lowercase()),
                at: start,
            });
            continue;
        }

        let token = match c {
            '+' => Token::Plus,
            '-' | '\u{2212}' => Token::Minus,
            '*' | '\u{00d7}' | '\u{22c5}' => Token::Star,
            '/' | '\u{00f7}' => Token::Slash,
            '^' => Token::Caret,
            '%' => Token::Percent,
            '(' | '[' | '{' => Token::LParen,
            ')' | ']' | '}' => Token::RParen,
            ',' | ';' => Token::Comma,
            _ => return Err(ParseError::new(format!("unexpected character '{c}'"), at)),
        };
        out.push(Lexeme { token, at });
        i += 1;
    }

    out.push(Lexeme {
        token: Token::Eof,
        at: chars.len(),
    });
    Ok(out)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Mod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Func {
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Sinh,
    Cosh,
    Tanh,
    Exp,
    Ln,
    Log2,
    Log10,
    Sqrt,
    Cbrt,
    Abs,
    Sign,
    Floor,
    Ceil,
    Round,
    Atan2,
    Min,
    Max,
    Pow,
    Log,
    Hypot,
}

impl Func {
    pub fn arity(self) -> usize {
        match self {
            Func::Atan2 | Func::Min | Func::Max | Func::Pow | Func::Log | Func::Hypot => 2,
            _ => 1,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Func::Sin => "sin",
            Func::Cos => "cos",
            Func::Tan => "tan",
            Func::Asin => "asin",
            Func::Acos => "acos",
            Func::Atan => "atan",
            Func::Sinh => "sinh",
            Func::Cosh => "cosh",
            Func::Tanh => "tanh",
            Func::Exp => "exp",
            Func::Ln => "ln",
            Func::Log2 => "log2",
            Func::Log10 => "log10",
            Func::Sqrt => "sqrt",
            Func::Cbrt => "cbrt",
            Func::Abs => "abs",
            Func::Sign => "sign",
            Func::Floor => "floor",
            Func::Ceil => "ceil",
            Func::Round => "round",
            Func::Atan2 => "atan2",
            Func::Min => "min",
            Func::Max => "max",
            Func::Pow => "pow",
            Func::Log => "log",
            Func::Hypot => "hypot",
        }
    }

    fn from_name(name: &str) -> Option<Func> {
        Some(match name {
            "sin" => Func::Sin,
            "cos" => Func::Cos,
            "tan" => Func::Tan,
            "asin" | "arcsin" => Func::Asin,
            "acos" | "arccos" => Func::Acos,
            "atan" | "arctan" => Func::Atan,
            "sinh" => Func::Sinh,
            "cosh" => Func::Cosh,
            "tanh" => Func::Tanh,
            "exp" => Func::Exp,
            "ln" => Func::Ln,
            "log2" | "lb" => Func::Log2,
            "log10" | "lg" => Func::Log10,
            "sqrt" => Func::Sqrt,
            "cbrt" => Func::Cbrt,
            "abs" => Func::Abs,
            "sign" | "sgn" => Func::Sign,
            "floor" => Func::Floor,
            "ceil" => Func::Ceil,
            "round" => Func::Round,
            "atan2" => Func::Atan2,
            "min" => Func::Min,
            "max" => Func::Max,
            "pow" => Func::Pow,
            "log" => Func::Log,
            "hypot" => Func::Hypot,
            _ => return None,
        })
    }
}

pub const PHI: f64 = 1.618_033_988_749_895;

fn constant(name: &str) -> Option<f64> {
    Some(match name {
        "pi" | "\u{03c0}" => std::f64::consts::PI,
        "tau" => std::f64::consts::TAU,
        "e" => std::f64::consts::E,
        "phi" => PHI,
        "inf" | "infinity" => f64::INFINITY,
        _ => return None,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64),
    Var(u32),
    Neg(Box<Expr>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    Call(Func, Vec<Expr>),
}

impl Expr {
    pub fn number(value: f64) -> Expr {
        Expr::Number(value)
    }

    pub fn binary(op: BinOp, left: Expr, right: Expr) -> Expr {
        Expr::Binary(op, Box::new(left), Box::new(right))
    }

    pub fn is_number(&self, value: f64) -> bool {
        matches!(self, Expr::Number(n) if (*n - value).abs() < f64::EPSILON)
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            Expr::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn uses_var(&self, slot: u32) -> bool {
        match self {
            Expr::Number(_) => false,
            Expr::Var(s) => *s == slot,
            Expr::Neg(inner) => inner.uses_var(slot),
            Expr::Binary(_, a, b) => a.uses_var(slot) || b.uses_var(slot),
            Expr::Call(_, args) => args.iter().any(|a| a.uses_var(slot)),
        }
    }
}

struct Parser<'a> {
    lexemes: Vec<Lexeme>,
    pos: usize,
    vars: &'a [&'a str],
}

fn precedence(op: BinOp) -> (u8, bool) {
    match op {
        BinOp::Add | BinOp::Sub => (1, true),
        BinOp::Mul | BinOp::Div | BinOp::Mod => (2, true),
        BinOp::Pow => (4, false),
    }
}

impl<'a> Parser<'a> {
    fn peek(&self) -> &Token {
        &self.lexemes[self.pos].token
    }

    fn peek_at(&self) -> usize {
        self.lexemes[self.pos].at
    }

    fn advance(&mut self) -> Token {
        let token = self.lexemes[self.pos].token.clone();
        if self.pos + 1 < self.lexemes.len() {
            self.pos += 1;
        }
        token
    }

    fn expect(&mut self, expected: Token, what: &str) -> ParseResult<()> {
        if *self.peek() == expected {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::new(format!("expected {what}"), self.peek_at()))
        }
    }

    fn parse_expr(&mut self, min_precedence: u8) -> ParseResult<Expr> {
        let mut left = self.parse_unary()?;

        loop {
            let op = match self.peek() {
                Token::Plus => Some(BinOp::Add),
                Token::Minus => Some(BinOp::Sub),
                Token::Star => Some(BinOp::Mul),
                Token::Slash => Some(BinOp::Div),
                Token::Caret => Some(BinOp::Pow),
                Token::Percent => Some(BinOp::Mod),
                _ => None,
            };

            if let Some(op) = op {
                let (prec, left_assoc) = precedence(op);
                if prec < min_precedence {
                    break;
                }
                self.advance();
                let next_min = if left_assoc { prec + 1 } else { prec };
                let right = self.parse_expr(next_min)?;
                left = Expr::binary(op, left, right);
                continue;
            }

            let implicit = self.pos > 0
                && self.lexemes[self.pos - 1].token.ends_value()
                && self.peek().starts_value()
                && !matches!(self.peek(), Token::Minus);

            if implicit {
                let (prec, _) = precedence(BinOp::Mul);
                if prec < min_precedence {
                    break;
                }
                let right = self.parse_expr(prec + 1)?;
                left = Expr::binary(BinOp::Mul, left, right);
                continue;
            }

            break;
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> ParseResult<Expr> {
        match self.peek() {
            Token::Minus => {
                self.advance();
                let inner = self.parse_expr(3)?;
                Ok(Expr::Neg(Box::new(inner)))
            }
            Token::Plus => {
                self.advance();
                self.parse_unary()
            }
            _ => self.parse_atom(),
        }
    }

    fn parse_atom(&mut self) -> ParseResult<Expr> {
        let at = self.peek_at();
        match self.advance() {
            Token::Number(value) => Ok(Expr::Number(value)),

            Token::LParen => {
                let inner = self.parse_expr(0)?;
                self.expect(Token::RParen, "a closing bracket")?;
                Ok(inner)
            }

            Token::Ident(name) => {
                if let Some(func) = Func::from_name(&name) {
                    if *self.peek() == Token::LParen {
                        self.advance();
                        let mut args = vec![self.parse_expr(0)?];
                        while *self.peek() == Token::Comma {
                            self.advance();
                            args.push(self.parse_expr(0)?);
                        }
                        self.expect(Token::RParen, "a closing bracket")?;
                        if args.len() != func.arity() {
                            return Err(ParseError::new(
                                format!(
                                    "{} takes {} argument(s), got {}",
                                    func.name(),
                                    func.arity(),
                                    args.len()
                                ),
                                at,
                            ));
                        }
                        return Ok(Expr::Call(func, args));
                    }
                    return Err(ParseError::new(
                        format!("{name} needs brackets, like {name}(x)"),
                        at,
                    ));
                }

                if let Some(slot) = self.vars.iter().position(|v| *v == name) {
                    return Ok(Expr::Var(slot as u32));
                }

                if let Some(value) = constant(&name) {
                    return Ok(Expr::Number(value));
                }

                Err(ParseError::new(format!("unknown name '{name}'"), at))
            }

            Token::Eof => Err(ParseError::new("the expression ends too early", at)),

            other => Err(ParseError::new(
                format!("unexpected {}", describe(&other)),
                at,
            )),
        }
    }
}

fn describe(token: &Token) -> String {
    match token {
        Token::Plus => "'+'".into(),
        Token::Minus => "'-'".into(),
        Token::Star => "'*'".into(),
        Token::Slash => "'/'".into(),
        Token::Caret => "'^'".into(),
        Token::Percent => "'%'".into(),
        Token::LParen => "'('".into(),
        Token::RParen => "')'".into(),
        Token::Comma => "','".into(),
        Token::Number(n) => format!("number {n}"),
        Token::Ident(name) => format!("'{name}'"),
        Token::Eof => "end of input".into(),
    }
}

pub fn parse(source: &str, vars: &[&str]) -> ParseResult<Expr> {
    let lexemes = lex(source)?;
    let mut parser = Parser {
        lexemes,
        pos: 0,
        vars,
    };
    let expr = parser.parse_expr(0)?;
    if *parser.peek() != Token::Eof {
        return Err(ParseError::new(
            format!("unexpected {}", describe(parser.peek())),
            parser.peek_at(),
        ));
    }
    Ok(expr)
}

pub fn format(expr: &Expr, vars: &[&str]) -> String {
    write_expr(expr, vars, 0)
}

fn write_expr(expr: &Expr, vars: &[&str], parent: u8) -> String {
    match expr {
        Expr::Number(n) => {
            if *n == std::f64::consts::PI {
                "pi".to_string()
            } else if n.fract() == 0.0 && n.abs() < 1e15 {
                format!("{}", *n as i64)
            } else {
                let text = format!("{n}");
                text
            }
        }
        Expr::Var(slot) => vars
            .get(*slot as usize)
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("v{slot}")),
        Expr::Neg(inner) => {
            let body = format!("-{}", write_expr(inner, vars, 3));
            if parent > 3 {
                format!("({body})")
            } else {
                body
            }
        }
        Expr::Binary(op, a, b) => {
            let (prec, left_assoc) = precedence(*op);
            let symbol = match op {
                BinOp::Add => " + ",
                BinOp::Sub => " - ",
                BinOp::Mul => "*",
                BinOp::Div => "/",
                BinOp::Pow => "^",
                BinOp::Mod => " mod ",
            };
            let (left_parent, right_parent) = if left_assoc {
                (prec, prec + 1)
            } else {
                (prec + 1, prec)
            };
            let left = write_expr(a, vars, left_parent);
            let right = write_expr(b, vars, right_parent);
            let body = format!("{left}{symbol}{right}");
            if prec < parent {
                format!("({body})")
            } else {
                body
            }
        }
        Expr::Call(func, args) => {
            let inner: Vec<String> = args.iter().map(|a| write_expr(a, vars, 0)).collect();
            format!("{}({})", func.name(), inner.join(", "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VARS: &[&str] = &["x", "y", "t"];

    fn p(source: &str) -> Expr {
        parse(source, VARS).expect("parses")
    }

    #[test]
    fn arithmetic_precedence() {
        assert_eq!(format(&p("1 + 2 * 3"), VARS), "1 + 2*3");
        assert_eq!(format(&p("(1 + 2) * 3"), VARS), "(1 + 2)*3");
        assert_eq!(format(&p("2 ^ 3 ^ 2"), VARS), "2^3^2");
        assert_eq!(format(&p("1 - 2 - 3"), VARS), "1 - 2 - 3");
    }

    #[test]
    fn implicit_multiplication() {
        assert_eq!(format(&p("2x"), VARS), "2*x");
        assert_eq!(format(&p("2(x + 1)"), VARS), "2*(x + 1)");
        assert_eq!(format(&p("(x+1)(x-1)"), VARS), "(x + 1)*(x - 1)");
        assert_eq!(format(&p("3x^2"), VARS), "3*x^2");
        assert_eq!(format(&p("2pi"), VARS), "2*pi");
    }

    #[test]
    fn unary_minus_binds_tighter_than_plus() {
        assert_eq!(format(&p("-x^2"), VARS), "-x^2");
        assert_eq!(format(&p("1 - -2"), VARS), "1 - -2");
    }

    #[test]
    fn functions_and_constants() {
        assert!(p("sin(x)").uses_var(0));
        assert!(p("atan2(y, x)").uses_var(1));
        assert_eq!(p("pi").as_number(), Some(std::f64::consts::PI));
        assert!(matches!(p("max(x, 3)"), Expr::Call(Func::Max, _)));
    }

    #[test]
    fn brackets_are_interchangeable() {
        assert_eq!(format(&p("[x + 1]"), VARS), "x + 1");
    }

    #[test]
    fn errors_point_at_the_problem() {
        let err = parse("sin(x", VARS).unwrap_err();
        assert!(err.message.contains("closing"));

        let err = parse("2 + $", VARS).unwrap_err();
        assert_eq!(err.at, 4);

        let err = parse("q + 1", VARS).unwrap_err();
        assert!(err.message.contains("unknown name"));

        let err = parse("sin x", VARS).unwrap_err();
        assert!(err.message.contains("brackets"));

        let err = parse("min(x)", VARS).unwrap_err();
        assert!(err.message.contains("2 argument"));
    }

    #[test]
    fn scientific_notation() {
        assert_eq!(p("1e-3").as_number(), Some(0.001));
        assert_eq!(p("2.5e2").as_number(), Some(250.0));
    }
}

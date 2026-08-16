pub mod calculus;
pub mod eval;
pub mod expr;
pub mod plot;

use wasm_bindgen::prelude::*;

use eval::Program;
use expr::{Expr, ParseError};
use plot::Viewport;

pub const VARS: &[&str] = &["x", "t", "a", "b", "c", "d"];
pub const SLOT_X: usize = 0;
pub const SLOT_T: usize = 1;
pub const PARAM_COUNT: usize = 4;

fn error_json(error: &ParseError) -> String {
    format!("{{\"message\":{:?},\"at\":{}}}", error.message, error.at)
}

#[wasm_bindgen]
pub struct Function {
    ast: Expr,
    program: Program,
    slope: Option<Program>,
    slope_ast: Option<Expr>,
    vars: Vec<f64>,
}

#[wasm_bindgen]
impl Function {
    #[wasm_bindgen(constructor)]
    pub fn new(source: &str) -> Result<Function, String> {
        let ast = expr::parse(source, VARS).map_err(|e| error_json(&e))?;
        let simplified = calculus::simplify(&ast);
        let program = Program::compile(&simplified);
        Ok(Function {
            ast: simplified,
            program,
            slope: None,
            slope_ast: None,
            vars: vec![0.0; VARS.len()],
        })
    }

    #[wasm_bindgen(js_name = setParam)]
    pub fn set_param(&mut self, index: usize, value: f64) {
        if index < PARAM_COUNT {
            self.vars[2 + index] = value;
        }
    }

    #[wasm_bindgen(js_name = setTime)]
    pub fn set_time(&mut self, value: f64) {
        self.vars[SLOT_T] = value;
    }

    #[wasm_bindgen(js_name = usesTime)]
    pub fn uses_time(&self) -> bool {
        self.ast.uses_var(SLOT_T as u32)
    }

    #[wasm_bindgen(js_name = usesParam)]
    pub fn uses_param(&self, index: usize) -> bool {
        index < PARAM_COUNT && self.ast.uses_var((2 + index) as u32)
    }

    pub fn eval(&self, x: f64) -> f64 {
        let mut vars = self.vars.clone();
        vars[SLOT_X] = x;
        self.program.eval(&vars)
    }

    #[wasm_bindgen(js_name = prettyPrint)]
    pub fn pretty_print(&self) -> String {
        expr::format(&self.ast, VARS)
    }

    fn ensure_slope(&mut self) {
        if self.slope.is_none() {
            let derivative = calculus::differentiate(&self.ast, SLOT_X as u32);
            self.slope = Some(Program::compile(&derivative));
            self.slope_ast = Some(derivative);
        }
    }

    #[wasm_bindgen(js_name = derivativeText)]
    pub fn derivative_text(&mut self) -> String {
        self.ensure_slope();
        expr::format(self.slope_ast.as_ref().unwrap(), VARS)
    }

    #[wasm_bindgen(js_name = slopeAt)]
    pub fn slope_at(&mut self, x: f64) -> f64 {
        self.ensure_slope();
        let mut vars = self.vars.clone();
        vars[SLOT_X] = x;
        self.slope.as_ref().unwrap().eval(&vars)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn sample(
        &self,
        x_min: f64,
        x_max: f64,
        y_min: f64,
        y_max: f64,
        width: f64,
        height: f64,
    ) -> Vec<f32> {
        let view = Viewport {
            x_min,
            x_max,
            y_min,
            y_max,
            width,
            height,
        };
        plot::sample(&self.program, view, &self.vars, SLOT_X).points
    }

    #[wasm_bindgen(js_name = sampleDerivative)]
    #[allow(clippy::too_many_arguments)]
    pub fn sample_derivative(
        &mut self,
        x_min: f64,
        x_max: f64,
        y_min: f64,
        y_max: f64,
        width: f64,
        height: f64,
    ) -> Vec<f32> {
        self.ensure_slope();
        let view = Viewport {
            x_min,
            x_max,
            y_min,
            y_max,
            width,
            height,
        };
        plot::sample(self.slope.as_ref().unwrap(), view, &self.vars, SLOT_X).points
    }

    pub fn roots(&self, from: f64, to: f64) -> Vec<f64> {
        let vars = self.vars.clone();
        let program = &self.program;
        calculus::find_roots(
            |x| {
                let mut local = vars.clone();
                local[SLOT_X] = x;
                program.eval(&local)
            },
            from,
            to,
            64,
        )
    }

    pub fn extrema(&mut self, from: f64, to: f64) -> Vec<f64> {
        self.ensure_slope();
        let vars = self.vars.clone();
        let slope = self.slope.as_ref().unwrap();

        let critical = calculus::find_roots(
            |x| {
                let mut local = vars.clone();
                local[SLOT_X] = x;
                slope.eval(&local)
            },
            from,
            to,
            64,
        );

        let mut out = Vec::with_capacity(critical.len() * 3);
        let step = (to - from) * 1e-5;

        for x in critical {
            let mut local = vars.clone();
            local[SLOT_X] = x;
            let y = self.program.eval(&local);
            if !y.is_finite() {
                continue;
            }

            local[SLOT_X] = x - step;
            let before = slope.eval(&local);
            local[SLOT_X] = x + step;
            let after = slope.eval(&local);

            let kind = if before > 0.0 && after < 0.0 {
                1.0
            } else if before < 0.0 && after > 0.0 {
                -1.0
            } else {
                0.0
            };

            if kind != 0.0 {
                out.push(x);
                out.push(y);
                out.push(kind);
            }
        }

        out
    }

    pub fn integral(&self, from: f64, to: f64) -> f64 {
        let vars = self.vars.clone();
        let program = &self.program;
        calculus::integrate(
            |x| {
                let mut local = vars.clone();
                local[SLOT_X] = x;
                program.eval(&local)
            },
            from,
            to,
            4096,
        )
    }
}

#[wasm_bindgen(js_name = niceStep)]
pub fn nice_step(span: f64, target_count: f64) -> f64 {
    plot::nice_step(span, target_count)
}

#[wasm_bindgen(js_name = checkSyntax)]
pub fn check_syntax(source: &str) -> String {
    match expr::parse(source, VARS) {
        Ok(_) => "{\"ok\":true}".to_string(),
        Err(error) => format!("{{\"ok\":false,{}}}", &error_json(&error)[1..]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn function_round_trip() {
        let mut f = Function::new("x^2 + 1").unwrap();
        assert_eq!(f.eval(3.0), 10.0);
        assert_eq!(f.derivative_text(), "2*x");
        assert_eq!(f.slope_at(2.0), 4.0);
    }

    #[test]
    fn parameters_and_time_are_wired_up() {
        let mut f = Function::new("a*sin(x + t)").unwrap();
        assert!(f.uses_time());
        assert!(f.uses_param(0));
        assert!(!f.uses_param(1));

        f.set_param(0, 2.0);
        f.set_time(0.0);
        assert!((f.eval(std::f64::consts::FRAC_PI_2) - 2.0).abs() < 1e-12);
    }

    #[test]
    fn syntax_errors_carry_a_position() {
        let report = check_syntax("2 + $");
        assert!(report.contains("\"ok\":false"));
        assert!(report.contains("\"at\":4"));

        assert_eq!(check_syntax("sin(x)"), "{\"ok\":true}");
    }

    #[test]
    fn extrema_are_classified() {
        let mut f = Function::new("x^3 - 3x").unwrap();
        let found = f.extrema(-5.0, 5.0);
        assert_eq!(found.len(), 6);
        assert!((found[0] + 1.0).abs() < 1e-6);
        assert_eq!(found[2], 1.0);
        assert!((found[3] - 1.0).abs() < 1e-6);
        assert_eq!(found[5], -1.0);
    }

    #[test]
    fn integral_of_a_parabola() {
        let f = Function::new("x^2").unwrap();
        assert!((f.integral(0.0, 3.0) - 9.0).abs() < 1e-9);
    }

    #[test]
    fn sampling_returns_pairs() {
        let f = Function::new("sin(x)").unwrap();
        let points = f.sample(-10.0, 10.0, -2.0, 2.0, 800.0, 600.0);
        assert_eq!(points.len() % 2, 0);
        assert!(points.len() > 200);
    }
}

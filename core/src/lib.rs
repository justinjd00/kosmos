pub mod calculus;
pub mod chem;
pub mod dynamics;
pub mod eval;
pub mod expr;
pub mod field;
pub mod plot;

use wasm_bindgen::prelude::*;

use dynamics::{Integrator, Kind};
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

const TRAIL_CAPACITY: usize = 60_000;

#[wasm_bindgen]
pub struct System {
    kind: Kind,
    params: Vec<f64>,
    state: Vec<f64>,
    integrator: Integrator,
    trail: Vec<f32>,
    head: usize,
    filled: usize,
    time: f64,
}

#[wasm_bindgen]
impl System {
    #[wasm_bindgen(constructor)]
    pub fn new(name: &str) -> Result<System, String> {
        let kind = Kind::from_name(name).ok_or_else(|| format!("unknown system '{name}'"))?;
        let mut system = System {
            kind,
            params: kind.defaults().to_vec(),
            state: kind.initial(),
            integrator: Integrator::default(),
            trail: vec![0.0; TRAIL_CAPACITY * 3],
            head: 0,
            filled: 0,
            time: 0.0,
        };
        system.record();
        Ok(system)
    }

    fn record(&mut self) {
        let point = self.sample_point();
        let base = self.head * 3;
        self.trail[base] = point[0] as f32;
        self.trail[base + 1] = point[1] as f32;
        self.trail[base + 2] = point[2] as f32;
        self.head = (self.head + 1) % TRAIL_CAPACITY;
        self.filled = (self.filled + 1).min(TRAIL_CAPACITY);
    }

    fn sample_point(&self) -> [f64; 3] {
        match self.kind {
            Kind::DoublePendulum => {
                let (l1, l2) = (self.params[2], self.params[3]);
                let (t1, t2) = (self.state[0], self.state[1]);
                let x1 = l1 * t1.sin();
                let y1 = -l1 * t1.cos();
                [x1 + l2 * t2.sin(), y1 - l2 * t2.cos(), 0.0]
            }
            Kind::ThreeBody => [self.state[0], self.state[1], 0.0],
            _ => [self.state[0], self.state[1], self.state[2]],
        }
    }

    pub fn advance(&mut self, seconds: f64) -> usize {
        let dt = self.kind.suggested_step();
        let steps = ((seconds / dt).round() as usize).min(4000);
        for _ in 0..steps {
            self.integrator
                .step(self.kind, &self.params, &mut self.state, dt);
            self.time += dt;
            self.record();
        }
        steps
    }

    pub fn reset(&mut self) {
        self.state = self.kind.initial();
        self.head = 0;
        self.filled = 0;
        self.time = 0.0;
        self.record();
    }

    #[wasm_bindgen(js_name = clearTrail)]
    pub fn clear_trail(&mut self) {
        self.head = 0;
        self.filled = 0;
        self.record();
    }

    #[wasm_bindgen(js_name = setState)]
    pub fn set_state(&mut self, values: Vec<f64>) {
        if values.len() == self.state.len() {
            self.state.copy_from_slice(&values);
            self.head = 0;
            self.filled = 0;
            self.record();
        }
    }

    #[wasm_bindgen(js_name = setTime)]
    pub fn set_time(&mut self, value: f64) {
        self.time = value;
    }

    #[wasm_bindgen(js_name = nudge)]
    pub fn nudge(&mut self, index: usize, amount: f64) {
        if index < self.state.len() {
            self.state[index] += amount;
        }
    }

    #[wasm_bindgen(js_name = setParam)]
    pub fn set_param(&mut self, index: usize, value: f64) {
        if index < self.params.len() {
            self.params[index] = value;
        }
    }

    #[wasm_bindgen(js_name = paramCount)]
    pub fn param_count(&self) -> usize {
        self.params.len()
    }

    #[wasm_bindgen(js_name = isSpatial)]
    pub fn is_spatial(&self) -> bool {
        self.kind.is_spatial()
    }

    pub fn time(&self) -> f64 {
        self.time
    }

    pub fn energy(&self) -> f64 {
        dynamics::energy(self.kind, &self.params, &self.state)
    }

    pub fn state(&self) -> Vec<f64> {
        self.state.clone()
    }

    pub fn positions(&self) -> Vec<f64> {
        match self.kind {
            Kind::DoublePendulum => {
                let (l1, l2) = (self.params[2], self.params[3]);
                let (t1, t2) = (self.state[0], self.state[1]);
                let x1 = l1 * t1.sin();
                let y1 = -l1 * t1.cos();
                vec![x1, y1, x1 + l2 * t2.sin(), y1 - l2 * t2.cos()]
            }
            Kind::ThreeBody => (0..3)
                .flat_map(|body| [self.state[body * 4], self.state[body * 4 + 1]])
                .collect(),
            _ => vec![self.state[0], self.state[1], self.state[2]],
        }
    }

    pub fn trail(&self, yaw: f64, pitch: f64, keep: usize) -> Vec<f32> {
        let count = self.filled.min(keep.max(2));
        let mut out = Vec::with_capacity(count * 2);

        let start = (self.head + TRAIL_CAPACITY - count) % TRAIL_CAPACITY;

        if !self.kind.is_spatial() {
            for offset in 0..count {
                let index = ((start + offset) % TRAIL_CAPACITY) * 3;
                out.push(self.trail[index]);
                out.push(self.trail[index + 1]);
            }
            return out;
        }

        let (sin_yaw, cos_yaw) = yaw.sin_cos();
        let (sin_pitch, cos_pitch) = pitch.sin_cos();

        for offset in 0..count {
            let index = ((start + offset) % TRAIL_CAPACITY) * 3;
            let x = self.trail[index] as f64;
            let y = self.trail[index + 1] as f64;
            let z = self.trail[index + 2] as f64;

            let rotated_x = x * cos_yaw - y * sin_yaw;
            let rotated_y = x * sin_yaw + y * cos_yaw;

            out.push(rotated_x as f32);
            out.push((z * cos_pitch - rotated_y * sin_pitch) as f32);
        }

        out
    }

    #[wasm_bindgen(js_name = trailLength)]
    pub fn trail_length(&self) -> usize {
        self.filled
    }

    pub fn bounds(&self, yaw: f64, pitch: f64) -> Vec<f32> {
        let points = self.trail(yaw, pitch, TRAIL_CAPACITY);
        if points.is_empty() {
            return vec![-1.0, 1.0, -1.0, 1.0];
        }
        let mut min_x = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        for pair in points.chunks_exact(2) {
            if !pair[0].is_finite() || !pair[1].is_finite() {
                continue;
            }
            min_x = min_x.min(pair[0]);
            max_x = max_x.max(pair[0]);
            min_y = min_y.min(pair[1]);
            max_y = max_y.max(pair[1]);
        }
        if !min_x.is_finite() {
            return vec![-1.0, 1.0, -1.0, 1.0];
        }
        if !self.kind.is_spatial() {
            min_x = min_x.min(0.0);
            max_x = max_x.max(0.0);
            min_y = min_y.min(0.0);
            max_y = max_y.max(0.0);
        }
        vec![min_x, max_x, min_y, max_y]
    }
}

#[wasm_bindgen(js_name = systemNames)]
pub fn system_names() -> Vec<String> {
    [
        "lorenz",
        "rossler",
        "aizawa",
        "thomas",
        "halvorsen",
        "chen",
        "double-pendulum",
        "three-body",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

const SHEET_MIN: usize = 32;
const SHEET_MAX: usize = 1024;

#[wasm_bindgen]
pub struct Sheet {
    field: field::Field,
    preset: String,
}

#[wasm_bindgen]
impl Sheet {
    #[wasm_bindgen(constructor)]
    pub fn new(preset: &str, width: usize, height: usize) -> Result<Sheet, String> {
        if !(SHEET_MIN..=SHEET_MAX).contains(&width) || !(SHEET_MIN..=SHEET_MAX).contains(&height) {
            return Err(format!("a {width}x{height} grid is out of range"));
        }
        let field = field::Field::build(preset, width, height)
            .ok_or_else(|| format!("unknown field '{preset}'"))?;
        Ok(Sheet {
            field,
            preset: preset.to_string(),
        })
    }

    pub fn preset(&self) -> String {
        self.preset.clone()
    }

    pub fn kind(&self) -> String {
        match self.field.kind() {
            field::Kind::Wave => "wave",
            field::Kind::Heat => "heat",
            field::Kind::Charge => "charge",
        }
        .to_string()
    }

    pub fn evolves(&self) -> bool {
        self.field.kind().evolves()
    }

    pub fn load(&mut self, preset: &str) -> Result<(), String> {
        let entry = field::preset(preset).ok_or_else(|| format!("unknown field '{preset}'"))?;
        if entry.kind != self.field.kind() {
            return Err("that field needs a different grid".to_string());
        }
        self.field.apply(preset);
        self.preset = preset.to_string();
        Ok(())
    }

    pub fn reset(&mut self) {
        let preset = self.preset.clone();
        self.field.apply(&preset);
    }

    pub fn advance(&mut self, seconds: f64) -> usize {
        self.field.advance(seconds.clamp(0.0, 0.25))
    }

    pub fn paint(&mut self, gain: f64, contours: bool) -> *const u8 {
        self.field.paint(gain, contours).as_ptr()
    }

    pub fn bytes(&self) -> usize {
        self.field.width() * self.field.height() * 4
    }

    pub fn width(&self) -> usize {
        self.field.width()
    }

    pub fn height(&self) -> usize {
        self.field.height()
    }

    pub fn time(&self) -> f64 {
        self.field.time()
    }

    pub fn energy(&self) -> f64 {
        self.field.energy()
    }

    pub fn probe(&self, x: f64, y: f64) -> f64 {
        self.field.probe(x.clamp(0.0, 1.0), y.clamp(0.0, 1.0))
    }

    pub fn poke(&mut self, x: f64, y: f64, radius: f64, amount: f64) {
        self.field
            .poke(x.clamp(0.0, 1.0), y.clamp(0.0, 1.0), radius, amount);
    }

    pub fn wall(&mut self, x: f64, y: f64, radius: f64, solid: bool) {
        self.field
            .wall(x.clamp(0.0, 1.0), y.clamp(0.0, 1.0), radius, solid);
    }

    #[wasm_bindgen(js_name = clearWalls)]
    pub fn clear_walls(&mut self) {
        self.field.clear_geometry();
    }

    #[wasm_bindgen(js_name = setSpeed)]
    pub fn set_speed(&mut self, value: f64) {
        self.field.set_speed(value);
    }

    #[wasm_bindgen(js_name = setDamping)]
    pub fn set_damping(&mut self, value: f64) {
        self.field.set_damping(value);
    }

    #[wasm_bindgen(js_name = setDiffusivity)]
    pub fn set_diffusivity(&mut self, value: f64) {
        self.field.set_diffusivity(value);
    }

    #[wasm_bindgen(js_name = setEdge)]
    pub fn set_edge(&mut self, absorbing: bool) {
        self.field.set_edge(if absorbing {
            field::Edge::Absorb
        } else {
            field::Edge::Reflect
        });
    }

    pub fn speed(&self) -> f64 {
        self.field.speed()
    }

    pub fn damping(&self) -> f64 {
        self.field.damping()
    }

    pub fn diffusivity(&self) -> f64 {
        self.field.diffusivity()
    }

    pub fn absorbing(&self) -> bool {
        self.field.edge() == field::Edge::Absorb
    }

    #[wasm_bindgen(js_name = sourceCount)]
    pub fn source_count(&self) -> usize {
        self.field.sources().len()
    }

    pub fn sources(&self) -> Vec<f64> {
        self.field
            .sources()
            .iter()
            .flat_map(|s| [s.x, s.y, s.strength, s.span])
            .collect()
    }

    #[wasm_bindgen(js_name = moveSource)]
    pub fn move_source(&mut self, index: usize, x: f64, y: f64) {
        self.field
            .move_source(index, x.clamp(0.0, 1.0), y.clamp(0.0, 1.0));
    }
}

#[wasm_bindgen(js_name = fieldPresets)]
pub fn field_presets() -> Vec<String> {
    field::PRESETS
        .iter()
        .map(|entry| {
            let kind = match entry.kind {
                field::Kind::Wave => "wave",
                field::Kind::Heat => "heat",
                field::Kind::Charge => "charge",
            };
            format!("{}:{}", entry.id, kind)
        })
        .collect()
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

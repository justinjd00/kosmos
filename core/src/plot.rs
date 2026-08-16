use crate::eval::Program;

#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
    pub width: f64,
    pub height: f64,
}

impl Viewport {
    pub fn to_screen_x(&self, x: f64) -> f64 {
        (x - self.x_min) / (self.x_max - self.x_min) * self.width
    }

    pub fn to_screen_y(&self, y: f64) -> f64 {
        (self.y_max - y) / (self.y_max - self.y_min) * self.height
    }

    pub fn span_y(&self) -> f64 {
        self.y_max - self.y_min
    }
}

const MAX_REFINEMENT: u32 = 8;
const ANGLE_TOLERANCE: f64 = 0.28;

const SUBPIXEL: f64 = 0.6;

pub struct Curve {
    pub points: Vec<f32>,
}

struct Sampler<'a> {
    program: &'a Program,
    vars: Vec<f64>,
    slot: usize,
    view: Viewport,
    out: Vec<f32>,
}

impl Sampler<'_> {
    fn value_at(&mut self, x: f64) -> f64 {
        self.vars[self.slot] = x;
        self.program.eval(&self.vars)
    }

    fn push(&mut self, x: f64, y: f64) {
        self.out.push(x as f32);
        self.out.push(y as f32);
    }

    fn push_break(&mut self) {
        if self.out.len() >= 2 {
            let last = self.out[self.out.len() - 1];
            if last.is_nan() {
                return;
            }
        }
        self.out.push(f32::NAN);
        self.out.push(f32::NAN);
    }

    fn is_pole_between(&mut self, x0: f64, y0: f64, x2: f64, y2: f64) -> bool {
        let mut lo = x0;
        let mut flo = y0;
        let mut hi = x2;
        let mut fhi = y2;

        for _ in 0..14 {
            let mid = 0.5 * (lo + hi);
            let fmid = self.value_at(mid);
            if !fmid.is_finite() {
                return true;
            }
            if flo.signum() != fmid.signum() {
                hi = mid;
                fhi = fmid;
            } else {
                lo = mid;
                flo = fmid;
            }
        }

        flo.abs().min(fhi.abs()) > y0.abs().max(y2.abs())
    }

    fn is_visible(&self, y: f64) -> bool {
        let margin = self.view.span_y() * 4.0;
        y.is_finite() && y > self.view.y_min - margin && y < self.view.y_max + margin
    }

    fn refine(&mut self, x0: f64, y0: f64, x2: f64, y2: f64, depth: u32) {
        if depth >= MAX_REFINEMENT {
            return;
        }

        let x1 = 0.5 * (x0 + x2);
        let y1 = self.value_at(x1);

        if !y1.is_finite() {
            self.push_break();
            return;
        }

        let sx0 = self.view.to_screen_x(x0);
        let sx1 = self.view.to_screen_x(x1);
        let sx2 = self.view.to_screen_x(x2);
        let sy0 = self.view.to_screen_y(y0);
        let sy1 = self.view.to_screen_y(y1);
        let sy2 = self.view.to_screen_y(y2);

        let ax = sx1 - sx0;
        let ay = sy1 - sy0;
        let bx = sx2 - sx1;
        let by = sy2 - sy1;

        let cross = (ax * by - ay * bx).abs();
        let dot = ax * bx + ay * by;
        let bend = cross.atan2(dot).abs();

        let straight = bend < ANGLE_TOLERANCE
            || (ax.abs() + ay.abs() < SUBPIXEL && bx.abs() + by.abs() < SUBPIXEL);

        if straight {
            self.push(x1, y1);
            return;
        }

        self.refine(x0, y0, x1, y1, depth + 1);
        self.push(x1, y1);
        self.refine(x1, y1, x2, y2, depth + 1);
    }
}

pub fn sample(program: &Program, view: Viewport, vars: &[f64], slot: usize) -> Curve {
    let base_points = (view.width as usize / 6).clamp(48, 512);
    let mut sampler = Sampler {
        program,
        vars: vars.to_vec(),
        slot,
        view,
        out: Vec::with_capacity(base_points * 6),
    };

    let step = (view.x_max - view.x_min) / base_points as f64;
    let mut previous_x = view.x_min;
    let mut previous_y = sampler.value_at(previous_x);

    if sampler.is_visible(previous_y) {
        sampler.push(previous_x, previous_y);
    } else {
        sampler.push_break();
    }

    for i in 1..=base_points {
        let x = view.x_min + step * i as f64;
        let y = sampler.value_at(x);

        let both_finite = previous_y.is_finite() && y.is_finite();
        let jumped = both_finite
            && previous_y != 0.0
            && y != 0.0
            && previous_y.signum() != y.signum()
            && sampler.is_pole_between(previous_x, previous_y, x, y);

        if !y.is_finite() || jumped {
            sampler.push_break();
        } else if both_finite && sampler.is_visible(y) && sampler.is_visible(previous_y) {
            sampler.refine(previous_x, previous_y, x, y, 0);
            sampler.push(x, y);
        } else if sampler.is_visible(y) {
            sampler.push(x, y);
        } else {
            sampler.push(x, y);
            sampler.push_break();
        }

        previous_x = x;
        previous_y = y;
    }

    Curve {
        points: sampler.out,
    }
}

pub fn nice_step(span: f64, target_count: f64) -> f64 {
    if span <= 0.0 || !span.is_finite() {
        return 1.0;
    }
    let rough = span / target_count.max(1.0);
    let magnitude = 10f64.powf(rough.log10().floor());
    let normalised = rough / magnitude;
    let step = if normalised < 1.5 {
        1.0
    } else if normalised < 3.5 {
        2.0
    } else if normalised < 7.5 {
        5.0
    } else {
        10.0
    };
    step * magnitude
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::parse;

    const VARS: &[&str] = &["x", "y", "t"];

    fn view() -> Viewport {
        Viewport {
            x_min: -10.0,
            x_max: 10.0,
            y_min: -10.0,
            y_max: 10.0,
            width: 800.0,
            height: 600.0,
        }
    }

    fn curve(source: &str) -> Curve {
        let expr = parse(source, VARS).unwrap();
        let program = Program::compile(&expr);
        sample(&program, view(), &[0.0, 0.0, 0.0], 0)
    }

    #[test]
    fn a_line_needs_few_points() {
        let c = curve("x");
        let count = c.points.len() / 2;
        assert!(count < 1000, "{count} points for a straight line");
    }

    #[test]
    fn curvature_gets_more_points_than_flatness() {
        let wiggly = curve("sin(8x)").points.len();
        let flat = curve("0.001x").points.len();
        assert!(wiggly > flat * 2, "{wiggly} vs {flat}");
    }

    #[test]
    fn poles_produce_a_break() {
        let c = curve("1/x");
        let breaks = c
            .points
            .chunks(2)
            .filter(|p| p[0].is_nan() && p[1].is_nan())
            .count();
        assert!(breaks >= 1, "expected a break at the pole");
    }

    #[test]
    fn tangent_breaks_at_every_pole() {
        let c = curve("tan(x)");
        let breaks = c.points.chunks(2).filter(|p| p[0].is_nan()).count();
        assert!(breaks >= 6, "only {breaks} breaks across six poles");
    }

    #[test]
    fn undefined_regions_are_skipped() {
        let c = curve("sqrt(x)");
        for pair in c.points.chunks(2) {
            if !pair[0].is_nan() {
                assert!(pair[0] >= -0.2, "sampled sqrt at {}", pair[0]);
            }
        }
    }

    #[test]
    fn axis_steps_are_human_friendly() {
        assert_eq!(nice_step(20.0, 10.0), 2.0);
        assert_eq!(nice_step(1.0, 10.0), 0.1);
        assert_eq!(nice_step(0.05, 5.0), 0.01);
        assert_eq!(nice_step(1000.0, 10.0), 100.0);
    }
}

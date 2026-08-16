pub const MAX_STATE: usize = 12;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Lorenz,
    Rossler,
    Aizawa,
    Thomas,
    Halvorsen,
    Chen,
    DoublePendulum,
    ThreeBody,
}

impl Kind {
    pub fn from_name(name: &str) -> Option<Kind> {
        Some(match name {
            "lorenz" => Kind::Lorenz,
            "rossler" => Kind::Rossler,
            "aizawa" => Kind::Aizawa,
            "thomas" => Kind::Thomas,
            "halvorsen" => Kind::Halvorsen,
            "chen" => Kind::Chen,
            "double-pendulum" => Kind::DoublePendulum,
            "three-body" => Kind::ThreeBody,
            _ => return None,
        })
    }

    pub fn dimension(self) -> usize {
        match self {
            Kind::DoublePendulum => 4,
            Kind::ThreeBody => 12,
            _ => 3,
        }
    }

    pub fn is_spatial(self) -> bool {
        matches!(
            self,
            Kind::Lorenz
                | Kind::Rossler
                | Kind::Aizawa
                | Kind::Thomas
                | Kind::Halvorsen
                | Kind::Chen
        )
    }

    pub fn defaults(self) -> &'static [f64] {
        match self {
            Kind::Lorenz => &[10.0, 28.0, 2.666_666_666_666_667],
            Kind::Rossler => &[0.2, 0.2, 5.7],
            Kind::Aizawa => &[0.95, 0.7, 0.6, 3.5],
            Kind::Thomas => &[0.19],
            Kind::Halvorsen => &[1.4],
            Kind::Chen => &[5.0, -10.0, -0.38],
            Kind::DoublePendulum => &[1.0, 1.0, 1.0, 1.0, 9.81],
            Kind::ThreeBody => &[1.0, 1.0, 1.0],
        }
    }

    pub fn initial(self) -> Vec<f64> {
        match self {
            Kind::Lorenz => vec![0.9, 0.0, 1.0],
            Kind::Rossler => vec![1.0, 1.0, 1.0],
            Kind::Aizawa => vec![0.1, 0.0, 0.0],
            Kind::Thomas => vec![1.1, 1.1, -0.01],
            Kind::Halvorsen => vec![-1.48, -1.51, 2.04],
            Kind::Chen => vec![5.0, 10.0, 10.0],
            Kind::DoublePendulum => vec![2.2, 2.4, 0.0, 0.0],
            Kind::ThreeBody => vec![
                -0.97000436,
                0.24308753,
                0.4662036850,
                0.4323657300,
                0.97000436,
                -0.24308753,
                0.4662036850,
                0.4323657300,
                0.0,
                0.0,
                -0.93240737,
                -0.86473146,
            ],
        }
    }

    pub fn suggested_step(self) -> f64 {
        match self {
            Kind::Thomas => 0.02,
            Kind::DoublePendulum => 0.004,
            Kind::ThreeBody => 0.002,
            _ => 0.006,
        }
    }
}

#[allow(clippy::needless_range_loop)]
pub fn derivative(kind: Kind, params: &[f64], state: &[f64], out: &mut [f64]) {
    match kind {
        Kind::Lorenz => {
            let (sigma, rho, beta) = (params[0], params[1], params[2]);
            let (x, y, z) = (state[0], state[1], state[2]);
            out[0] = sigma * (y - x);
            out[1] = x * (rho - z) - y;
            out[2] = x * y - beta * z;
        }

        Kind::Rossler => {
            let (a, b, c) = (params[0], params[1], params[2]);
            let (x, y, z) = (state[0], state[1], state[2]);
            out[0] = -y - z;
            out[1] = x + a * y;
            out[2] = b + z * (x - c);
        }

        Kind::Aizawa => {
            let (a, b, c, d) = (params[0], params[1], params[2], params[3]);
            let (x, y, z) = (state[0], state[1], state[2]);
            out[0] = (z - b) * x - d * y;
            out[1] = d * x + (z - b) * y;
            out[2] = c + a * z - z * z * z / 3.0 - (x * x + y * y) * (1.0 + 0.25 * z)
                + 0.1 * z * x * x * x;
        }

        Kind::Thomas => {
            let b = params[0];
            let (x, y, z) = (state[0], state[1], state[2]);
            out[0] = y.sin() - b * x;
            out[1] = z.sin() - b * y;
            out[2] = x.sin() - b * z;
        }

        Kind::Halvorsen => {
            let a = params[0];
            let (x, y, z) = (state[0], state[1], state[2]);
            out[0] = -a * x - 4.0 * y - 4.0 * z - y * y;
            out[1] = -a * y - 4.0 * z - 4.0 * x - z * z;
            out[2] = -a * z - 4.0 * x - 4.0 * y - x * x;
        }

        Kind::Chen => {
            let (a, b, c) = (params[0], params[1], params[2]);
            let (x, y, z) = (state[0], state[1], state[2]);
            out[0] = a * x - y * z;
            out[1] = b * y + x * z;
            out[2] = c * z + x * y / 3.0;
        }

        Kind::DoublePendulum => {
            let (m1, m2, l1, l2, g) = (params[0], params[1], params[2], params[3], params[4]);
            let (t1, t2, w1, w2) = (state[0], state[1], state[2], state[3]);
            let delta = t1 - t2;
            let denominator = 2.0 * m1 + m2 - m2 * (2.0 * delta).cos();

            out[0] = w1;
            out[1] = w2;
            out[2] = (-g * (2.0 * m1 + m2) * t1.sin()
                - m2 * g * (t1 - 2.0 * t2).sin()
                - 2.0 * delta.sin() * m2 * (w2 * w2 * l2 + w1 * w1 * l1 * delta.cos()))
                / (l1 * denominator);
            out[3] = (2.0
                * delta.sin()
                * (w1 * w1 * l1 * (m1 + m2)
                    + g * (m1 + m2) * t1.cos()
                    + w2 * w2 * l2 * m2 * delta.cos()))
                / (l2 * denominator);
        }

        Kind::ThreeBody => {
            const SOFTENING: f64 = 1e-4;
            for body in 0..3 {
                let base = body * 4;
                out[base] = state[base + 2];
                out[base + 1] = state[base + 3];
                let mut ax = 0.0;
                let mut ay = 0.0;
                for other in 0..3 {
                    if other == body {
                        continue;
                    }
                    let target = other * 4;
                    let dx = state[target] - state[base];
                    let dy = state[target + 1] - state[base + 1];
                    let squared = dx * dx + dy * dy + SOFTENING;
                    let inverse = params[other] / (squared * squared.sqrt());
                    ax += dx * inverse;
                    ay += dy * inverse;
                }
                out[base + 2] = ax;
                out[base + 3] = ay;
            }
        }
    }
}

pub struct Integrator {
    k1: [f64; MAX_STATE],
    k2: [f64; MAX_STATE],
    k3: [f64; MAX_STATE],
    k4: [f64; MAX_STATE],
    scratch: [f64; MAX_STATE],
}

impl Default for Integrator {
    fn default() -> Self {
        Integrator {
            k1: [0.0; MAX_STATE],
            k2: [0.0; MAX_STATE],
            k3: [0.0; MAX_STATE],
            k4: [0.0; MAX_STATE],
            scratch: [0.0; MAX_STATE],
        }
    }
}

impl Integrator {
    #[allow(clippy::needless_range_loop)]
    pub fn step(&mut self, kind: Kind, params: &[f64], state: &mut [f64], dt: f64) {
        let n = state.len();

        derivative(kind, params, state, &mut self.k1);

        for i in 0..n {
            self.scratch[i] = state[i] + 0.5 * dt * self.k1[i];
        }
        derivative(kind, params, &self.scratch[..n], &mut self.k2);

        for i in 0..n {
            self.scratch[i] = state[i] + 0.5 * dt * self.k2[i];
        }
        derivative(kind, params, &self.scratch[..n], &mut self.k3);

        for i in 0..n {
            self.scratch[i] = state[i] + dt * self.k3[i];
        }
        derivative(kind, params, &self.scratch[..n], &mut self.k4);

        for i in 0..n {
            state[i] += dt / 6.0 * (self.k1[i] + 2.0 * self.k2[i] + 2.0 * self.k3[i] + self.k4[i]);
        }
    }
}

#[allow(clippy::needless_range_loop)]
pub fn energy(kind: Kind, params: &[f64], state: &[f64]) -> f64 {
    match kind {
        Kind::DoublePendulum => {
            let (m1, m2, l1, l2, g) = (params[0], params[1], params[2], params[3], params[4]);
            let (t1, t2, w1, w2) = (state[0], state[1], state[2], state[3]);
            let kinetic = 0.5 * m1 * l1 * l1 * w1 * w1
                + 0.5
                    * m2
                    * (l1 * l1 * w1 * w1
                        + l2 * l2 * w2 * w2
                        + 2.0 * l1 * l2 * w1 * w2 * (t1 - t2).cos());
            let potential = -(m1 + m2) * g * l1 * t1.cos() - m2 * g * l2 * t2.cos();
            kinetic + potential
        }

        Kind::ThreeBody => {
            let mut total = 0.0;
            for body in 0..3 {
                let base = body * 4;
                let vx = state[base + 2];
                let vy = state[base + 3];
                total += 0.5 * params[body] * (vx * vx + vy * vy);
            }
            for a in 0..3 {
                for b in (a + 1)..3 {
                    let dx = state[b * 4] - state[a * 4];
                    let dy = state[b * 4 + 1] - state[a * 4 + 1];
                    let distance = (dx * dx + dy * dy).sqrt().max(1e-9);
                    total -= params[a] * params[b] / distance;
                }
            }
            total
        }

        _ => f64::NAN,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(kind: Kind, steps: usize) -> Vec<f64> {
        let params = kind.defaults().to_vec();
        let mut state = kind.initial();
        let mut integrator = Integrator::default();
        let dt = kind.suggested_step();
        for _ in 0..steps {
            integrator.step(kind, &params, &mut state, dt);
        }
        state
    }

    #[test]
    fn every_system_stays_finite() {
        for kind in [
            Kind::Lorenz,
            Kind::Rossler,
            Kind::Aizawa,
            Kind::Thomas,
            Kind::Halvorsen,
            Kind::Chen,
            Kind::DoublePendulum,
            Kind::ThreeBody,
        ] {
            let state = run(kind, 4000);
            assert_eq!(state.len(), kind.dimension());
            for value in &state {
                assert!(value.is_finite(), "{kind:?} diverged to {value}");
            }
        }
    }

    #[test]
    fn lorenz_settles_onto_its_attractor() {
        let state = run(Kind::Lorenz, 20_000);
        assert!(state[0].abs() < 30.0);
        assert!(state[2] > 0.0 && state[2] < 60.0);
    }

    #[test]
    fn the_pendulum_conserves_energy() {
        let kind = Kind::DoublePendulum;
        let params = kind.defaults().to_vec();
        let mut state = kind.initial();
        let start = energy(kind, &params, &state);

        let mut integrator = Integrator::default();
        for _ in 0..20_000 {
            integrator.step(kind, &params, &mut state, kind.suggested_step());
        }

        let drift = (energy(kind, &params, &state) - start).abs() / start.abs();
        assert!(drift < 1e-4, "energy drifted by {:.3e} relative", drift);
    }

    #[test]
    fn the_figure_eight_orbit_is_periodic() {
        let kind = Kind::ThreeBody;
        let params = kind.defaults().to_vec();
        let start = kind.initial();
        let mut state = start.clone();
        let mut integrator = Integrator::default();

        let period = 6.324_449_066_4_f64;
        let dt = 1e-4_f64;
        let steps = (period / dt).round() as usize;
        for _ in 0..steps {
            integrator.step(kind, &params, &mut state, dt);
        }

        for (a, b) in state.iter().zip(start.iter()) {
            assert!((a - b).abs() < 5e-3, "orbit closed badly: {a} vs {b}");
        }
    }

    #[test]
    fn tiny_differences_grow_exponentially() {
        let kind = Kind::Lorenz;
        let params = kind.defaults().to_vec();
        let mut a = kind.initial();
        let mut b = kind.initial();
        b[0] += 1e-9;

        let mut integrator = Integrator::default();
        for _ in 0..8000 {
            integrator.step(kind, &params, &mut a, 0.005);
            integrator.step(kind, &params, &mut b, 0.005);
        }

        let separation =
            ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
        assert!(separation > 1.0, "butterfly effect too weak: {separation}");
    }
}

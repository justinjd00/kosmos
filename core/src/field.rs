pub const MAX_SOURCES: usize = 12;
pub const STEPS_PER_SECOND: f64 = 480.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Wave,
    Heat,
    Charge,
}

impl Kind {
    pub fn from_name(name: &str) -> Option<Kind> {
        Some(match name {
            "wave" => Kind::Wave,
            "heat" => Kind::Heat,
            "charge" => Kind::Charge,
            _ => return None,
        })
    }

    pub fn evolves(self) -> bool {
        !matches!(self, Kind::Charge)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Source {
    pub x: f64,
    pub y: f64,
    pub strength: f64,
    pub frequency: f64,
    pub span: f64,
}

impl Source {
    pub fn point(x: f64, y: f64, strength: f64, frequency: f64) -> Source {
        Source {
            x,
            y,
            strength,
            frequency,
            span: 0.0,
        }
    }

    pub fn line(x: f64, y: f64, span: f64, strength: f64, frequency: f64) -> Source {
        Source {
            x,
            y,
            strength,
            frequency,
            span,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    Reflect,
    Absorb,
}

pub struct Field {
    kind: Kind,
    width: usize,
    height: usize,
    now: Vec<f32>,
    before: Vec<f32>,
    next: Vec<f32>,
    solid: Vec<bool>,
    medium: Vec<f32>,
    sources: Vec<Source>,
    edge: Edge,
    speed: f64,
    damping: f64,
    diffusivity: f64,
    time: f64,
    dirty: bool,
    peak: f32,
    pixels: Vec<u8>,
    palette: [[u8; 3]; 256],
}

fn clamp01(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

fn mix(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    let t = clamp01(t);
    [
        (a[0] as f32 + (b[0] as f32 - a[0] as f32) * t) as u8,
        (a[1] as f32 + (b[1] as f32 - a[1] as f32) * t) as u8,
        (a[2] as f32 + (b[2] as f32 - a[2] as f32) * t) as u8,
    ]
}

fn ramp(stops: &[[u8; 3]]) -> [[u8; 3]; 256] {
    let mut out = [[0u8; 3]; 256];
    let spans = stops.len() - 1;
    for (index, slot) in out.iter_mut().enumerate() {
        let t = index as f32 / 255.0 * spans as f32;
        let low = (t.floor() as usize).min(spans - 1);
        *slot = mix(stops[low], stops[low + 1], t - low as f32);
    }
    out
}

fn palette_for(kind: Kind) -> [[u8; 3]; 256] {
    match kind {
        Kind::Wave => ramp(&[
            [8, 24, 68],
            [22, 88, 158],
            [78, 178, 210],
            [10, 12, 17],
            [232, 148, 88],
            [206, 74, 60],
            [96, 16, 34],
        ]),
        Kind::Heat => ramp(&[
            [6, 6, 12],
            [58, 12, 72],
            [154, 28, 78],
            [226, 96, 40],
            [248, 186, 62],
            [255, 246, 214],
        ]),
        Kind::Charge => ramp(&[
            [10, 32, 92],
            [34, 106, 186],
            [110, 190, 222],
            [12, 14, 20],
            [236, 176, 96],
            [214, 84, 62],
            [104, 18, 36],
        ]),
    }
}

impl Field {
    pub fn new(kind: Kind, width: usize, height: usize) -> Field {
        let cells = width * height;
        let mut field = Field {
            kind,
            width,
            height,
            now: vec![0.0; cells],
            before: vec![0.0; cells],
            next: vec![0.0; cells],
            solid: vec![false; cells],
            medium: vec![1.0; cells],
            sources: Vec::new(),
            edge: Edge::Absorb,
            speed: 0.42,
            damping: 0.0,
            diffusivity: 0.22,
            time: 0.0,
            dirty: true,
            peak: 1.0,
            pixels: vec![0; cells * 4],
            palette: palette_for(kind),
        };
        field.clear();
        field
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn time(&self) -> f64 {
        self.time
    }

    pub fn kind(&self) -> Kind {
        self.kind
    }

    pub fn clear(&mut self) {
        self.now.iter_mut().for_each(|v| *v = 0.0);
        self.before.iter_mut().for_each(|v| *v = 0.0);
        self.next.iter_mut().for_each(|v| *v = 0.0);
        self.time = 0.0;
        self.dirty = true;
        self.peak = 0.0;
        if self.kind == Kind::Heat {
            self.stamp_sources();
        }
    }

    pub fn clear_geometry(&mut self) {
        self.solid.iter_mut().for_each(|v| *v = false);
        self.medium.iter_mut().for_each(|v| *v = 1.0);
        self.sources.clear();
        self.dirty = true;
    }

    pub fn set_edge(&mut self, edge: Edge) {
        self.edge = edge;
    }

    pub fn edge(&self) -> Edge {
        self.edge
    }

    pub fn speed(&self) -> f64 {
        self.speed
    }

    pub fn damping(&self) -> f64 {
        self.damping
    }

    pub fn diffusivity(&self) -> f64 {
        self.diffusivity
    }

    pub fn set_speed(&mut self, value: f64) {
        self.speed = value.clamp(0.02, 0.5);
    }

    pub fn set_damping(&mut self, value: f64) {
        self.damping = value.clamp(0.0, 0.02);
    }

    pub fn set_diffusivity(&mut self, value: f64) {
        self.diffusivity = value.clamp(0.01, 0.24);
    }

    pub fn add_source(&mut self, source: Source) {
        if self.sources.len() < MAX_SOURCES {
            self.sources.push(source);
            self.dirty = true;
        }
    }

    pub fn sources(&self) -> &[Source] {
        &self.sources
    }

    pub fn move_source(&mut self, index: usize, x: f64, y: f64) {
        if let Some(source) = self.sources.get_mut(index) {
            source.x = x;
            source.y = y;
            self.dirty = true;
        }
    }

    pub fn wall(&mut self, x: f64, y: f64, radius: f64, solid: bool) {
        self.paint_disc(x, y, radius, |field, index| {
            field.solid[index] = solid;
            if solid {
                field.now[index] = 0.0;
                field.before[index] = 0.0;
                field.next[index] = 0.0;
            }
        });
    }

    pub fn lens(&mut self, x: f64, y: f64, radius: f64, index_of_refraction: f64) {
        let n = index_of_refraction.clamp(0.25, 4.0) as f32;
        self.paint_disc(x, y, radius, move |field, cell| {
            field.medium[cell] = n;
        });
    }

    pub fn poke(&mut self, x: f64, y: f64, radius: f64, amount: f64) {
        let cx = x * self.width as f64;
        let cy = y * self.height as f64;
        let r = (radius * self.width as f64).max(1.0);
        let amount = amount as f32;

        let x0 = ((cx - r).floor().max(0.0)) as usize;
        let x1 = ((cx + r).ceil().min(self.width as f64 - 1.0)) as usize;
        let y0 = ((cy - r).floor().max(0.0)) as usize;
        let y1 = ((cy + r).ceil().min(self.height as f64 - 1.0)) as usize;

        for row in y0..=y1 {
            for column in x0..=x1 {
                let dx = column as f64 - cx;
                let dy = row as f64 - cy;
                let distance = (dx * dx + dy * dy).sqrt() / r;
                if distance > 1.0 {
                    continue;
                }
                let falloff = (0.5 + 0.5 * (std::f64::consts::PI * distance).cos()) as f32;
                let cell = row * self.width + column;
                if self.solid[cell] {
                    continue;
                }
                match self.kind {
                    Kind::Wave => {
                        self.now[cell] += amount * falloff;
                        self.before[cell] += amount * falloff;
                    }
                    Kind::Heat => {
                        self.now[cell] = (self.now[cell] + amount * falloff).clamp(-1.5, 1.5)
                    }
                    Kind::Charge => {}
                }
            }
        }
        self.dirty = true;
    }

    fn paint_disc<F: FnMut(&mut Field, usize)>(
        &mut self,
        x: f64,
        y: f64,
        radius: f64,
        mut apply: F,
    ) {
        let cx = x * self.width as f64;
        let cy = y * self.height as f64;
        let r = (radius * self.width as f64).max(1.0);

        let x0 = ((cx - r).floor().max(0.0)) as usize;
        let x1 = ((cx + r).ceil().min(self.width as f64 - 1.0)) as usize;
        let y0 = ((cy - r).floor().max(0.0)) as usize;
        let y1 = ((cy + r).ceil().min(self.height as f64 - 1.0)) as usize;

        for row in y0..=y1 {
            for column in x0..=x1 {
                let dx = column as f64 - cx;
                let dy = row as f64 - cy;
                if dx * dx + dy * dy > r * r {
                    continue;
                }
                let cell = row * self.width + column;
                apply(self, cell);
            }
        }
        self.dirty = true;
    }

    pub fn barrier(&mut self, x: f64, gaps: &[(f64, f64)], thickness: f64) {
        let column_centre = x * self.width as f64;
        let half = (thickness * self.width as f64 * 0.5).max(1.0);
        let x0 = ((column_centre - half).floor().max(0.0)) as usize;
        let x1 = ((column_centre + half).ceil().min(self.width as f64 - 1.0)) as usize;

        for row in 0..self.height {
            let position = row as f64 / self.height as f64;
            let open = gaps
                .iter()
                .any(|(centre, width)| (position - centre).abs() <= width * 0.5);
            if open {
                continue;
            }
            for column in x0..=x1 {
                let cell = row * self.width + column;
                self.solid[cell] = true;
                self.now[cell] = 0.0;
                self.before[cell] = 0.0;
                self.next[cell] = 0.0;
            }
        }
        self.dirty = true;
    }

    fn each_source_cell<F: FnMut(&mut Field, usize)>(
        &mut self,
        source: Source,
        radius: f64,
        mut apply: F,
    ) {
        if source.span > 0.0 {
            let column = ((source.x * self.width as f64) as usize).min(self.width - 1);
            let low = ((source.y - source.span * 0.5) * self.height as f64).max(0.0) as usize;
            let high = (((source.y + source.span * 0.5) * self.height as f64)
                .min(self.height as f64 - 1.0)) as usize;
            for row in low..=high {
                let cell = row * self.width + column;
                if !self.solid[cell] {
                    apply(self, cell);
                }
            }
            return;
        }

        let cx = source.x * self.width as f64;
        let cy = source.y * self.height as f64;
        let r = (radius * self.width as f64).max(1.0);
        let x0 = (cx - r).floor().max(0.0) as usize;
        let x1 = ((cx + r).ceil().min(self.width as f64 - 1.0)) as usize;
        let y0 = (cy - r).floor().max(0.0) as usize;
        let y1 = ((cy + r).ceil().min(self.height as f64 - 1.0)) as usize;

        for row in y0..=y1 {
            for column in x0..=x1 {
                let dx = column as f64 - cx;
                let dy = row as f64 - cy;
                if dx * dx + dy * dy > r * r {
                    continue;
                }
                let cell = row * self.width + column;
                if !self.solid[cell] {
                    apply(self, cell);
                }
            }
        }
    }

    fn stamp_sources(&mut self) {
        for index in 0..self.sources.len() {
            let source = self.sources[index];
            let level = source.strength as f32;
            self.each_source_cell(source, 0.014, move |field, cell| {
                field.now[cell] = level;
            });
        }
    }

    pub fn advance(&mut self, seconds: f64) -> usize {
        if !self.kind.evolves() {
            self.time += seconds;
            return 0;
        }

        let steps = ((seconds * STEPS_PER_SECOND).round() as usize).clamp(1, 2400);
        let dt = 1.0 / STEPS_PER_SECOND;
        for _ in 0..steps {
            match self.kind {
                Kind::Wave => self.step_wave(),
                Kind::Heat => self.step_heat(),
                Kind::Charge => {}
            }
            self.time += dt;
        }
        self.dirty = true;
        steps
    }

    fn drive(&mut self) {
        for index in 0..self.sources.len() {
            let source = self.sources[index];
            if source.frequency <= 0.0 {
                continue;
            }
            let value = ((self.time * source.frequency * std::f64::consts::TAU).sin()
                * source.strength) as f32;
            self.each_source_cell(source, 0.009, move |field, cell| {
                field.now[cell] += value * 0.22;
            });
        }
    }

    fn step_wave(&mut self) {
        self.drive();

        let width = self.width;
        let height = self.height;
        let damping = 1.0 - self.damping as f32;
        let base = (self.speed * self.speed) as f32;

        for row in 1..height - 1 {
            for column in 1..width - 1 {
                let cell = row * width + column;
                if self.solid[cell] {
                    self.next[cell] = 0.0;
                    continue;
                }
                let here = self.now[cell];
                let laplacian = self.now[cell - 1]
                    + self.now[cell + 1]
                    + self.now[cell - width]
                    + self.now[cell + width]
                    - 4.0 * here;
                let n = self.medium[cell];
                let c2 = base / (n * n);
                self.next[cell] = ((2.0 * here) - self.before[cell] + c2 * laplacian) * damping;
            }
        }

        self.apply_edges();

        std::mem::swap(&mut self.before, &mut self.now);
        std::mem::swap(&mut self.now, &mut self.next);
    }

    fn apply_edges(&mut self) {
        let width = self.width;
        let height = self.height;

        match self.edge {
            Edge::Reflect => {
                for column in 0..width {
                    self.next[column] = 0.0;
                    self.next[(height - 1) * width + column] = 0.0;
                }
                for row in 0..height {
                    self.next[row * width] = 0.0;
                    self.next[row * width + width - 1] = 0.0;
                }
            }
            Edge::Absorb => {
                let k = (self.speed - 1.0) / (self.speed + 1.0);
                let k = k as f32;
                for column in 0..width {
                    let top = column;
                    let inner = width + column;
                    self.next[top] = self.now[inner] + k * (self.next[inner] - self.now[top]);

                    let bottom = (height - 1) * width + column;
                    let inner = (height - 2) * width + column;
                    self.next[bottom] = self.now[inner] + k * (self.next[inner] - self.now[bottom]);
                }
                for row in 0..height {
                    let left = row * width;
                    let inner = left + 1;
                    self.next[left] = self.now[inner] + k * (self.next[inner] - self.now[left]);

                    let right = row * width + width - 1;
                    let inner = right - 1;
                    self.next[right] = self.now[inner] + k * (self.next[inner] - self.now[right]);
                }
            }
        }
    }

    fn step_heat(&mut self) {
        let width = self.width;
        let height = self.height;
        let alpha = self.diffusivity as f32;

        for row in 1..height - 1 {
            for column in 1..width - 1 {
                let cell = row * width + column;
                if self.solid[cell] {
                    self.next[cell] = 0.0;
                    continue;
                }
                let here = self.now[cell];
                let laplacian = self.now[cell - 1]
                    + self.now[cell + 1]
                    + self.now[cell - width]
                    + self.now[cell + width]
                    - 4.0 * here;
                self.next[cell] = here + alpha * laplacian;
            }
        }

        for column in 0..width {
            self.next[column] = self.next[width + column];
            self.next[(height - 1) * width + column] = self.next[(height - 2) * width + column];
        }
        for row in 0..height {
            self.next[row * width] = self.next[row * width + 1];
            self.next[row * width + width - 1] = self.next[row * width + width - 2];
        }

        std::mem::swap(&mut self.now, &mut self.next);
        self.stamp_sources();
    }

    fn solve_charge(&mut self) {
        let width = self.width as f64;
        let height = self.height as f64;
        let softening = (0.006 * width).powi(2);

        for row in 0..self.height {
            for column in 0..self.width {
                let cell = row * self.width + column;
                let mut potential = 0.0;
                for source in &self.sources {
                    let dx = column as f64 - source.x * width;
                    let dy = row as f64 - source.y * height;
                    potential += source.strength / (dx * dx + dy * dy + softening).sqrt();
                }
                self.now[cell] = (potential * width * 0.06) as f32;
            }
        }
    }

    pub fn energy(&self) -> f64 {
        match self.kind {
            Kind::Wave => {
                let c2 = self.speed * self.speed;
                let mut kinetic = 0.0f64;
                let mut potential = 0.0f64;
                for row in 1..self.height - 1 {
                    for column in 1..self.width - 1 {
                        let cell = row * self.width + column;
                        let velocity = (self.now[cell] - self.before[cell]) as f64;
                        kinetic += velocity * velocity;

                        let across_now = (self.now[cell + 1] - self.now[cell]) as f64;
                        let across_before = (self.before[cell + 1] - self.before[cell]) as f64;
                        let down_now = (self.now[cell + self.width] - self.now[cell]) as f64;
                        let down_before =
                            (self.before[cell + self.width] - self.before[cell]) as f64;
                        potential += across_now * across_before + down_now * down_before;
                    }
                }
                0.5 * (kinetic + c2 * potential) / (self.width * self.height) as f64
            }
            Kind::Heat => {
                self.now.iter().map(|v| *v as f64).sum::<f64>() / (self.width * self.height) as f64
            }
            Kind::Charge => 0.0,
        }
    }

    pub fn probe(&self, x: f64, y: f64) -> f64 {
        let column = ((x * self.width as f64) as usize).min(self.width - 1);
        let row = ((y * self.height as f64) as usize).min(self.height - 1);
        self.now[row * self.width + column] as f64
    }

    fn measure_peak(&mut self) {
        let mut total = 0.0f64;
        let mut counted = 0usize;
        for (cell, value) in self.now.iter().enumerate() {
            if self.solid[cell] {
                continue;
            }
            total += value.abs() as f64;
            counted += 1;
        }
        let typical = if counted == 0 {
            0.0
        } else {
            (total / counted as f64) as f32
        };
        self.peak = if self.peak <= 0.0 {
            typical
        } else {
            self.peak * 0.9 + typical * 0.1
        };
    }

    pub fn paint(&mut self, gain: f64, contours: bool) -> &[u8] {
        if self.kind == Kind::Charge && self.dirty {
            self.solve_charge();
            self.dirty = false;
        }

        let gain = if gain > 0.0 {
            gain as f32
        } else {
            self.measure_peak();
            0.38 / self.peak.max(0.004)
        };
        let signed = self.kind != Kind::Heat;

        for (cell, value) in self.now.iter().enumerate() {
            let level = if signed {
                clamp01(0.5 + 0.5 * (value * gain).tanh())
            } else {
                clamp01((value * gain).tanh())
            };

            let mut colour = self.palette[(level * 255.0) as usize];

            if contours && (0.03..0.97).contains(&level) {
                let bands = 11.0;
                let scaled = level * bands;
                let ripple = (scaled - scaled.floor() - 0.5).abs();
                if ripple > 0.44 {
                    let weight = (ripple - 0.44) / 0.06;
                    colour = mix(colour, [236, 240, 248], weight * 0.5);
                }
            }

            if self.solid[cell] {
                colour = [58, 64, 78];
            } else if self.medium[cell] != 1.0 {
                colour = mix(colour, [150, 168, 200], 0.3);
            }

            let base = cell * 4;
            self.pixels[base] = colour[0];
            self.pixels[base + 1] = colour[1];
            self.pixels[base + 2] = colour[2];
            self.pixels[base + 3] = 255;
        }

        &self.pixels
    }
}

pub struct Preset {
    pub id: &'static str,
    pub kind: Kind,
}

pub const PRESETS: &[Preset] = &[
    Preset {
        id: "ripples",
        kind: Kind::Wave,
    },
    Preset {
        id: "double-slit",
        kind: Kind::Wave,
    },
    Preset {
        id: "single-slit",
        kind: Kind::Wave,
    },
    Preset {
        id: "lens",
        kind: Kind::Wave,
    },
    Preset {
        id: "drum",
        kind: Kind::Wave,
    },
    Preset {
        id: "harbour",
        kind: Kind::Wave,
    },
    Preset {
        id: "hotspot",
        kind: Kind::Heat,
    },
    Preset {
        id: "radiator",
        kind: Kind::Heat,
    },
    Preset {
        id: "heatsink",
        kind: Kind::Heat,
    },
    Preset {
        id: "dipole",
        kind: Kind::Charge,
    },
    Preset {
        id: "quadrupole",
        kind: Kind::Charge,
    },
    Preset {
        id: "capacitor",
        kind: Kind::Charge,
    },
];

pub fn preset(id: &str) -> Option<&'static Preset> {
    PRESETS.iter().find(|entry| entry.id == id)
}

impl Field {
    pub fn build(id: &str, width: usize, height: usize) -> Option<Field> {
        let entry = preset(id)?;
        let mut field = Field::new(entry.kind, width, height);
        field.apply(id);
        Some(field)
    }

    pub fn apply(&mut self, id: &str) {
        self.clear_geometry();
        self.clear();
        self.set_edge(Edge::Absorb);
        self.set_speed(0.45);
        self.set_damping(0.0);
        self.set_diffusivity(0.2);

        match id {
            "ripples" => {
                self.add_source(Source::point(0.5, 0.5, 1.0, 5.5));
            }
            "double-slit" => {
                self.add_source(Source::line(0.06, 0.5, 0.92, 1.0, 14.0));
                self.barrier(0.32, &[(0.38, 0.024), (0.62, 0.024)], 0.014);
            }
            "single-slit" => {
                self.add_source(Source::line(0.06, 0.5, 0.92, 1.0, 14.0));
                self.barrier(0.32, &[(0.5, 0.028)], 0.014);
            }
            "lens" => {
                self.add_source(Source::line(0.05, 0.5, 0.92, 1.0, 9.0));
                self.lens(0.42, 0.5, 0.17, 1.9);
            }
            "drum" => {
                self.set_edge(Edge::Reflect);
                self.set_damping(0.0004);
                self.poke(0.36, 0.42, 0.035, 1.0);
            }
            "harbour" => {
                self.add_source(Source::point(0.16, 0.5, 1.0, 7.0));
                self.barrier(0.5, &[(0.5, 0.09)], 0.016);
                self.wall(0.78, 0.22, 0.11, true);
                self.wall(0.78, 0.78, 0.11, true);
            }
            "hotspot" => {
                self.set_diffusivity(0.22);
                self.poke(0.5, 0.5, 0.09, 1.0);
            }
            "radiator" => {
                self.set_diffusivity(0.2);
                self.add_source(Source::line(0.1, 0.5, 0.5, 1.0, 0.0));
                self.add_source(Source::line(0.9, 0.5, 0.5, -0.6, 0.0));
                self.clear();
            }
            "heatsink" => {
                self.set_diffusivity(0.22);
                self.add_source(Source::point(0.22, 0.5, 1.0, 0.0));
                self.wall(0.5, 0.26, 0.1, true);
                self.wall(0.5, 0.74, 0.1, true);
                self.clear();
            }
            "dipole" => {
                self.add_source(Source::point(0.36, 0.5, 1.0, 0.0));
                self.add_source(Source::point(0.64, 0.5, -1.0, 0.0));
            }
            "quadrupole" => {
                self.add_source(Source::point(0.38, 0.34, 1.0, 0.0));
                self.add_source(Source::point(0.62, 0.34, -1.0, 0.0));
                self.add_source(Source::point(0.38, 0.66, -1.0, 0.0));
                self.add_source(Source::point(0.62, 0.66, 1.0, 0.0));
            }
            "capacitor" => {
                for step in 0..4 {
                    let y = 0.3 + step as f64 * 0.135;
                    self.add_source(Source::point(0.36, y, 0.8, 0.0));
                    self.add_source(Source::point(0.64, y, -0.8, 0.0));
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wave(width: usize, height: usize) -> Field {
        let mut field = Field::new(Kind::Wave, width, height);
        field.set_edge(Edge::Reflect);
        field
    }

    #[test]
    fn a_still_field_stays_still() {
        let mut field = wave(64, 64);
        field.advance(1.0);
        assert!(field.energy() < 1e-12);
    }

    #[test]
    fn a_pulse_spreads_outwards() {
        let mut field = wave(129, 129);
        field.poke(0.5, 0.5, 0.03, 1.0);

        let centre = field.probe(0.5, 0.5).abs();
        let far = field.probe(0.5, 0.15).abs();
        assert!(centre > 0.5, "the pulse should start at the centre");
        assert!(far < 1e-6, "nothing should have arrived yet");

        field.advance(0.6);
        assert!(
            field.probe(0.5, 0.15).abs() > 1e-4,
            "the wave should have reached the edge of the grid"
        );
    }

    #[test]
    fn a_closed_box_keeps_its_energy() {
        let mut field = wave(96, 96);
        field.poke(0.5, 0.5, 0.05, 1.0);
        field.advance(0.05);
        let start = field.energy();
        field.advance(1.5);
        let end = field.energy();
        assert!(
            (end - start).abs() / start < 0.35,
            "energy drifted from {start} to {end}"
        );
    }

    #[test]
    fn damping_takes_energy_away() {
        let mut field = wave(96, 96);
        field.set_damping(0.004);
        field.poke(0.5, 0.5, 0.05, 1.0);
        field.advance(0.05);
        let start = field.energy();
        field.advance(1.5);
        assert!(field.energy() < start * 0.6, "damping did nothing");
    }

    #[test]
    fn walls_stay_silent() {
        let mut field = wave(96, 96);
        field.wall(0.25, 0.5, 0.06, true);
        field.poke(0.5, 0.5, 0.05, 1.0);
        field.advance(1.0);
        assert!(field.probe(0.25, 0.5).abs() < 1e-9);
    }

    #[test]
    fn heat_flows_from_hot_to_cold() {
        let mut field = Field::new(Kind::Heat, 96, 96);
        field.poke(0.5, 0.5, 0.06, 1.0);
        let peak = field.probe(0.5, 0.5);
        let away = field.probe(0.7, 0.5);
        field.advance(0.4);
        assert!(field.probe(0.5, 0.5) < peak, "the hot spot should cool");
        assert!(field.probe(0.7, 0.5) > away, "the surroundings should warm");
    }

    #[test]
    fn heat_never_runs_backwards() {
        let mut field = Field::new(Kind::Heat, 80, 80);
        field.poke(0.5, 0.5, 0.08, 1.0);
        let mut previous = f64::MAX;
        for _ in 0..30 {
            field.advance(0.1);
            let peak = field.probe(0.5, 0.5);
            assert!(
                peak <= previous + 1e-6,
                "the peak grew from {previous} to {peak}"
            );
            previous = peak;
        }
    }

    #[test]
    fn opposite_charges_make_opposite_signs() {
        let mut field = Field::new(Kind::Charge, 96, 96);
        field.add_source(Source::point(0.3, 0.5, 1.0, 0.0));
        field.add_source(Source::point(0.7, 0.5, -1.0, 0.0));
        field.paint(1.0, true);

        assert!(field.probe(0.3, 0.5) > 0.0);
        assert!(field.probe(0.7, 0.5) < 0.0);
        assert!(
            field.probe(0.5, 0.5).abs() < 1e-6,
            "the midpoint of a dipole must be neutral"
        );
    }

    #[test]
    fn a_single_charge_falls_off_like_one_over_r() {
        let mut field = Field::new(Kind::Charge, 201, 201);
        field.add_source(Source::point(0.5, 0.5, 1.0, 0.0));
        field.paint(1.0, false);

        let near = field.probe(0.6, 0.5);
        let far = field.probe(0.7, 0.5);
        let ratio = near / far;
        assert!(
            (ratio - 2.0).abs() < 0.05,
            "twice the distance should halve the potential, got {ratio}"
        );
    }

    fn brightness(field: &mut Field, x: f64, seconds: f64) -> Vec<f64> {
        let column = ((x * field.width() as f64) as usize).min(field.width() - 1);
        let mut peak = vec![0.0f64; field.height()];
        let slices = 120;
        for _ in 0..slices {
            field.advance(seconds / slices as f64);
            for (row, slot) in peak.iter_mut().enumerate() {
                let value = field.now[row * field.width() + column].abs() as f64;
                if value > *slot {
                    *slot = value;
                }
            }
        }
        peak
    }

    fn lobes(profile: &[f64]) -> usize {
        let highest = profile.iter().cloned().fold(0.0f64, f64::max);
        if highest <= 0.0 {
            return 0;
        }
        let floor = highest * 0.16;
        let mut count = 0;
        let mut inside = false;
        for value in profile {
            if *value > floor && !inside {
                inside = true;
                count += 1;
            } else if *value < floor * 0.8 {
                inside = false;
            }
        }
        count
    }

    #[test]
    fn every_preset_builds_and_stays_finite() {
        for entry in PRESETS {
            let mut field = Field::build(entry.id, 96, 64).expect(entry.id);
            field.advance(2.0);
            field.paint(0.0, true);
            assert!(
                field.now.iter().all(|v| v.is_finite()),
                "{} blew up",
                entry.id
            );
        }
    }

    #[test]
    fn one_slit_gives_one_lobe_and_two_slits_give_several() {
        let mut single = Field::build("single-slit", 260, 200).expect("single");
        let mut double = Field::build("double-slit", 260, 200).expect("double");

        let one = brightness(&mut single, 0.8, 3.0);
        let two = brightness(&mut double, 0.8, 3.0);

        assert_eq!(lobes(&one), 1, "a single slit should spread into one lobe");
        assert!(
            lobes(&two) >= 3,
            "two slits should interfere into several lobes, saw {}",
            lobes(&two)
        );
    }

    #[test]
    fn a_slow_disc_focuses_the_beam() {
        let mut with_lens = Field::build("lens", 260, 200).expect("lens");
        let mut without = Field::build("lens", 260, 200).expect("lens");
        without.medium.iter_mut().for_each(|n| *n = 1.0);

        let focused = brightness(&mut with_lens, 0.6, 3.0);
        let plain = brightness(&mut without, 0.6, 3.0);

        let middle = focused.len() / 2;
        let on_axis: f64 = focused[middle - 5..middle + 5].iter().sum();
        let reference: f64 = plain[middle - 5..middle + 5].iter().sum();

        assert!(
            on_axis > reference * 1.3,
            "the same wave should be brighter on the axis behind the lens, {on_axis} vs {reference}"
        );
    }

    #[test]
    fn a_wall_casts_a_shadow() {
        let mut field = Field::build("ripples", 200, 160).expect("ripples");
        field.move_source(0, 0.18, 0.5);
        field.wall(0.55, 0.5, 0.17, true);
        let profile = brightness(&mut field, 0.86, 2.5);

        let middle = profile.len() / 2;
        let shadow: f64 = profile[middle - 4..middle + 4].iter().sum();
        let open: f64 = profile[6..14].iter().sum();

        assert!(
            shadow < open,
            "the far side of the wall should be quieter, {shadow} vs {open}"
        );
    }

    #[test]
    fn painting_fills_every_pixel() {
        let mut field = wave(48, 32);
        field.poke(0.5, 0.5, 0.1, 1.0);
        let pixels = field.paint(1.0, false);
        assert_eq!(pixels.len(), 48 * 32 * 4);
        assert!(pixels.chunks_exact(4).all(|p| p[3] == 255));
    }
}

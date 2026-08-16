# kosmos

An interactive calculator for the natural world — mathematics, physics and biology, computed in
Rust and rendered live in your browser.

Named after Alexander von Humboldt's *Kosmos*, his attempt to describe the whole of nature in one
work.

```sh
docker run -p 8080:80 ghcr.io/justinjd00/kosmos
```

Nothing is sent anywhere. The entire engine — parser, calculus, solvers — is compiled to
WebAssembly and runs on your machine, offline, with no accounts and no telemetry.

## Why Rust

A plotter is easy. What is hard is everything you want *after* the plot: a Reaction-Diffusion
system at 512×512 stepping at 60 frames per second, a three-body problem integrated live while you
drag a planet, a hundred thousand adaptive samples recomputed on every zoom. That is not a
JavaScript workload. The numerical core is Rust compiled to WebAssembly; the interface is React,
which is exactly the right tool for the other half of the job.

## What works today

**Functions & calculus**

- A real expression language: `2x`, `3sin(x)` and `(x+1)(x-1)` parse without multiplication signs
- **Symbolic differentiation** — not finite differences. `sin(x^2)` gives you `cos(x^2)*2*x`,
  printed and plotted
- **Adaptive sampling**: flat stretches get few points, tight curvature gets many, so a plot stays
  smooth at any zoom without wasting work
- **Poles are distinguished from roots.** Where a sign change happens, the curve is bisected — if
  the values collapse toward zero it is a root, if they grow it is a pole and the line breaks
  instead of drawing a false vertical stroke through `tan(x)`
- Roots and extrema found numerically and classified as maxima or minima
- Live parameters `a b c d` on sliders, and a time variable `t` that animates

**Chaos**

Eight systems integrated live with a fourth-order Runge-Kutta scheme: Lorenz, Rössler, Aizawa,
Thomas, Halvorsen and Chen in 3D (drag to rotate), plus the double pendulum and the three-body
problem.

Two of them are there because they can be checked rather than just admired:

- The **double pendulum** conserves energy. The readout stays flat to within one part in a
  hundred thousand over 20,000 steps — that number is a test in the suite, so a bad edit to the
  integrator fails CI rather than quietly producing plausible nonsense.
- The **three-body problem** starts on the figure-eight orbit found by Cristopher Moore in 1993.
  It is exactly periodic, and the test suite integrates one full period and checks that all
  twelve state variables come back to where they started.

**Butterfly twin** launches a second copy of the running system one ten-millionth away from the
first. Same equations, same machine, no randomness anywhere. Watch the separation readout climb
from `1e-7` to `1e+1` and the two coloured trajectories peel apart. That is what "chaos" means,
and it is the reason weather forecasts stop working after about ten days.

## Self-hosting

kosmos is a static site plus a `.wasm` file. Anything that serves files can host it.

**Docker (recommended)**

```sh
git clone https://github.com/justinjd00/kosmos
cd kosmos
docker compose up -d          # http://localhost:8080
```

Change the port with `KOSMOS_PORT=9000 docker compose up -d`.

**Any web server**

```sh
cd web && npm ci && npm run build
```

Copy `web/dist/` wherever you like. Two things matter:

1. `.wasm` files must be served as `application/wasm`. nginx and Caddy do this by default;
   `python3 -m http.server` does not, and the app will fail to start.
2. The build uses relative paths, so it works from a subdirectory
   (`https://example.com/tools/kosmos/`) with no configuration.

A ready-made nginx config is in [`deploy/nginx.conf`](deploy/nginx.conf).

**Caddy**

```
example.com {
    root * /srv/kosmos
    file_server
    try_files {path} /index.html
}
```

## Building from source

You need Rust (1.75+), [wasm-pack](https://rustwasm.github.io/wasm-pack/) and Node 20+.

```sh
cargo install wasm-pack
cd core && wasm-pack build --release --target web --out-dir ../web/src/wasm --out-name kosmos
cd ../web && npm ci && npm run dev
```

Run the engine's test suite with `cd core && cargo test` — 40 tests covering the parser, the
evaluator, symbolic derivatives (checked against finite differences), pole detection, adaptive
sampling, energy conservation and orbital periodicity.

## Syntax

| | |
|---|---|
| Operators | `+ - * / ^ %`, brackets `( ) [ ] { }` |
| Implicit product | `2x`, `3sin(x)`, `(x+1)(x-1)`, `2pi` |
| One argument | `sin cos tan asin acos atan sinh cosh tanh exp ln log2 log10 sqrt cbrt abs sign floor ceil round` |
| Two arguments | `min max atan2 hypot pow log` |
| Constants | `pi tau e phi` |
| Variables | `x`, time `t`, parameters `a b c d` |

Errors carry the exact character position, so a typo is underlined rather than described.

## Controls

| | |
|---|---|
| Drag | pan |
| Scroll | zoom at the cursor |
| Shift + scroll | zoom the x axis only |
| Alt + scroll | zoom the y axis only |
| Double-click | reset the view |

## Roadmap

- **Fields** — the wave equation, heat diffusion, electric fields, solved live
- **Life** — Turing patterns, predator–prey dynamics, epidemics, cellular automata

## License

MIT

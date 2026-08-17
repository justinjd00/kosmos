<div align="center">

# kosmos

### An interactive calculator for the natural world

Mathematics, physics and biology — computed in Rust, rendered live in your browser.

[![ci](https://img.shields.io/github/actions/workflow/status/justinjd00/kosmos/ci.yml?branch=main&style=for-the-badge&label=ci&labelColor=0f1117&color=5eead4)](https://github.com/justinjd00/kosmos/actions)
[![live demo](https://img.shields.io/badge/live-demo-a78bfa?style=for-the-badge&labelColor=0f1117)](https://justinjd00.github.io/kosmos/)
[![container](https://img.shields.io/badge/ghcr.io-kosmos-60a5fa?style=for-the-badge&labelColor=0f1117&logo=docker&logoColor=white)](https://github.com/justinjd00/kosmos/pkgs/container/kosmos)
[![license](https://img.shields.io/badge/license-MIT-fbbf24?style=for-the-badge&labelColor=0f1117)](LICENSE)
[![proved in Lean 4](https://img.shields.io/badge/calculus-proved%20in%20Lean%204-fb7185?style=for-the-badge&labelColor=0f1117)](proofs/)
[![algebra in OCaml](https://img.shields.io/badge/algebra-OCaml-ee6a1a?style=for-the-badge&labelColor=0f1117)](cas/)

**[Open it →](https://justinjd00.github.io/kosmos/)**

<br>

<img src="docs/lorenz.jpg" alt="The Lorenz attractor with a butterfly twin: two trajectories that started a ten-millionth apart, now completely separated" width="100%">

</div>

<br>

> [!NOTE]
> Nothing is sent anywhere. The parser, the calculus, the solvers — all of it is Rust compiled to
> WebAssembly, running on your machine. No account, no telemetry, no network calls. Load the page
> once and it works on a plane.

## Why bother with Rust

A function plotter is easy in any language. What is hard is everything you want *after* the plot:
a reaction–diffusion grid stepping at 60 frames per second, a three-body problem integrated while
you drag a planet, a hundred thousand adaptive samples recomputed on every zoom. That is not a
JavaScript workload.

So the split is: **Rust computes, React presents.** The canvas receives finished `Float32Array`
point lists straight out of linear memory — no JSON, no per-point marshalling across the boundary.

```mermaid
flowchart LR
    A["Your input<br/><i>sin(x²) · Lorenz · σ=10</i>"] --> B

    subgraph engine ["Rust engine → WebAssembly"]
        direction TB
        B["Lexer + Pratt parser"] --> C["AST"]
        C --> D["Bytecode<br/><i>stack machine, zero allocation</i>"]
        C --> E["Symbolic derivative<br/><i>+ simplifier</i>"]
        D --> F["Adaptive sampler<br/><i>+ pole detection</i>"]
        E --> F
        G["RK4 integrator<br/><i>8 dynamical systems</i>"] --> H["Ring-buffered trail<br/><i>projected 3D → 2D</i>"]
    end

    F --> I["Float32Array"]
    H --> I
    I --> J["React + Canvas 2D"]

    style engine fill:#0f1117,stroke:#5eead4,color:#e7eaf1
```

<br>

## Functions & calculus

<img src="docs/functions.jpg" alt="Two plotted functions: one showing its symbolic derivative as a dashed curve, the other with roots and extrema marked" width="100%">

**The derivative is symbolic, not numerical.** Type `sin(x^2)` and the engine differentiates the
syntax tree by the chain rule, simplifies the result, and hands back a real expression:

$$\frac{d}{dx}\sin(x^2) = \cos(x^2)\cdot 2x$$

It is printed in the sidebar and plotted as a dashed curve. Every rule is checked against a central
difference quotient — two independent routes to the same number:

$$\left|\ \underbrace{f'(x)}_{\text{symbolic}} - \underbrace{\frac{f(x+h)-f(x-h)}{2h}}_{\text{numerical}}\ \right| < 10^{-5}$$

**Poles are told apart from roots.** Both look identical to a naive plotter: the sign flips. So at
every sign change the interval is bisected fourteen times and the values are watched. Collapsing
toward zero means a root. Growing without bound means a pole — and the line breaks instead of
drawing a vertical stroke that isn't there:

$$\min\big(|f(a)|,|f(b)|\big) > \max\big(|f(x_0)|,|f(x_2)|\big) \implies \text{pole}$$

That single check is why `tan(x)` looks right.

**Sampling follows curvature, not the x-axis.** Flat stretches get a handful of points, tight turns
get hundreds — and the bend is measured in *screen space*, because whether a curve looks kinked
depends on your zoom, not on the numbers.

<details>
<summary><b>What the expression language accepts</b></summary>

<br>

| | |
|---|---|
| **Operators** | `+ - * / ^ %` and brackets `( ) [ ] { }` |
| **Implicit product** | `2x`, `3sin(x)`, `(x+1)(x-1)`, `2pi` — no `*` needed |
| **One argument** | `sin cos tan asin acos atan sinh cosh tanh exp ln log2 log10 sqrt cbrt abs sign floor ceil round` |
| **Two arguments** | `min max atan2 hypot pow log` |
| **Constants** | `pi tau e phi` |
| **Variables** | `x`, time `t`, and the parameters `a b c d` on sliders |

Errors carry the exact character position, so a typo gets underlined rather than described:

```
2 + $
    ^ unexpected character '$'
```

Precedence comes from a Pratt parser, so `2^3^2` is $2^{(3^2)} = 512$ and `1-2-3` is
$(1-2)-3 = -4$, exactly as mathematics expects. `3x^2` parses as $3x^2$, not $(3x)^2$.

</details>

<br>

## Algebra, in OCaml

<img src="docs/algebra.jpg" alt="The algebra panel: an antiderivative and a Taylor series computed in the browser, each with a button that plots the result as a new curve" width="100%">

Press **algebra** on any function and a third language joins in. Integration, solving
and series expansion are handled by a small computer-algebra module written in
[OCaml](https://ocaml.org) and compiled to JavaScript — see [`cas/`](cas/).

| | |
|---|---|
| **∫ dx** | a symbolic antiderivative, or an honest refusal |
| **solve = 0** | every real root it can find |
| **Taylor** | the series to any order up to 12 |
| **simplify** | collect, fold and multiply out |

Every result is one click from being plotted, so you can put a function and its
antiderivative — or a function and its sixth-order approximation — on the same axes.

### Why not just write it in Rust

The engine already differentiates, and Rust is the right tool for that: it is one
pass over a tree, and the result runs sixty times a second inside a bytecode
interpreter with no allocation.

Integration is a different shape of problem. There is no formula — it is a
**search**: try the power rule, try substituting `u = g(x)`, try parts, back out
and try the next rule, on immutable trees that are built and discarded by the
thousand. In Rust that is `Box`, `clone` and lifetimes threaded through a
backtracking search. In OCaml it is what the language was made for.

```ocaml
| Fn (name, [ arg ]) -> (
    match slope_of arg with
    | Some k -> ( match basic name arg with Some anti -> div anti k | None -> give_up ())
    | None -> give_up ())
```

So each language keeps the job it is best at:

```mermaid
flowchart LR
    L["<b>Lean 4</b><br/>proves the<br/>differentiation rules"] -->|"a corpus<br/>of numbers"| R
    O["<b>OCaml</b><br/>integrates, solves,<br/>expands in series"] -->|"a table<br/>of answers"| R
    R["<b>Rust → WebAssembly</b><br/>parses, compiles, evaluates<br/>60 times a second"]

    style L fill:#0f1117,stroke:#fb7185,color:#e7eaf1
    style O fill:#0f1117,stroke:#ee6a1a,color:#e7eaf1
    style R fill:#0f1117,stroke:#5eead4,color:#e7eaf1
```

> [!IMPORTANT]
> **No antiderivative is returned unchecked.** Before an answer leaves the module it
> is differentiated again and compared against the original integrand at six points.
> A wrong branch of the search therefore produces *"no closed form found"* — never a
> wrong formula.

And because two languages now describe the same mathematics, they are held to one
table. [`cas/expected.txt`](cas/expected.txt) records what the module answers;
[`core/tests/algebra.rs`](core/tests/algebra.rs) makes the **Rust** engine check it:
every answer must parse with the Rust parser, every integral must differentiate back
to its integrand under the Rust differentiator, and every series must hug its function
near the centre. Neither implementation can drift without the other noticing.

<details>
<summary><b>What it knows, and what it does not</b></summary>

<br>

Integration covers the power rule, linearity, the standard forms, linear
substitution, general substitution `u = g(x)` discovered by dividing by `g'(x)`,
integration by parts, expansion of products of sums, the two quadratic denominators
that give `atan` and `asin`, and the cyclic case `e^(kx)·sin(mx)`.

```
∫ x² sin(x) dx   →   -x²·cos(x) + 2x·sin(x) + 2cos(x)
∫ 2x·e^(x²) dx   →   e^(x²)
∫ 1/√(1-x²) dx   →   asin(x)
∫ e^(-x²) dx     →   no closed form found
```

The last one has no elementary antiderivative at all. The module cannot tell that
apart from an integral it merely failed to find — there is no Risch algorithm here —
so a refusal means *"not found"*, not *"does not exist"*.

Solving is closed-form to quadratics, Newton from four hundred starting points above
that, and otherwise peels functions off the outside (`ln(x) = 0` → `x = e⁰`). Roots
are substituted back into the original equation before being shown.

Out of scope: complex numbers, matrices, assumptions about a variable's sign, and
partial fractions.

</details>

<br>

## Chaos

Eight systems, integrated live with fourth-order Runge–Kutta. Drag to rotate the spatial ones.

<table>
<tr>
<td width="33%"><img src="docs/aizawa.jpg" alt="Aizawa attractor"></td>
<td width="33%"><img src="docs/thomas.jpg" alt="Thomas attractor"></td>
<td width="33%"><img src="docs/halvorsen.jpg" alt="Halvorsen attractor"></td>
</tr>
<tr>
<td align="center"><a href="https://justinjd00.github.io/kosmos/#chaos/aizawa"><b>Aizawa</b></a></td>
<td align="center"><a href="https://justinjd00.github.io/kosmos/#chaos/thomas"><b>Thomas</b></a></td>
<td align="center"><a href="https://justinjd00.github.io/kosmos/#chaos/halvorsen"><b>Halvorsen</b></a></td>
</tr>
</table>

The Lorenz system is three lines of arithmetic — a stripped-down model of a convecting fluid:

$$
\dot{x} = \sigma(y-x), \qquad
\dot{y} = x(\rho-z)-y, \qquad
\dot{z} = xy-\beta z
$$

With $\sigma=10$, $\rho=28$, $\beta=8/3$ it never repeats and never escapes. In 1963 Edward Lorenz
restarted a weather simulation from a printout rounded to three digits instead of six and got a
completely different forecast. That accident is where chaos theory starts.

| System | Dim. | Why it's here |
|---|:--:|---|
| [Lorenz](https://justinjd00.github.io/kosmos/#chaos/lorenz) | 3 | The original. Convection, and the discovery of chaos |
| [Rössler](https://justinjd00.github.io/kosmos/#chaos/rossler) | 3 | Designed in 1976 to be the *simplest* possible chaotic system |
| [Aizawa](https://justinjd00.github.io/kosmos/#chaos/aizawa) | 3 | A sphere with an axis drilled through it |
| [Thomas](https://justinjd00.github.io/kosmos/#chaos/thomas) | 3 | Cyclically symmetric — each axis driven by the sine of the next |
| [Halvorsen](https://justinjd00.github.io/kosmos/#chaos/halvorsen) | 3 | Three-fold symmetry, three curled arms |
| [Chen](https://justinjd00.github.io/kosmos/#chaos/chen) | 3 | A cousin of Lorenz, not topologically equivalent to it |
| [Double pendulum](https://justinjd00.github.io/kosmos/#chaos/double-pendulum) | 4 | Conserves energy — which makes it a test |
| [Three-body problem](https://justinjd00.github.io/kosmos/#chaos/three-body) | 12 | Exactly periodic — which makes it a better test |

<br>

## The butterfly effect, as a button

> [!TIP]
> Open [**Lorenz with a butterfly twin**](https://justinjd00.github.io/kosmos/#chaos/lorenz+twin)
> and watch the `separation` readout climb.

**Butterfly twin** forks the running system and displaces it by $10^{-7}$ — about the width of a
virus, on a scale of tens. Same equations. Same machine. No randomness anywhere in the code.

The gap between the two grows exponentially, governed by the largest Lyapunov exponent, which is
$\lambda \approx 0.9$ for Lorenz:

$$\delta(t) \approx \delta_0\, e^{\lambda t}$$

so a difference in the seventh decimal reaches order 1 in roughly

$$t \approx \frac{\ln(10^{7})}{0.9} \approx 18\ \text{seconds}$$

The image at the top of this page is that moment several times over: started $10^{-7}$ apart, now
$2.4 \times 10^{1}$ apart, two coloured threads wandering the same shape along completely different
paths. This is also, precisely, why weather forecasts fall apart after about ten days — not because
the models are bad, but because the atmosphere multiplies every measurement error.

<br>

## The calculus is machine-checked

Testing a derivative means picking a few functions and a few points. That catches
blunders, not subtleties — and a wrong rule that happens to agree at `x = 1` will
sail through.

So the differentiation rules are also **proved**, in [Lean 4](https://lean-lang.org)
against [Mathlib](https://github.com/leanprover-community/mathlib4). The expression
tree, the evaluator and the differentiator are mirrored in
[`proofs/`](proofs/), and the theorem says what you would want it to say:

$$\forall\, e \in \texttt{Expr},\ \forall x \in \mathbb{R}: \quad \texttt{HasDerivAt}\ (\texttt{eval } e)\ \big(\texttt{eval } (\texttt{derive } e)\ x\big)\ x$$

For every expression the engine can build, at every point of the real line. Not
sampled — proved, by induction over the expression tree. A second theorem covers
the simplifier: `eval (simplify e) x = eval e x`, so rewriting can never change
what an expression means.

> [!NOTE]
> `#print axioms` reports only Lean's three standard axioms — `propext`,
> `Classical.choice`, `Quot.sound`. No `sorryAx`, so there are no holes, and CI
> fails if one ever appears.

**Where the guarantee stops.** Those theorems are about the Lean definitions, not
about `calculus.rs` — Lean cannot see the Rust file. The gap is narrowed by
generating test data *from the verified function*: `proofs/Proofs/Bridge.lean`
evaluates the proved derivative and writes
[`core/tests/verified-derivatives.txt`](core/tests/verified-derivatives.txt), and
the Rust suite must reproduce every number with its own rules, plus agree with the
printed derivative across a further 1,600 points. CI regenerates the file and
fails on any difference, so the two cannot drift apart unnoticed.

That is weaker than extracting verified code and stronger than a hand-written test
suite. Stated plainly rather than dressed up.

The proved fragment is `+ - * ^ sin cos exp` and every composition of them —
differentiable everywhere, so the theorem needs no side conditions. Division,
`ln`, `sqrt` and `tan` have domain holes and need explicit hypotheses; that is
the next piece.

<br>

## Two systems that double as proofs

Simulations are easy to get subtly wrong, because *plausible* and *correct* look the same on
screen — and in a chaotic system everything looks different anyway. So the suite leans on
**invariants**: quantities that mathematics forbids from changing.

<table>
<tr>
<td width="50%" valign="top">

<img src="docs/pendulum.jpg" alt="Double pendulum with a butterfly twin">

**Energy must stay flat**

A frictionless pendulum cannot gain or lose energy. After 20,000 steps the test demands

$$\frac{|E(t) - E_0|}{|E_0|} < 10^{-4}$$

Break the integrator and the readout drifts, so CI fails instead of quietly rendering something
that merely looks like a pendulum.

</td>
<td width="50%" valign="top">

<img src="docs/three-body.jpg" alt="The figure-eight three-body orbit">

**The orbit must close**

Three equal masses chasing each other along a figure eight — found by Cristopher Moore in 1993 and
exactly periodic with $T = 6.3244490664$.

The test integrates one full period and checks that **all twelve** state variables return to where
they began. One wrong sign anywhere and the orbit never comes home.

</td>
</tr>
</table>

<br>

## Deep links

Every view has a URL, so a system, an experiment or a pre-run state can be shared as a plain link.

| Link | Meaning |
|---|---|
| [`#functions`](https://justinjd00.github.io/kosmos/#functions) | the plotter |
| [`#chaos/lorenz`](https://justinjd00.github.io/kosmos/#chaos/lorenz) | a specific system |
| [`#chaos/lorenz+twin`](https://justinjd00.github.io/kosmos/#chaos/lorenz+twin) | with the butterfly twin running |
| [`#chaos/thomas@300`](https://justinjd00.github.io/kosmos/#chaos/thomas@300) | fast-forwarded 300 simulated seconds before the first frame |

The `@` suffix is also what makes the screenshots in this README reproducible: they are taken by
[`tools/shoot.ps1`](tools/shoot.ps1) against a headless browser, not captured by hand.

<br>

## Self-hosting

kosmos is a static site plus one `.wasm` file and one `.js` file. Anything that can serve files
can host it.

<details open>
<summary><b>Docker</b></summary>

<br>

```sh
docker run -p 8080:80 ghcr.io/justinjd00/kosmos
```

or build it yourself:

```sh
git clone https://github.com/justinjd00/kosmos
cd kosmos
docker compose up -d          # http://localhost:8080
```

Change the port with `KOSMOS_PORT=9000 docker compose up -d`.

</details>

<details>
<summary><b>Any web server — nginx, Caddy, S3, a Raspberry Pi …</b></summary>

<br>

```sh
cd web && npm ci && npm run build
```

Copy `web/dist/` wherever you like.

> [!IMPORTANT]
> `.wasm` files must be served as `application/wasm`. nginx and Caddy do this by default;
> `python3 -m http.server` does **not**, and the app will refuse to start.

The build uses relative paths, so it also works from a subdirectory such as
`https://example.com/tools/kosmos/` with no configuration.

A ready-made nginx config sits in [`deploy/nginx.conf`](deploy/nginx.conf). For Caddy:

```
example.com {
    root * /srv/kosmos
    file_server
    try_files {path} /index.html
}
```

</details>

<details>
<summary><b>Building from source</b></summary>

<br>

Requires Rust 1.75+, [wasm-pack](https://rustwasm.github.io/wasm-pack/) and Node 20+.

```sh
cargo install wasm-pack

cd core
wasm-pack build --release --target web --out-dir ../web/src/wasm --out-name kosmos

cd ../web
npm ci && npm run dev
```

The algebra module ships pre-built as `web/public/cas.js`, so none of the above needs OCaml.
Rebuilding it does:

```sh
sudo apt-get install ocaml ocaml-findlib js-of-ocaml
cas/test.sh && cas/build.sh
```

> [!WARNING]
> The dev server hot-reloads TypeScript but **not Rust**. After every change in `core/`, run
> `wasm-pack build` again — otherwise the browser keeps running the old engine. This is the single
> most common source of confusion in this architecture.

</details>

<br>

## Tests

```sh
cd core && cargo test      # the engine, plus both cross-language contracts
cas/test.sh                # the algebra module, natively
node cas/check.mjs         # the shipped algebra bundle
tools/verify.ps1           # the Lean proofs, and the corpus they generate
```

46 tests in the engine, no `unsafe`, six dependencies.

| Area | What is checked |
|---|---|
| **Parser** | precedence, associativity, implicit products, error positions |
| **Evaluator** | arithmetic, unary minus, integer-power optimisation, NaN instead of panics |
| **Calculus** | chain, product and quotient rule, $x^x$ — all against finite differences |
| **Plotting** | poles vs. roots, curvature-driven sampling, undefined regions |
| **Dynamics** | boundedness, energy conservation, orbital periodicity, exponential divergence |
| **Against Lean** | the shipped derivatives must match numbers generated from the proved rules |
| **Against OCaml** | every algebra answer must parse, and every integral must differentiate back |

<br>

## Roadmap

- [x] **Functions** — expression language, symbolic calculus, adaptive plotting
- [x] **Chaos** — eight systems, RK4, butterfly twin, deep links
- [x] **Proofs** — differentiation and simplification machine-checked in Lean 4
- [x] **Algebra** — integration, solving and series in OCaml, compiled to JavaScript
- [ ] **Proofs, part two** — `ln`, `sqrt` and `tan` with their domain conditions
- [ ] **Algebra, part two** — partial fractions, and factoring polynomials over the rationals
- [ ] **Fields** — the wave equation, heat diffusion, electrostatics, solved live
- [ ] **Life** — Turing patterns, predator–prey dynamics, epidemics, cellular automata

<br>

## License

[MIT](LICENSE) — named after Alexander von Humboldt's *Kosmos*, his attempt to describe the whole of
nature in a single work.

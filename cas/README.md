# Algebra

The part of kosmos that rewrites formulas rather than evaluating them, written
in OCaml and compiled to JavaScript with `js_of_ocaml`.

## Why a second language

The Rust engine turns a formula into bytecode and runs it sixty times a second.
That is a job for a language with no allocator surprises and no garbage
collector, and Rust is very good at it.

Symbolic integration is the opposite job. It is a search over rewrite rules —
try the power rule, try a substitution, try parts, back out and try the next
thing — on an immutable tree that is copied constantly and thrown away just as
fast. Every attempt is a pattern match twelve cases deep. In Rust that means
`Box`, `clone`, and lifetimes threaded through a backtracking search; in OCaml
it is the language's home ground:

```ocaml
| Fn (name, [ arg ]) -> (
    match slope_of arg with
    | Some k -> ( match basic name arg with Some anti -> div anti k | None -> give_up ())
    | None -> give_up ())
```

So each language does what it is best at, and they meet at a string.

## What it does

| Entry point | Does |
|---|---|
| `simplify` | normalises and expands: collects like terms, folds constants, multiplies out products of sums |
| `derivative` | symbolic differentiation, the same rules the Rust engine implements |
| `integral` | symbolic antiderivative, or an honest refusal |
| `solve` | real roots of `f(x) = 0`, by closed form up to quadratics, Newton from many seeds above that, and by peeling functions off the outside otherwise |
| `taylor` | the Taylor polynomial of any order up to 12, about any point |

The integrator knows the power rule, linearity, standard forms, linear
substitution, general substitution `u = g(x)` found by dividing by `g'(x)`,
integration by parts, expansion of products of sums, the two quadratic
denominators that give `atan` and `asin`, and the cyclic form
`e^(kx) · sin(mx)`.

**Nothing is returned unchecked.** Every antiderivative is differentiated again
and compared against the original integrand at six points before it is handed
back. A wrong branch of the search therefore produces "no closed form found"
rather than a wrong answer.

## Layout

| File | Contents |
|---|---|
| `src/syntax.ml` | the expression tree, lexer, Pratt parser and printer |
| `src/algebra.ml` | canonical form, differentiation, expansion, numeric evaluation |
| `src/integrate.ml` | the integration rules and the search between them |
| `src/solve.ml` | root finding and isolation |
| `src/series.ml` | Taylor polynomials |
| `src/main.ml` | the `js_of_ocaml` surface: `globalThis.kosmosCas` |

## Building and checking

```sh
sudo apt-get install ocaml ocaml-findlib js-of-ocaml

./test.sh                 # the OCaml suite, run natively
./build.sh                # rebuild web/public/cas.js
node check.mjs            # the shipped bundle against expected.txt
```

`expected.txt` is the contract. `check.mjs` holds the committed bundle to it,
and `core/tests/algebra.rs` holds the *Rust* engine to it as well: every answer
in the table must parse with the Rust parser, every integral must differentiate
back to its integrand under the Rust differentiator, and every series must hug
its function near the centre. Two independent implementations, one table.

## Scope

Real functions of one variable. No complex numbers, no matrices, no assumptions
about a variable's sign, no partial fractions, and no Risch algorithm — an
integral that has no elementary antiderivative and one that this module simply
cannot find are reported the same way, which is worth remembering before
concluding anything from a refusal.

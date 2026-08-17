# Proofs

Machine-checked correctness for the parts of the kosmos engine where being
"probably right" is not good enough.

## What is proved

| Theorem | Statement |
|---|---|
| `hasDerivAt_eval` | For **every** expression and **every** real `x`, the symbolically derived expression is the derivative of the original |
| `simplify_eval` | Simplification never changes the value of an expression |
| `hasDerivAt_simplify_derive` | The combination the engine actually ships — differentiate, then simplify — is still the derivative |
| `differentiable_eval` | Every expression is differentiable everywhere, which is why no side conditions are needed |
| `deriv_eval` | The result agrees with Mathlib's own `deriv` |

Nothing is assumed. `#print axioms` reports only Lean's three standard axioms
(`propext`, `Classical.choice`, `Quot.sound`) — no `sorryAx`, so there are no gaps.
CI fails if that ever changes.

## What is *not* proved

These theorems are about the definitions in `Proofs/Expr.lean`, not about
`core/src/calculus.rs`. Lean cannot see the Rust file.

`Proofs/Bridge.lean` narrows that gap: it evaluates the *verified* derivative at
sample points and writes them to `core/tests/verified-derivatives.txt`. The Rust
test suite differentiates the same expressions with its own rules and must
reproduce those numbers, plus agree with the printed derivative across a further
1,600 points. CI regenerates the file and fails on any difference, so the two
implementations cannot drift apart silently.

That is weaker than extracting verified code, and stronger than a hand-written
test suite. It is stated plainly rather than dressed up.

The scope is also limited on purpose: `+ - * ^ sin cos exp` and every
composition of them. These are differentiable everywhere, so the theorem needs no
hypotheses. Division, `ln`, `sqrt` and `tan` have domain holes and need explicit
side conditions — worth doing, not done yet.

## Running it

```powershell
..\tools\verify.ps1
```

That builds the proofs, checks no axiom sneaked in, regenerates the corpus and
runs the Rust test against it.

Or piecewise:

```sh
lake exe cache get     # prebuilt Mathlib, ~5 GB — do not compile it yourself
lake build
lake env lean --run Proofs/Bridge.lean
```

## Files

| File | Contents |
|---|---|
| `Proofs/Expr.lean` | The expression tree, `eval` and `derive` — the specification |
| `Proofs/Deriv.lean` | The differentiation theorem |
| `Proofs/Simplify.lean` | The simplifier and its soundness theorem |
| `Proofs/Bridge.lean` | Generates the corpus the Rust tests check against |

One design note: constants are rational, not real. Equality on `ℝ` is undecidable,
so a simplifier that asks "is this zero?" could not be written as a program at
all. Every literal the parser accepts is a decimal, hence rational, so nothing is
lost.

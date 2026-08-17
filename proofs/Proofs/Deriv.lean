/-
Copyright (c) 2026 justinjd00. All rights reserved.
Released under the MIT license as described in the file LICENSE.
Authors: justinjd00
-/
import Proofs.Expr

/-!
# Correctness of symbolic differentiation

The engine in `core/src/calculus.rs` differentiates an expression tree by pattern
matching on its shape. This file states that those rules are right — not at a few
sample points, but for every expression and every real number.
-/

namespace Kosmos

/-- **The main theorem.**

For every expression the engine can build, and at every point of the real line,
the symbolically derived expression really is the derivative of the original.

The proof is an induction over the expression tree in which every case is exactly
one Mathlib differentiation rule. That correspondence is the point: it is what
makes this a check of the rules rather than a restatement of them. -/
theorem hasDerivAt_eval (e : Expr) (x : ℝ) :
    HasDerivAt (eval e) (eval (derive e) x) x := by
  induction e with
  | const c =>
      simpa [derive] using hasDerivAt_const x ((c : ℝ))
  | var =>
      simpa [derive] using hasDerivAt_id' (𝕜 := ℝ) x
  | add a b iha ihb =>
      exact iha.add ihb
  | sub a b iha ihb =>
      exact iha.sub ihb
  | mul a b iha ihb =>
      exact iha.mul ihb
  | neg a iha =>
      exact iha.neg
  | pow a n iha =>
      have step : eval (derive (Expr.pow a n)) x
          = (n : ℝ) * eval a x ^ (n - 1) * eval (derive a) x := by
        change ((n : ℚ) : ℝ) * eval a x ^ (n - 1) * eval (derive a) x = _
        push_cast
        ring
      rw [step]
      exact iha.pow n
  | sin a iha =>
      exact iha.sin
  | cos a iha =>
      have step : eval (derive (Expr.cos a)) x
          = -Real.sin (eval a x) * eval (derive a) x := by
        change -(Real.sin (eval a x) * eval (derive a) x) = _
        ring
      rw [step]
      exact iha.cos
  | exp a iha =>
      exact iha.exp

/-- Every expression is differentiable everywhere.

A corollary, but worth stating: it is why the engine may hand back a derivative
without attaching side conditions. -/
theorem differentiable_eval (e : Expr) : Differentiable ℝ (eval e) :=
  fun x => (hasDerivAt_eval e x).differentiableAt

/-- The result agrees with Mathlib's own `deriv`, so the engine's answer can be
compared against the standard definition rather than only against itself. -/
theorem deriv_eval (e : Expr) (x : ℝ) : deriv (eval e) x = eval (derive e) x :=
  (hasDerivAt_eval e x).deriv

/-- Differentiating `n` times gives the `n`-th derivative, for every `n`. -/
theorem hasDerivAt_eval_iterate (e : Expr) (n : ℕ) (x : ℝ) :
    HasDerivAt (eval (derive^[n] e)) (eval (derive^[n + 1] e) x) x := by
  rw [Function.iterate_succ_apply']
  exact hasDerivAt_eval _ x

end Kosmos

/-
Copyright (c) 2026 justinjd00. All rights reserved.
Released under the MIT license as described in the file LICENSE.
Authors: justinjd00
-/
import Mathlib.Analysis.SpecialFunctions.Trigonometric.Deriv
import Mathlib.Analysis.SpecialFunctions.ExpDeriv

/-!
# The expression language

A Lean mirror of the expression tree, evaluator and symbolic differentiator that
the kosmos engine implements in Rust. The definitions here are the specification;
`Deriv.lean` proves the differentiation rules correct against Mathlib.
-/

namespace Kosmos

/-- The expression tree of the kosmos engine, mirrored from `core/src/expr.rs`.

Constants are rational rather than real on purpose: equality on `ℝ` is not
decidable, so a simplifier that pattern matches on "is this zero" could not be
written as a program at all. Every literal the parser can produce is a decimal,
hence rational, so nothing is lost. -/
inductive Expr where
  | const : ℚ → Expr
  | var : Expr
  | add : Expr → Expr → Expr
  | sub : Expr → Expr → Expr
  | mul : Expr → Expr → Expr
  | neg : Expr → Expr
  | pow : Expr → ℕ → Expr
  | sin : Expr → Expr
  | cos : Expr → Expr
  | exp : Expr → Expr
  deriving DecidableEq, Repr, Inhabited

/-- What an expression means: a function from ℝ to ℝ. -/
noncomputable def eval : Expr → (ℝ → ℝ)
  | .const c => fun _ => (c : ℝ)
  | .var => fun x => x
  | .add a b => fun x => eval a x + eval b x
  | .sub a b => fun x => eval a x - eval b x
  | .mul a b => fun x => eval a x * eval b x
  | .neg a => fun x => -eval a x
  | .pow a n => fun x => eval a x ^ n
  | .sin a => fun x => Real.sin (eval a x)
  | .cos a => fun x => Real.cos (eval a x)
  | .exp a => fun x => Real.exp (eval a x)

/-- Symbolic differentiation, rule for rule the same as `calculus.rs`. -/
def derive : Expr → Expr
  | .const _ => .const 0
  | .var => .const 1
  | .add a b => .add (derive a) (derive b)
  | .sub a b => .sub (derive a) (derive b)
  | .mul a b => .add (.mul (derive a) b) (.mul a (derive b))
  | .neg a => .neg (derive a)
  | .pow a n => .mul (.mul (.const (n : ℚ)) (.pow a (n - 1))) (derive a)
  | .sin a => .mul (.cos a) (derive a)
  | .cos a => .neg (.mul (.sin a) (derive a))
  | .exp a => .mul (.exp a) (derive a)

@[simp] theorem eval_const (c : ℚ) : eval (.const c) = fun _ => (c : ℝ) := rfl
@[simp] theorem eval_var : eval .var = fun x : ℝ => x := rfl
@[simp] theorem eval_add (a b : Expr) : eval (.add a b) = fun x => eval a x + eval b x := rfl
@[simp] theorem eval_sub (a b : Expr) : eval (.sub a b) = fun x => eval a x - eval b x := rfl
@[simp] theorem eval_mul (a b : Expr) : eval (.mul a b) = fun x => eval a x * eval b x := rfl
@[simp] theorem eval_neg (a : Expr) : eval (.neg a) = fun x => -eval a x := rfl
@[simp] theorem eval_pow (a : Expr) (n : ℕ) : eval (.pow a n) = fun x => eval a x ^ n := rfl
@[simp] theorem eval_sin (a : Expr) : eval (.sin a) = fun x => Real.sin (eval a x) := rfl
@[simp] theorem eval_cos (a : Expr) : eval (.cos a) = fun x => Real.cos (eval a x) := rfl
@[simp] theorem eval_exp (a : Expr) : eval (.exp a) = fun x => Real.exp (eval a x) := rfl

end Kosmos

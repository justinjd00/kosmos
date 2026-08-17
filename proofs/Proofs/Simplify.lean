/-
Copyright (c) 2026 justinjd00. All rights reserved.
Released under the MIT license as described in the file LICENSE.
Authors: justinjd00
-/
import Proofs.Deriv

/-!
# Correctness of simplification

The engine simplifies after every differentiation. Rewrite rules are the part of a
computer algebra system most likely to be quietly wrong, because a bad rule still
produces a well-formed expression — it just no longer means the same thing. This
file proves that every rule preserves the value.
-/

namespace Kosmos

/-- Smart constructors, mirroring the rewrite rules in `simplify` in `calculus.rs`.
Each one folds constants and drops neutral elements. -/
def addSmart : Expr → Expr → Expr
  | .const p, .const q => .const (p + q)
  | .const 0, b => b
  | a, .const 0 => a
  | a, b => .add a b

def subSmart : Expr → Expr → Expr
  | .const p, .const q => .const (p - q)
  | a, .const 0 => a
  | a, b => .sub a b

def negSmart : Expr → Expr
  | .const c => .const (-c)
  | .neg a => a
  | a => .neg a

def mulSmart : Expr → Expr → Expr
  | .const p, .const q => .const (p * q)
  | .const 0, _ => .const 0
  | _, .const 0 => .const 0
  | .const 1, b => b
  | a, .const 1 => a
  | a, b => .mul a b

def powSmart : Expr → ℕ → Expr
  | _, 0 => .const 1
  | a, 1 => a
  | a, n => .pow a n

/-- The simplifier: rebuild the tree bottom-up through the smart constructors. -/
def simplify : Expr → Expr
  | .const c => .const c
  | .var => .var
  | .add a b => addSmart (simplify a) (simplify b)
  | .sub a b => subSmart (simplify a) (simplify b)
  | .mul a b => mulSmart (simplify a) (simplify b)
  | .neg a => negSmart (simplify a)
  | .pow a n => powSmart (simplify a) n
  | .sin a => .sin (simplify a)
  | .cos a => .cos (simplify a)
  | .exp a => .exp (simplify a)

@[simp] theorem addSmart_eval (a b : Expr) (x : ℝ) :
    eval (addSmart a b) x = eval a x + eval b x := by
  unfold addSmart
  split <;> simp

@[simp] theorem subSmart_eval (a b : Expr) (x : ℝ) :
    eval (subSmart a b) x = eval a x - eval b x := by
  unfold subSmart
  split <;> simp

@[simp] theorem negSmart_eval (a : Expr) (x : ℝ) :
    eval (negSmart a) x = -eval a x := by
  unfold negSmart
  split <;> simp

@[simp] theorem mulSmart_eval (a b : Expr) (x : ℝ) :
    eval (mulSmart a b) x = eval a x * eval b x := by
  unfold mulSmart
  split <;> simp

@[simp] theorem powSmart_eval (a : Expr) (n : ℕ) (x : ℝ) :
    eval (powSmart a n) x = eval a x ^ n := by
  unfold powSmart
  split <;> simp

/-- **Simplification never changes what an expression means.**

The engine simplifies after every differentiation — without this, a derivative
could be structurally smaller and numerically wrong, which is exactly the kind of
bug that survives a test suite. -/
theorem simplify_eval (e : Expr) (x : ℝ) : eval (simplify e) x = eval e x := by
  induction e with
  | const c => rfl
  | var => rfl
  | add a b iha ihb => simp [simplify, iha, ihb]
  | sub a b iha ihb => simp [simplify, iha, ihb]
  | mul a b iha ihb => simp [simplify, iha, ihb]
  | neg a iha => simp [simplify, iha]
  | pow a n iha => simp [simplify, iha]
  | sin a iha => simp [simplify, iha]
  | cos a iha => simp [simplify, iha]
  | exp a iha => simp [simplify, iha]

/-- Simplifying a derivative is still the derivative — the combination the engine
actually ships. -/
theorem hasDerivAt_simplify_derive (e : Expr) (x : ℝ) :
    HasDerivAt (eval e) (eval (simplify (derive e)) x) x := by
  rw [simplify_eval]
  exact hasDerivAt_eval e x

end Kosmos

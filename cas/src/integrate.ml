open Syntax
open Algebra

exception Give_up

let give_up () = raise Give_up
let marker = Fn ("#u", [])
let ln_abs a = Fn ("ln", [ Fn ("abs", [ a ]) ])

let slope_of e =
  let d = derive e in
  if depends d then None
  else
    let k = value_at 0.0 d in
    if Float.abs k < 1e-12 then None else Some (Num k)

let basic name arg =
  let sq a = power a (Num 2.0) in
  match name with
  | "sin" -> Some (negate (Fn ("cos", [ arg ])))
  | "cos" -> Some (Fn ("sin", [ arg ]))
  | "tan" -> Some (negate (ln_abs (Fn ("cos", [ arg ]))))
  | "exp" -> Some (Fn ("exp", [ arg ]))
  | "sinh" -> Some (Fn ("cosh", [ arg ]))
  | "cosh" -> Some (Fn ("sinh", [ arg ]))
  | "tanh" -> Some (Fn ("ln", [ Fn ("cosh", [ arg ]) ]))
  | "ln" -> Some (sub (mul arg (Fn ("ln", [ arg ]))) arg)
  | "log10" -> Some (div (sub (mul arg (Fn ("ln", [ arg ]))) arg) (Num (Float.log 10.0)))
  | "log2" -> Some (div (sub (mul arg (Fn ("ln", [ arg ]))) arg) (Num (Float.log 2.0)))
  | "asin" -> Some (add (mul arg (Fn ("asin", [ arg ]))) (power (sub one (sq arg)) (Num 0.5)))
  | "acos" -> Some (sub (mul arg (Fn ("acos", [ arg ]))) (power (sub one (sq arg)) (Num 0.5)))
  | "atan" ->
      Some (sub (mul arg (Fn ("atan", [ arg ]))) (mul (Num 0.5) (Fn ("ln", [ add one (sq arg) ]))))
  | _ -> None

let power_rule base exponent =
  match exponent with
  | Num e when Float.abs (e +. 1.0) < 1e-12 -> ln_abs base
  | Num e -> div (power base (Num (e +. 1.0))) (Num (e +. 1.0))
  | _ -> give_up ()

let quadratic_shape e =
  let square = ref 0.0 and constant = ref 0.0 and clean = ref true in
  let take part =
    match part with
    | Num v -> constant := !constant +. v
    | Pow (Var, Num 2.0) -> square := !square +. 1.0
    | Mul [ Num k; Pow (Var, Num 2.0) ] -> square := !square +. k
    | _ -> clean := false
  in
  (match norm e with Add parts -> List.iter take parts | single -> take single);
  if !clean && !square <> 0.0 then Some (!square, !constant) else None

let rec replace target body =
  if equal body target then marker
  else
    match body with
    | Num _ | Var -> body
    | Add xs -> Add (List.map (replace target) xs)
    | Mul xs -> Mul (List.map (replace target) xs)
    | Pow (a, b) -> Pow (replace target a, replace target b)
    | Fn (name, xs) -> Fn (name, List.map (replace target) xs)

let rec unmark body =
  if equal body marker then Var
  else
    match body with
    | Num _ | Var -> body
    | Add xs -> Add (List.map unmark xs)
    | Mul xs -> Mul (List.map unmark xs)
    | Pow (a, b) -> Pow (unmark a, unmark b)
    | Fn (name, xs) -> Fn (name, List.map unmark xs)

let rec subterms e =
  let self = if depends e && not (equal e Var) then [ e ] else [] in
  match e with
  | Num _ | Var -> []
  | Add xs | Mul xs | Fn (_, xs) -> self @ List.concat_map subterms xs
  | Pow (a, b) -> self @ subterms a @ subterms b

let children e =
  match e with
  | Add xs | Mul xs | Fn (_, xs) -> List.concat_map subterms xs
  | Pow (a, b) -> subterms a @ subterms b
  | _ -> []

let proportional e =
  if depends e && is_num 0.0 (substitute zero e) then
    let d = derive e in
    if depends d then None else Some (value_at 0.0 d)
  else None

let rec run depth e =
  if depth > 7 then give_up ();
  let e = norm e in
  if not (depends e) then mul e Var
  else
    match e with
    | Var -> div (power Var (Num 2.0)) (Num 2.0)
    | Add terms -> norm_add (List.map (run (depth + 1)) terms)
    | Mul factors -> product depth factors
    | Pow (Var, exponent) when not (depends exponent) -> power_rule Var exponent
    | Pow (b, exponent) when not (depends exponent) -> (
        match slope_of b with
        | Some k -> div (power_rule b exponent) k
        | None -> quadratic_rules e)
    | Pow (b, exponent) when not (depends b) -> (
        match slope_of exponent with
        | Some k -> div e (mul k (Fn ("ln", [ b ])))
        | None -> give_up ())
    | Fn (name, [ arg ]) -> (
        match slope_of arg with
        | Some k -> ( match basic name arg with Some anti -> div anti k | None -> give_up ())
        | None -> give_up ())
    | _ -> quadratic_rules e

and quadratic_rules e =
  match e with
  | Pow (denominator, Num p) when Float.abs (p +. 1.0) < 1e-12 -> (
      match quadratic_shape denominator with
      | Some (q, c) when q > 0.0 && c > 0.0 ->
          let a = Float.sqrt (c /. q) in
          div (Fn ("atan", [ div Var (Num a) ])) (Num (q *. a))
      | _ -> give_up ())
  | Pow (radicand, Num p) when Float.abs (p +. 0.5) < 1e-12 -> (
      match quadratic_shape radicand with
      | Some (q, c) when q < 0.0 && c > 0.0 ->
          div (Fn ("asin", [ div Var (Num (Float.sqrt (c /. -.q))) ])) (Num (Float.sqrt (-.q)))
      | Some (q, c) when q > 0.0 ->
          let shifted = add (power Var (Num 2.0)) (Num (c /. q)) in
          div (ln_abs (add Var (power shifted (Num 0.5)))) (Num (Float.sqrt q))
      | _ -> give_up ())
  | _ -> give_up ()

and product depth factors =
  let constants, rest = List.partition (fun f -> not (depends f)) factors in
  let coefficient = norm_mul constants in
  match rest with
  | [] -> mul coefficient Var
  | [ single ] -> mul coefficient (run (depth + 1) single)
  | _ ->
      mul coefficient
        (attempt
           [ (fun () -> exponential_wave rest);
             (fun () -> by_substitution depth rest);
             (fun () -> by_parts depth rest);
             (fun () -> run (depth + 1) (opened (norm_mul rest))) ])

and exponential_wave factors =
  let pair =
    match factors with
    | [ Fn ("exp", [ a ]); Fn (("sin" | "cos") as name, [ b ]) ] -> Some (a, name, b)
    | [ Fn (("sin" | "cos") as name, [ b ]); Fn ("exp", [ a ]) ] -> Some (a, name, b)
    | _ -> None
  in
  match pair with
  | None -> give_up ()
  | Some (a, name, b) -> (
      match (proportional a, proportional b) with
      | Some k, Some m when (k *. k) +. (m *. m) > 0.0 ->
          let scale = Num (1.0 /. ((k *. k) +. (m *. m))) in
          let wave =
            if name = "sin" then
              sub (mul (Num k) (Fn ("sin", [ b ]))) (mul (Num m) (Fn ("cos", [ b ])))
            else add (mul (Num k) (Fn ("cos", [ b ]))) (mul (Num m) (Fn ("sin", [ b ])))
          in
          mul scale (mul (Fn ("exp", [ a ])) wave)
      | _ -> give_up ())

and attempt = function
  | [] -> give_up ()
  | first :: rest -> ( try first () with Give_up | Error _ | Stack_overflow -> attempt rest)

and opened e =
  let before = norm e in
  let after = expand before in
  if equal after before then give_up () else after

and by_substitution depth factors =
  let body = norm_mul factors in
  let candidates = List.sort_uniq compare_expr (children body) in
  let try_one inner =
    let d = derive inner in
    if is_num 0.0 d then give_up ();
    let quotient = norm (div body d) in
    let marked = replace inner quotient in
    if depends marked then give_up ();
    let pattern = norm (unmark marked) in
    if not (depends pattern) then give_up ();
    let anti = run (depth + 1) pattern in
    substitute inner anti
  in
  attempt (List.map (fun c () -> try_one c) candidates)

and by_parts depth factors =
  let polynomial f =
    match f with
    | Var -> true
    | Pow (Var, Num n) when is_int n && n > 0.0 -> true
    | _ -> false
  in
  let logarithm f = match f with Fn ("ln", _) | Fn ("log2", _) | Fn ("log10", _) -> true | _ -> false in
  let parts u rest =
    let v = run (depth + 1) (norm_mul rest) in
    sub (mul u v) (run (depth + 1) (norm_mul [ derive u; v ]))
  in
  match List.partition logarithm factors with
  | [ u ], rest when rest <> [] -> parts u rest
  | _ -> (
      match List.partition polynomial factors with
      | [ u ], rest when rest <> [] -> parts u rest
      | _ -> give_up ())

let probes = [ -1.73; -0.61; 0.29; 0.87; 1.41; 2.63 ]

let confirms candidate target =
  try
    let back = norm (derive candidate) in
    let agree x =
      let a = value_at x back and b = value_at x target in
      if Float.is_nan a || Float.is_nan b || Float.abs a = Float.infinity then true
      else Float.abs (a -. b) <= 1e-6 *. (1.0 +. Float.abs b)
    in
    let checked = List.filter (fun x -> not (Float.is_nan (value_at x target))) probes in
    List.length checked >= 2 && List.for_all agree probes
  with _ -> false

let antiderivative e =
  let target = norm e in
  let result = norm (run 0 target) in
  if confirms result target then result else give_up ()

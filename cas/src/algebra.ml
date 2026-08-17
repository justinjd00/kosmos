open Syntax

let rec compare_expr a b =
  let ra = rank a and rb = rank b in
  if ra <> rb then compare ra rb
  else
    match (a, b) with
    | Num x, Num y -> compare x y
    | Num _, _ -> -1
    | _, Num _ -> 1
    | Var, Var -> 0
    | Var, _ -> -1
    | _, Var -> 1
    | Fn (n, xs), Fn (m, ys) ->
        let c = compare n m in
        if c <> 0 then c else compare_list xs ys
    | Fn _, _ -> -1
    | _, Fn _ -> 1
    | Pow (b1, e1), Pow (b2, e2) ->
        let c = compare_expr b1 b2 in
        if c <> 0 then c else compare_expr e1 e2
    | Mul xs, Mul ys -> compare_list xs ys
    | Add xs, Add ys -> compare_list xs ys
    | _ -> compare (rank a) (rank b)

and compare_list xs ys =
  match (xs, ys) with
  | [], [] -> 0
  | [], _ -> -1
  | _, [] -> 1
  | x :: xt, y :: yt ->
      let c = compare_expr x y in
      if c <> 0 then c else compare_list xt yt

let equal a b = compare_expr a b = 0

let rec flatten_add = function
  | [] -> []
  | Add inner :: rest -> flatten_add (inner @ rest)
  | e :: rest -> e :: flatten_add rest

let rec flatten_mul = function
  | [] -> []
  | Mul inner :: rest -> flatten_mul (inner @ rest)
  | e :: rest -> e :: flatten_mul rest

let split_coefficient = function
  | Mul factors ->
      let nums, rest =
        List.partition (function Num _ -> true | _ -> false) factors
      in
      let c = List.fold_left (fun acc f -> match f with Num v -> acc *. v | _ -> acc) 1.0 nums in
      (c, match rest with [] -> one | [ single ] -> single | many -> Mul many)
  | Num v -> (v, one)
  | other -> (1.0, other)

let split_power = function
  | Pow (b, e) -> (b, e)
  | other -> (other, one)

let pow_num base e =
  if e = 0.0 then Some 1.0
  else if base = 0.0 then if e > 0.0 then Some 0.0 else None
  else if base < 0.0 && not (is_int e) then None
  else
    let v = Float.pow base e in
    if Float.is_nan v || Float.abs v = Float.infinity then None else Some v

let apply_fn name args =
  let f1 g = match args with [ Num v ] -> Some (g v) | _ -> None in
  let value =
    match name with
    | "sin" -> f1 Float.sin
    | "cos" -> f1 Float.cos
    | "tan" -> f1 Float.tan
    | "asin" -> f1 Float.asin
    | "acos" -> f1 Float.acos
    | "atan" -> f1 Float.atan
    | "sinh" -> f1 Float.sinh
    | "cosh" -> f1 Float.cosh
    | "tanh" -> f1 Float.tanh
    | "exp" -> f1 Float.exp
    | "ln" -> f1 (fun v -> if v > 0.0 then Float.log v else Float.nan)
    | "log2" -> f1 (fun v -> if v > 0.0 then Float.log v /. Float.log 2.0 else Float.nan)
    | "log10" -> f1 (fun v -> if v > 0.0 then Float.log10 v else Float.nan)
    | "sqrt" -> f1 (fun v -> if v >= 0.0 then Float.sqrt v else Float.nan)
    | "cbrt" -> f1 Float.cbrt
    | "abs" -> f1 Float.abs
    | "sign" -> f1 (fun v -> if v > 0.0 then 1.0 else if v < 0.0 then -1.0 else 0.0)
    | "floor" -> f1 Float.floor
    | "ceil" -> f1 Float.ceil
    | "round" -> f1 Float.round
    | "min" -> (match args with [ Num a; Num b ] -> Some (Float.min a b) | _ -> None)
    | "max" -> (match args with [ Num a; Num b ] -> Some (Float.max a b) | _ -> None)
    | "atan2" -> (match args with [ Num a; Num b ] -> Some (Float.atan2 a b) | _ -> None)
    | "hypot" -> (match args with [ Num a; Num b ] -> Some (Float.hypot a b) | _ -> None)
    | _ -> None
  in
  match value with
  | Some v when Float.is_nan v -> None
  | Some v when Float.abs v > 1e300 -> None
  | Some v -> Some (Num v)
  | None -> None

let rec norm e =
  match e with
  | Num _ | Var -> e
  | Fn ("pow", [ a; b ]) -> norm (Pow (a, b))
  | Fn ("log", [ a; b ]) -> norm (Mul [ Fn ("ln", [ b ]); Pow (Fn ("ln", [ a ]), neg_one) ])
  | Fn ("sqrt", [ a ]) -> norm (Pow (a, Num 0.5))
  | Fn ("cbrt", [ a ]) -> norm (Pow (a, Num (1.0 /. 3.0)))
  | Fn (name, args) -> (
      let args = List.map norm args in
      match apply_fn name args with
      | Some v -> v
      | None -> (
          match (name, args) with
          | "exp", [ Fn ("ln", [ inner ]) ] -> inner
          | "ln", [ Fn ("exp", [ inner ]) ] -> inner
          | "abs", [ Pow (b, Num e) ] when is_int e && Float.rem e 2.0 = 0.0 -> Pow (b, Num e)
          | _ -> Fn (name, args)))
  | Pow (b, e) -> norm_pow (norm b) (norm e)
  | Mul factors -> norm_mul (List.map norm (flatten_mul factors))
  | Add terms -> norm_add (List.map norm (flatten_add terms))

and norm_pow b e =
  match (b, e) with
  | _, Num v when v = 0.0 -> one
  | _, Num v when v = 1.0 -> b
  | Num v, _ when v = 1.0 -> one
  | Num v, Num w -> ( match pow_num v w with Some r -> Num r | None -> Pow (b, e))
  | Pow (inner, ie), _ when is_int_pow e -> norm_pow inner (norm_mul [ ie; e ])
  | Mul factors, _ when is_int_pow e -> norm_mul (List.map (fun f -> norm_pow f e) factors)
  | Fn ("exp", [ a ]), _ -> Fn ("exp", [ norm_mul [ a; e ] ])
  | _ -> Pow (b, e)

and is_int_pow = function Num v -> is_int v | _ -> false

and norm_mul factors =
  let factors = flatten_mul factors in
  if List.exists (is_num 0.0) factors then zero
  else begin
    let coefficient = ref 1.0 in
    let table : (t * float ref) list ref = ref [] in
    let add_power base exponent =
      match List.find_opt (fun (b, _) -> equal b base) !table with
      | Some (_, slot) -> slot := !slot +. exponent
      | None -> table := (base, ref exponent) :: !table
    in
    List.iter
      (fun f ->
        match f with
        | Num v -> coefficient := !coefficient *. v
        | _ -> (
            let b, e = split_power f in
            match e with
            | Num v -> add_power b v
            | _ -> add_power f 1.0))
      factors;
    let rebuilt =
      List.filter_map
        (fun (base, slot) ->
          let e = !slot in
          if Float.abs e < 1e-12 then None
          else
            match (base, e) with
            | Num v, _ -> (
                match pow_num v e with
                | Some r ->
                    coefficient := !coefficient *. r;
                    None
                | None -> Some (Pow (base, Num e)))
            | _, 1.0 -> Some base
            | _ -> Some (norm_pow base (Num e)))
        (List.rev !table)
    in
    let rebuilt = List.sort compare_expr rebuilt in
    if !coefficient = 0.0 then zero
    else
      match rebuilt with
      | [] -> Num !coefficient
      | [ single ] when !coefficient = 1.0 -> single
      | _ ->
          let all = if !coefficient = 1.0 then rebuilt else Num !coefficient :: rebuilt in
          distribute all
  end

and distribute factors =
  match List.partition (function Add _ -> true | _ -> false) factors with
  | [ Add terms ], others when List.length terms <= 8 && others <> [] ->
      norm_add (List.map (fun term -> norm_mul (term :: others)) terms)
  | _ -> ( match factors with [ single ] -> single | _ -> Mul factors)

and norm_add terms =
  let terms = flatten_add terms in
  let constant = ref 0.0 in
  let table : (t * float ref) list ref = ref [] in
  List.iter
    (fun term ->
      let c, body = split_coefficient term in
      if body = one then constant := !constant +. c
      else
        match List.find_opt (fun (b, _) -> equal b body) !table with
        | Some (_, slot) -> slot := !slot +. c
        | None -> table := (body, ref c) :: !table)
    terms;
  let rebuilt =
    List.filter_map
      (fun (body, slot) ->
        let c = !slot in
        if Float.abs c < 1e-12 then None
        else if c = 1.0 then Some body
        else Some (norm_mul [ Num c; body ]))
      (List.rev !table)
  in
  let rebuilt = List.sort compare_expr rebuilt in
  let all = if !constant = 0.0 then rebuilt else rebuilt @ [ Num !constant ] in
  match all with [] -> zero | [ single ] -> single | _ -> Add all

let rec size = function
  | Num _ | Var -> 1
  | Add xs | Mul xs | Fn (_, xs) -> List.fold_left (fun acc e -> acc + size e) 1 xs
  | Pow (a, b) -> 1 + size a + size b

let terms_of = function Add xs -> xs | single -> [ single ]

let multiply_out a b =
  norm_add
    (List.concat_map (fun p -> List.map (fun q -> norm_mul [ p; q ]) (terms_of b)) (terms_of a))

let rec expand e =
  let e = norm e in
  if size e > 400 then e
  else
    match e with
    | Add xs -> norm_add (List.map expand xs)
    | Mul xs -> List.fold_left (fun acc f -> multiply_out acc (expand f)) one xs
    | Pow (b, Num n) when is_int n && n >= 2.0 && n <= 8.0 -> (
        match expand b with
        | Add _ as based ->
            let rec repeat k acc = if k <= 0 then acc else repeat (k - 1) (multiply_out acc based) in
            repeat (int_of_float n - 1) based
        | other -> norm_pow other (Num n))
    | Pow (a, b) -> norm_pow (expand a) (expand b)
    | Fn (name, xs) -> norm (Fn (name, List.map expand xs))
    | other -> other

let add a b = norm_add [ a; b ]
let sub a b = norm_add [ a; norm_mul [ neg_one; b ] ]
let mul a b = norm_mul [ a; b ]
let div a b = norm_mul [ a; norm_pow b neg_one ]
let power a b = norm_pow a b
let negate a = norm_mul [ neg_one; a ]

let rec depends = function
  | Var -> true
  | Num _ -> false
  | Add xs | Mul xs | Fn (_, xs) -> List.exists depends xs
  | Pow (a, b) -> depends a || depends b

let rec derive e =
  match e with
  | Num _ -> zero
  | Var -> one
  | Add terms -> norm_add (List.map derive terms)
  | Mul factors ->
      let rec each before = function
        | [] -> []
        | f :: after ->
            norm_mul (derive f :: (List.rev_append before after))
            :: each (f :: before) after
      in
      norm_add (each [] factors)
  | Pow (b, e') when not (depends e') -> mul (mul e' (power b (sub e' one))) (derive b)
  | Pow (b, e') -> mul e (add (mul (derive e') (Fn ("ln", [ b ]))) (div (mul e' (derive b)) b))
  | Fn (name, [ a ]) -> mul (derive_fn name a) (derive a)
  | Fn ("atan2", [ a; b ]) ->
      let denom = add (power a (Num 2.0)) (power b (Num 2.0)) in
      div (sub (mul (derive a) b) (mul a (derive b))) denom
  | Fn ("hypot", [ a; b ]) ->
      let h = Fn ("hypot", [ a; b ]) in
      div (add (mul a (derive a)) (mul b (derive b))) h
  | Fn (name, _) -> fail (Printf.sprintf "cannot differentiate %s here" name)

and derive_fn name a =
  match name with
  | "sin" -> Fn ("cos", [ a ])
  | "cos" -> negate (Fn ("sin", [ a ]))
  | "tan" -> add one (power (Fn ("tan", [ a ])) (Num 2.0))
  | "asin" -> power (sub one (power a (Num 2.0))) (Num (-0.5))
  | "acos" -> negate (power (sub one (power a (Num 2.0))) (Num (-0.5)))
  | "atan" -> div one (add one (power a (Num 2.0)))
  | "sinh" -> Fn ("cosh", [ a ])
  | "cosh" -> Fn ("sinh", [ a ])
  | "tanh" -> sub one (power (Fn ("tanh", [ a ])) (Num 2.0))
  | "exp" -> Fn ("exp", [ a ])
  | "ln" -> div one a
  | "log2" -> div one (mul a (Num (Float.log 2.0)))
  | "log10" -> div one (mul a (Num (Float.log 10.0)))
  | "abs" -> Fn ("sign", [ a ])
  | "sign" | "floor" | "ceil" | "round" -> zero
  | _ -> fail (Printf.sprintf "cannot differentiate %s" name)

let rec substitute target = function
  | Var -> target
  | Num v -> Num v
  | Add xs -> norm_add (List.map (substitute target) xs)
  | Mul xs -> norm_mul (List.map (substitute target) xs)
  | Pow (a, b) -> norm_pow (substitute target a) (substitute target b)
  | Fn (name, xs) -> norm (Fn (name, List.map (substitute target) xs))

let rec value_at x = function
  | Num v -> v
  | Var -> x
  | Add xs -> List.fold_left (fun acc e -> acc +. value_at x e) 0.0 xs
  | Mul xs -> List.fold_left (fun acc e -> acc *. value_at x e) 1.0 xs
  | Pow (a, b) -> Float.pow (value_at x a) (value_at x b)
  | Fn (name, args) -> (
      let v = List.map (value_at x) args in
      match (name, v) with
      | "sin", [ a ] -> Float.sin a
      | "cos", [ a ] -> Float.cos a
      | "tan", [ a ] -> Float.tan a
      | "asin", [ a ] -> Float.asin a
      | "acos", [ a ] -> Float.acos a
      | "atan", [ a ] -> Float.atan a
      | "sinh", [ a ] -> Float.sinh a
      | "cosh", [ a ] -> Float.cosh a
      | "tanh", [ a ] -> Float.tanh a
      | "exp", [ a ] -> Float.exp a
      | "ln", [ a ] -> Float.log a
      | "log2", [ a ] -> Float.log a /. Float.log 2.0
      | "log10", [ a ] -> Float.log10 a
      | "sqrt", [ a ] -> Float.sqrt a
      | "cbrt", [ a ] -> Float.cbrt a
      | "abs", [ a ] -> Float.abs a
      | "sign", [ a ] -> if a > 0.0 then 1.0 else if a < 0.0 then -1.0 else 0.0
      | "floor", [ a ] -> Float.floor a
      | "ceil", [ a ] -> Float.ceil a
      | "round", [ a ] -> Float.round a
      | "min", [ a; b ] -> Float.min a b
      | "max", [ a; b ] -> Float.max a b
      | "atan2", [ a; b ] -> Float.atan2 a b
      | "hypot", [ a; b ] -> Float.hypot a b
      | _ -> Float.nan)

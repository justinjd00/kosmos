open Syntax
open Algebra

exception Unsolved

let unsolved () = raise Unsolved

let coefficients e =
  let table = Hashtbl.create 8 in
  let bump k v =
    if not (is_int k) || k < 0.0 || k > 8.0 then unsolved ();
    let key = int_of_float k in
    Hashtbl.replace table key (v +. try Hashtbl.find table key with Not_found -> 0.0)
  in
  let take term =
    let c, body = split_coefficient term in
    match body with
    | Num v -> bump 0.0 (c *. v)
    | Var -> bump 1.0 c
    | Pow (Var, Num k) -> bump k c
    | _ -> unsolved ()
  in
  (match norm e with
  | Add terms -> List.iter take terms
  | Num v -> bump 0.0 v
  | single -> take single);
  let degree = Hashtbl.fold (fun k _ acc -> max k acc) table 0 in
  Array.init (degree + 1) (fun i -> try Hashtbl.find table i with Not_found -> 0.0)

let tidy v =
  let rounded = Float.round v in
  if Float.abs (v -. rounded) < 1e-11 then rounded else v

let polynomial_roots coefficients =
  let n = Array.length coefficients - 1 in
  match n with
  | 0 -> if coefficients.(0) = 0.0 then unsolved () else []
  | 1 -> [ -.coefficients.(0) /. coefficients.(1) ]
  | 2 ->
      let a = coefficients.(2) and b = coefficients.(1) and c = coefficients.(0) in
      let disc = (b *. b) -. (4.0 *. a *. c) in
      if disc < -1e-12 then []
      else if Float.abs disc < 1e-12 then [ -.b /. (2.0 *. a) ]
      else
        let r = Float.sqrt disc in
        List.sort compare [ (-.b -. r) /. (2.0 *. a); (-.b +. r) /. (2.0 *. a) ]
  | _ ->
      let value x = Array.to_list coefficients |> List.mapi (fun i c -> c *. Float.pow x (float_of_int i)) |> List.fold_left ( +. ) 0.0 in
      let slope x =
        Array.to_list coefficients
        |> List.mapi (fun i c -> if i = 0 then 0.0 else float_of_int i *. c *. Float.pow x (float_of_int (i - 1)))
        |> List.fold_left ( +. ) 0.0
      in
      let found = ref [] in
      let known x = List.exists (fun y -> Float.abs (x -. y) < 1e-7) !found in
      for step = -240 to 240 do
        let seed = float_of_int step *. 0.05 in
        let x = ref seed in
        let alive = ref true in
        for _ = 1 to 60 do
          if !alive then begin
            let d = slope !x in
            if Float.abs d < 1e-14 then alive := false else x := !x -. (value !x /. d)
          end
        done;
        if !alive && Float.abs !x < 1e7 && Float.abs (value !x) < 1e-9 && not (known !x) then
          found := tidy !x :: !found
      done;
      List.sort compare !found

let rec shift_into body target =
  match norm body with
  | Var -> [ target ]
  | Fn (name, [ arg ]) -> (
      match invert name target with Some next -> shift_into arg next | None -> unsolved ())
  | Pow (b, Num k) when depends b && not (depends target) ->
      let root = power target (Num (1.0 /. k)) in
      shift_into b root
  | Add terms -> (
      let constants, rest = List.partition (fun t -> not (depends t)) terms in
      match (constants, rest) with
      | [], _ -> unsolved ()
      | _, [ single ] -> shift_into single (sub target (norm_add constants))
      | _ -> unsolved ())
  | Mul factors -> (
      let constants, rest = List.partition (fun f -> not (depends f)) factors in
      match (constants, rest) with
      | [], _ -> unsolved ()
      | _, [ single ] -> shift_into single (div target (norm_mul constants))
      | _ -> unsolved ())
  | _ -> unsolved ()

and invert name target =
  match name with
  | "exp" -> Some (Fn ("ln", [ target ]))
  | "ln" -> Some (Fn ("exp", [ target ]))
  | "log10" -> Some (power (Num 10.0) target)
  | "log2" -> Some (power (Num 2.0) target)
  | "sin" -> Some (Fn ("asin", [ target ]))
  | "cos" -> Some (Fn ("acos", [ target ]))
  | "tan" -> Some (Fn ("atan", [ target ]))
  | "asin" -> Some (Fn ("sin", [ target ]))
  | "acos" -> Some (Fn ("cos", [ target ]))
  | "atan" -> Some (Fn ("tan", [ target ]))
  | "sinh" -> Some (Fn ("ln", [ add target (power (add (power target (Num 2.0)) one) (Num 0.5)) ]))
  | "cosh" -> Some (Fn ("ln", [ add target (power (sub (power target (Num 2.0)) one) (Num 0.5)) ]))
  | "tanh" -> Some (mul (Num 0.5) (Fn ("ln", [ div (add one target) (sub one target) ])))
  | _ -> None

let residual lhs rhs = norm (sub lhs rhs)

let verify equation root =
  let x = value_at 0.0 (norm root) in
  if Float.is_nan x || Float.abs x = Float.infinity then None
  else
    let v = value_at x equation in
    if Float.is_nan v then None
    else if Float.abs v <= 1e-7 *. (1.0 +. Float.abs x) then Some x
    else None

let clear_denominators e =
  match norm e with
  | Mul factors ->
      let keep = List.filter (function Pow (b, Num k) when k < 0.0 && depends b -> false | _ -> true) factors in
      if List.length keep = List.length factors then norm e else norm_mul keep
  | Add terms ->
      let denominators =
        List.concat_map
          (fun t ->
            match t with
            | Mul factors ->
                List.filter_map
                  (function Pow (b, Num k) when k < 0.0 && depends b -> Some (power b (Num (-.k))) | _ -> None)
                  factors
            | Pow (b, Num k) when k < 0.0 && depends b -> [ power b (Num (-.k)) ]
            | _ -> [])
          terms
      in
      let denominators = List.sort_uniq compare_expr denominators in
      if denominators = [] then norm e
      else norm (List.fold_left (fun acc d -> mul acc d) (norm e) denominators)
  | other -> other

let solve equation =
  let cleared = clear_denominators equation in
  let numeric =
    try
      let roots = polynomial_roots (coefficients cleared) in
      Some (List.map (fun r -> Num (tidy r)) roots)
    with Unsolved | Error _ -> None
  in
  let symbolic =
    match numeric with
    | Some _ -> None
    | None -> ( try Some (shift_into cleared zero) with Unsolved | Error _ | Stack_overflow -> None)
  in
  let candidates =
    match (numeric, symbolic) with Some r, _ -> r | None, Some r -> r | None, None -> unsolved ()
  in
  let kept =
    List.filter_map
      (fun root ->
        match verify equation root with
        | Some _ -> Some (norm root)
        | None -> if depends root then Some (norm root) else None)
      candidates
  in
  List.sort_uniq compare_expr kept

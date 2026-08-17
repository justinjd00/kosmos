type t =
  | Num of float
  | Var
  | Add of t list
  | Mul of t list
  | Pow of t * t
  | Fn of string * t list

exception Error of string

let fail msg = raise (Error msg)

let num n = Num n
let zero = Num 0.0
let one = Num 1.0
let neg_one = Num (-1.0)

let is_num n = function Num v -> Float.abs (v -. n) < 1e-12 | _ -> false

let as_num = function Num v -> Some v | _ -> None

let is_int v = Float.is_integer v && Float.abs v < 1e15

let unary =
  [ "sin"; "cos"; "tan"; "asin"; "acos"; "atan"; "sinh"; "cosh"; "tanh";
    "exp"; "ln"; "log2"; "log10"; "sqrt"; "cbrt"; "abs"; "sign"; "floor";
    "ceil"; "round" ]

let binary_fn = [ "atan2"; "min"; "max"; "pow"; "log"; "hypot" ]

let alias = function
  | "arcsin" -> "asin"
  | "arccos" -> "acos"
  | "arctan" -> "atan"
  | "lb" -> "log2"
  | "lg" -> "log10"
  | "sgn" -> "sign"
  | name -> name

let constant = function
  | "pi" -> Some (4.0 *. Float.atan 1.0)
  | "tau" -> Some (8.0 *. Float.atan 1.0)
  | "e" -> Some (Float.exp 1.0)
  | "phi" -> Some 1.618033988749895
  | "inf" | "infinity" -> Some Float.infinity
  | _ -> None

type token =
  | Tnum of float
  | Tident of string
  | Tplus
  | Tminus
  | Tstar
  | Tslash
  | Tcaret
  | Tlparen
  | Trparen
  | Tcomma
  | Teq
  | Teof

let starts_value = function
  | Tnum _ | Tident _ | Tlparen | Tminus -> true
  | _ -> false

let ends_value = function Tnum _ | Tident _ | Trparen -> true | _ -> false

let lex source =
  let n = String.length source in
  let out = ref [] in
  let i = ref 0 in
  let push tok = out := tok :: !out in
  while !i < n do
    let c = source.[!i] in
    if c = ' ' || c = '\t' || c = '\n' || c = '\r' then incr i
    else if
      (c >= '0' && c <= '9')
      || (c = '.' && !i + 1 < n && source.[!i + 1] >= '0' && source.[!i + 1] <= '9')
    then begin
      let start = !i in
      while !i < n && ((source.[!i] >= '0' && source.[!i] <= '9') || source.[!i] = '.') do
        incr i
      done;
      if !i < n && (source.[!i] = 'e' || source.[!i] = 'E') then begin
        let save = !i in
        incr i;
        if !i < n && (source.[!i] = '+' || source.[!i] = '-') then incr i;
        if !i < n && source.[!i] >= '0' && source.[!i] <= '9' then
          while !i < n && source.[!i] >= '0' && source.[!i] <= '9' do
            incr i
          done
        else i := save
      end;
      let text = String.sub source start (!i - start) in
      match float_of_string_opt text with
      | Some v -> push (Tnum v)
      | None -> fail (Printf.sprintf "'%s' is not a number" text)
    end
    else if
      (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c = '_'
    then begin
      let start = !i in
      while
        !i < n
        &&
        let d = source.[!i] in
        (d >= 'a' && d <= 'z') || (d >= 'A' && d <= 'Z') || (d >= '0' && d <= '9') || d = '_'
      do
        incr i
      done;
      push (Tident (String.lowercase_ascii (String.sub source start (!i - start))))
    end
    else begin
      (match c with
      | '+' -> push Tplus
      | '-' -> push Tminus
      | '*' -> push Tstar
      | '/' -> push Tslash
      | '^' -> push Tcaret
      | '(' | '[' | '{' -> push Tlparen
      | ')' | ']' | '}' -> push Trparen
      | ',' | ';' -> push Tcomma
      | '=' -> push Teq
      | _ -> fail (Printf.sprintf "unexpected character '%c'" c));
      incr i
    end
  done;
  Array.of_list (List.rev (Teof :: !out))

type parser_state = { toks : token array; mutable pos : int; var : string }

let peek p = p.toks.(p.pos)
let advance p = p.pos <- p.pos + 1

let precedence = function
  | Tplus | Tminus -> Some (1, true)
  | Tstar | Tslash -> Some (2, true)
  | Tcaret -> Some (4, false)
  | _ -> None

let rec parse_expr p limit =
  let left = ref (parse_unary p) in
  let continue_ = ref true in
  while !continue_ do
    let tok = peek p in
    let implicit = ends_value p.toks.(p.pos - 1) && starts_value tok && tok <> Tminus in
    if implicit && 2 >= limit then left := Mul [ !left; parse_expr p 3 ]
    else
      match precedence tok with
      | Some (prec, left_assoc) when prec >= limit ->
          advance p;
          let next = if left_assoc then prec + 1 else prec in
          let right = parse_expr p next in
          left :=
            (match tok with
            | Tplus -> Add [ !left; right ]
            | Tminus -> Add [ !left; Mul [ neg_one; right ] ]
            | Tstar -> Mul [ !left; right ]
            | Tslash -> Mul [ !left; Pow (right, neg_one) ]
            | Tcaret -> Pow (!left, right)
            | _ -> fail "impossible operator")
      | _ -> continue_ := false
  done;
  !left

and parse_unary p =
  match peek p with
  | Tminus ->
      advance p;
      Mul [ neg_one; parse_unary p ]
  | Tplus ->
      advance p;
      parse_unary p
  | _ -> parse_atom p

and parse_atom p =
  match peek p with
  | Tnum v ->
      advance p;
      Num v
  | Tlparen ->
      advance p;
      let inner = parse_expr p 1 in
      if peek p <> Trparen then fail "missing closing bracket";
      advance p;
      inner
  | Tident raw -> (
      advance p;
      let name = alias raw in
      if peek p = Tlparen then begin
        advance p;
        let args = ref [] in
        if peek p <> Trparen then begin
          args := [ parse_expr p 1 ];
          while peek p = Tcomma do
            advance p;
            args := parse_expr p 1 :: !args
          done
        end;
        if peek p <> Trparen then fail (Printf.sprintf "missing ) after %s(" name);
        advance p;
        let args = List.rev !args in
        let arity = List.length args in
        if List.mem name unary && arity <> 1 then
          fail (Printf.sprintf "%s takes one argument" name);
        if List.mem name binary_fn && arity <> 2 then
          fail (Printf.sprintf "%s takes two arguments" name);
        if (not (List.mem name unary)) && not (List.mem name binary_fn) then
          fail (Printf.sprintf "unknown function '%s'" name);
        Fn (name, args)
      end
      else if name = p.var then Var
      else
        match constant name with
        | Some v -> Num v
        | None -> fail (Printf.sprintf "unknown name '%s'" name))
  | _ -> fail "expected a value"

let parse ?(var = "x") source =
  let p = { toks = lex source; pos = 0; var } in
  if peek p = Teof then fail "nothing to read";
  let e = parse_expr p 1 in
  if peek p <> Teof then fail "trailing input";
  e

let split_equation ?(var = "x") source =
  let toks = lex source in
  let idx = ref (-1) in
  Array.iteri (fun i t -> if t = Teq && !idx < 0 then idx := i) toks;
  if !idx < 0 then (parse ~var source, zero)
  else
    let cut = ref (-1) in
    String.iteri (fun i c -> if c = '=' && !cut < 0 then cut := i) source;
    let lhs = String.sub source 0 !cut in
    let rhs = String.sub source (!cut + 1) (String.length source - !cut - 1) in
    (parse ~var lhs, parse ~var rhs)

let shortest v =
  let rec attempt digits =
    if digits > 17 then Printf.sprintf "%.17g" v
    else
      let text = Printf.sprintf "%.*g" digits v in
      if float_of_string text = v then text else attempt (digits + 1)
  in
  attempt 1

let rational v =
  if is_int v then Some (v, 1.0)
  else
    let limit = 1e-13 *. Float.max 1.0 (Float.abs v) in
    let rec search q =
      if q > 10000.0 then None
      else
        let p = Float.round (v *. q) in
        if Float.abs (v -. (p /. q)) <= limit && Float.abs p < 1e15 then Some (p, q)
        else search (q +. 1.0)
    in
    search 2.0

let show_number v =
  if Float.is_nan v then "nan"
  else if v = Float.infinity then "inf"
  else if v = Float.neg_infinity then "-inf"
  else if is_int v then Printf.sprintf "%.0f" v
  else
    let short = shortest v in
    if String.length short <= 8 then short
    else
      match rational v with
      | Some (p, q) when q > 1.0 -> Printf.sprintf "%.0f/%.0f" p q
      | _ -> short

let rank = function
  | Add _ -> 1
  | Mul _ -> 2
  | Pow _ -> 3
  | Num _ | Var | Fn _ -> 4

let rec print ?(var = "x") e = to_string var 0 e

and to_string var level e =
  let wrap need s = if need then "(" ^ s ^ ")" else s in
  match e with
  | Num v -> if v < 0.0 then wrap (level > 0) (show_number v) else show_number v
  | Var -> var
  | Fn (name, args) ->
      name ^ "(" ^ String.concat ", " (List.map (to_string var 0) args) ^ ")"
  | Pow (b, Num e') when e' = -1.0 ->
      wrap (level > 2) ("1/" ^ to_string var 3 b)
  | Pow (b, e') -> wrap (level > 3) (to_string var 4 b ^ "^" ^ to_string var 3 e')
  | Mul factors -> wrap (level > 2) (print_mul var factors)
  | Add terms -> wrap (level > 1) (print_add var terms)

and print_mul var factors =
  let tops, bottoms =
    List.partition (function Pow (_, Num e) when e < 0.0 -> false | _ -> true) factors
  in
  let coefficient, tops =
    match tops with Num v :: rest -> (v, rest) | _ -> (1.0, tops)
  in
  let sign = if coefficient < 0.0 then "-" else "" in
  let magnitude = Float.abs coefficient in
  let p, q = match rational magnitude with Some (p, q) -> (p, q) | None -> (magnitude, 1.0) in
  let scaled = if q > 1.0 then p else magnitude in
  let above =
    (if scaled = 1.0 && tops <> [] then [] else [ show_number scaled ])
    @ List.map (to_string var 3) tops
  in
  let below =
    (if q > 1.0 then [ show_number q ] else [])
    @ List.map
        (function
          | Pow (b, Num e) when e = -1.0 -> to_string var 4 b
          | Pow (b, Num e) -> to_string var 4 b ^ "^" ^ show_number (-.e)
          | other -> to_string var 4 other)
        bottoms
  in
  let above = if above = [] then [ "1" ] else above in
  let head = sign ^ String.concat "*" above in
  match below with [] -> head | _ -> head ^ "/" ^ String.concat "/" below

and degree = function
  | Num _ -> 0.0
  | Var -> 1.0
  | Pow (b, Num e) -> degree b *. e
  | Pow (b, _) -> degree b
  | Mul xs -> List.fold_left (fun acc f -> acc +. degree f) 0.0 xs
  | Add xs -> List.fold_left (fun acc t -> Float.max acc (degree t)) 0.0 xs
  | Fn (_, xs) -> if List.exists (fun a -> degree a > 0.0) xs then 0.5 else 0.0

and print_add var terms =
  let terms = List.stable_sort (fun a b -> compare (degree b) (degree a)) terms in
  match terms with
  | [] -> "0"
  | first :: rest ->
      let buf = Buffer.create 32 in
      Buffer.add_string buf (to_string var 1 first);
      List.iter
        (fun term ->
          match term with
          | Num v when v < 0.0 ->
              Buffer.add_string buf (" - " ^ show_number (-.v))
          | Mul (Num v :: tail) when v < 0.0 ->
              let body = if v = -1.0 then tail else Num (-.v) :: tail in
              let body = if body = [] then [ one ] else body in
              Buffer.add_string buf (" - " ^ to_string var 2 (Mul body))
          | _ -> Buffer.add_string buf (" + " ^ to_string var 1 term))
        rest;
      Buffer.contents buf

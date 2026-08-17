open Syntax
open Algebra

exception Impossible

let taylor e about order =
  if order < 0 || order > 12 then raise Impossible;
  let centre = Num about in
  let terms = ref [] in
  let current = ref (norm e) in
  let factorial = ref 1.0 in
  for k = 0 to order do
    if k > 0 then factorial := !factorial *. float_of_int k;
    let coefficient = value_at about !current in
    if Float.is_nan coefficient || Float.abs coefficient = Float.infinity then raise Impossible;
    let shifted = if about = 0.0 then Var else sub Var centre in
    let piece = mul (Num (coefficient /. !factorial)) (power shifted (Num (float_of_int k))) in
    terms := piece :: !terms;
    if k < order then current := norm (derive !current)
  done;
  norm_add (List.rev !terms)

let failures = ref 0
let checks = ref 0

let report label detail =
  incr failures;
  print_endline ("FAIL  " ^ label ^ "  " ^ detail)

let show e = Syntax.print e

let parsed source =
  try Some (Syntax.parse source) with Syntax.Error message -> print_endline ("parse: " ^ message); None

let numerically_equal a b =
  let probes = [ -2.3; -1.1; -0.4; 0.17; 0.62; 1.3; 2.9; 4.1 ] in
  let usable = ref 0 in
  let agree =
    List.for_all
      (fun x ->
        let u = Algebra.value_at x a and v = Algebra.value_at x b in
        if Float.is_nan u || Float.is_nan v || Float.abs u = Float.infinity then true
        else begin
          incr usable;
          Float.abs (u -. v) <= 1e-7 *. (1.0 +. Float.abs v)
        end)
      probes
  in
  agree && !usable >= 3

let simplifies source expected =
  incr checks;
  match parsed source with
  | None -> report source "did not parse"
  | Some e ->
      let got = show (Algebra.norm e) in
      if got <> expected then report source (Printf.sprintf "got %s, wanted %s" got expected)

let integrates source =
  incr checks;
  match parsed source with
  | None -> report source "did not parse"
  | Some e -> (
      match Integrate.antiderivative e with
      | result ->
          let back = Algebra.norm (Algebra.derive result) in
          if not (numerically_equal back (Algebra.norm e)) then
            report source (Printf.sprintf "integral %s differentiates back wrong" (show result))
          else print_endline (Printf.sprintf "  int %-28s = %s" source (show result))
      | exception Integrate.Give_up -> report source "no closed form"
      | exception Syntax.Error message -> report source message)

let refuses source =
  incr checks;
  match parsed source with
  | None -> report source "did not parse"
  | Some e -> (
      match Integrate.antiderivative e with
      | result -> report source ("should have refused, produced " ^ show result)
      | exception _ -> ())

let solves source expected =
  incr checks;
  let lhs, rhs = Syntax.split_equation source in
  match Solve.solve (Algebra.sub lhs rhs) with
  | roots ->
      let got = String.concat ", " (List.map show roots) in
      if got <> expected then report source (Printf.sprintf "got [%s], wanted [%s]" got expected)
  | exception _ -> report source "no solution found"

let series source about order expected =
  incr checks;
  match parsed source with
  | None -> report source "did not parse"
  | Some e ->
      let got = show (Series.taylor e about order) in
      if got <> expected then report source (Printf.sprintf "got %s, wanted %s" got expected)

let expands source expected =
  incr checks;
  match parsed source with
  | None -> report source "did not parse"
  | Some e ->
      let got = show (Algebra.expand e) in
      if got <> expected then report source (Printf.sprintf "got %s, wanted %s" got expected)

let () =
  simplifies "x + x" "2*x";
  simplifies "x*x" "x^2";
  simplifies "2*(x+3)" "2*x + 6";
  simplifies "x - x" "0";
  simplifies "0*sin(x)" "0";
  simplifies "x/x" "1";
  simplifies "(x^2)^3" "x^6";
  simplifies "sin(x)/sin(x)" "1";
  simplifies "3*x + 4*x - 2" "7*x - 2";
  simplifies "exp(ln(x))" "x";
  simplifies "sqrt(x)*sqrt(x)" "x";
  simplifies "2*3*x" "6*x";
  simplifies "1/(1/x)" "x";

  integrates "x^3";
  integrates "1/x";
  integrates "sin(x)";
  integrates "cos(3*x)";
  integrates "exp(-2*x)";
  integrates "x*exp(x)";
  integrates "x^2*sin(x)";
  integrates "ln(x)";
  integrates "x*ln(x)";
  integrates "1/(1+x^2)";
  integrates "1/sqrt(1-x^2)";
  integrates "x/(1+x^2)";
  integrates "2*x*exp(x^2)";
  integrates "sin(x)*cos(x)";
  integrates "tan(x)";
  integrates "(2*x+1)^5";
  integrates "1/(2*x+1)";
  integrates "x*sin(x^2)";
  integrates "atan(x)";
  integrates "x^2 + 3*x + 1";
  integrates "sqrt(x)";
  integrates "exp(x)*cos(x)" ;

  refuses "exp(x^2)";
  refuses "sin(x)/x";

  solves "x^2 - 4 = 0" "-2, 2";
  solves "2*x + 6 = 0" "-3";
  solves "x^2 + 1 = 0" "";
  solves "x^3 - x = 0" "-1, 0, 1";
  solves "exp(x) = 1" "0";
  solves "ln(x) = 0" "1";
  solves "1/x = 2" "0.5";
  solves "x^2 = 2*x" "0, 2";
  solves "sin(x) = 0" "0";

  series "exp(x)" 0.0 4 "x^4/24 + x^3/6 + x^2/2 + x + 1";
  series "sin(x)" 0.0 5 "x^5/120 - x^3/6 + x";

  expands "(x+1)*(x+1) - x^2" "2*x + 1";
  expands "(x+1)^3" "x^3 + 3*x^2 + 3*x + 1";
  expands "(x-1)*(x+1)" "x^2 - 1";
  expands "(x+2)^2 - (x-2)^2" "8*x";

  Printf.printf "\n%d checks, %d failures\n" !checks !failures;
  if !failures > 0 then exit 1

open Js_of_ocaml

type outcome = { ok : bool; text : string }

let failure message = { ok = false; text = message }
let success text = { ok = true; text }

let explain = function
  | Syntax.Error message -> message
  | Integrate.Give_up -> "no closed form found"
  | Solve.Unsolved -> "no solution found"
  | Series.Impossible -> "the series does not exist here"
  | Stack_overflow -> "the expression is too deeply nested"
  | other -> Printexc.to_string other

let guard f = try success (f ()) with exn -> failure (explain exn)

let read source variable = Syntax.parse ~var:variable source
let write variable e = Syntax.print ~var:variable e

let simplify source variable =
  guard (fun () ->
      let e = read source variable in
      let plain = Algebra.norm e in
      let opened = Algebra.expand e in
      let choice = if Algebra.size opened <= Algebra.size plain then opened else plain in
      write variable choice)

let differentiate source variable =
  guard (fun () -> write variable (Algebra.derive (Algebra.norm (read source variable))))

let integrate source variable =
  guard (fun () -> write variable (Integrate.antiderivative (read source variable)))

let solve source variable =
  guard (fun () ->
      let lhs, rhs = Syntax.split_equation ~var:variable source in
      let roots = Solve.solve (Algebra.sub lhs rhs) in
      if roots = [] then "no real solution"
      else String.concat ", " (List.map (fun r -> variable ^ " = " ^ write variable r) roots))

let taylor source variable about order =
  guard (fun () -> write variable (Series.taylor (read source variable) about order))

let expose name (f : string -> string -> outcome) =
  Js.Unsafe.set
    (Js.Unsafe.pure_js_expr "globalThis.kosmosCas")
    (Js.string name)
    (Js.wrap_callback (fun source variable ->
         let result = f (Js.to_string source) (Js.to_string variable) in
         object%js
           val ok = Js.bool result.ok
           val text = Js.string result.text
         end))

let () =
  Js.Unsafe.set (Js.Unsafe.pure_js_expr "globalThis") (Js.string "kosmosCas") (object%js end);
  expose "simplify" simplify;
  expose "derivative" differentiate;
  expose "integral" integrate;
  expose "solve" solve;
  Js.Unsafe.set
    (Js.Unsafe.pure_js_expr "globalThis.kosmosCas")
    (Js.string "taylor")
    (Js.wrap_callback (fun source variable about order ->
         let result = taylor (Js.to_string source) (Js.to_string variable) (Js.float_of_number about) (int_of_float (Js.float_of_number order)) in
         object%js
           val ok = Js.bool result.ok
           val text = Js.string result.text
         end));
  Js.Unsafe.set
    (Js.Unsafe.pure_js_expr "globalThis.kosmosCas")
    (Js.string "version")
    (Js.string "1")

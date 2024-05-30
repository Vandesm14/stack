use js_sys::{Array, Boolean};
use stack_core::prelude::*;
use wasm_bindgen::prelude::*;

// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// // Import the `window.alert` function from the Web.
// #[wasm_bindgen]
// extern "C" {
//   fn alert(s: &str);
// }

#[wasm_bindgen]
pub fn run(code: &str) -> Result<JsValue, JsError> {
  let code = code.to_owned();
  let source = Source::new("runner", code);
  let mut lexer = Lexer::new(source);
  let exprs = parse(&mut lexer).unwrap();

  let engine = Engine::new();
  let context = Context::new();

  // engine.add_module(stack_std::str::module());
  // engine.add_module(stack_std::fs::module(false));
  // engine.add_module(stack_std::scope::module());

  let result = engine.run(context, exprs);
  match result {
    Ok(context) => Ok(
      context
        .stack()
        .iter()
        .map(|e| match e.kind {
          ExprKind::Nil => JsValue::null(),

          ExprKind::Boolean(bool) => JsValue::from(bool),
          ExprKind::Integer(int) => JsValue::from(int),
          ExprKind::Float(float) => JsValue::from_f64(float),
          ExprKind::String(ref str) => JsValue::from_str(str.as_str()),

          ExprKind::Symbol(sym) => JsValue::symbol(Some(sym.as_str())),

          _ => JsValue::undefined(),
        })
        .collect::<Array>()
        .into(),
    ),
    Err(err) => Err(JsError::new(&err.to_string())),
  }
}

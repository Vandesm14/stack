use stack::Program;

fn run() -> Result<Program, stack::EvalError> {
  let string = include_str!("for_each.stack");
  let mut program = Program::new().with_core().unwrap();

  match program.eval_string(string) {
    Ok(_) => Ok(program),
    Err(error) => Err(error),
  }
}

fn main() {
  match run() {
    Ok(program) => println!("Result: {}", program),
    Err(error) => println!("Error: {}", error),
  }
}

#[cfg(test)]
mod tests {
  use stack::Expr;

  use super::*;

  #[test]
  fn for_each() {
    let mut result = run().unwrap();
    let expected = result.context.intern("the words should be in order");

    assert_eq!(result.stack, vec![Expr::String(expected)]);
  }
}

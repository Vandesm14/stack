use stack::Program;

fn run() -> Result<Program, stack::EvalError> {
  let string = include_str!("let.stack");
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
  use super::*;
  use stack::Expr;

  #[test]
  fn let_fn() {
    let result = run();
    assert!(result.is_ok());
    assert_eq!(result.unwrap().stack, vec![Expr::Integer(8)]);
  }
}

use stack::Program;

fn run() -> Result<Program, stack::EvalError> {
  let string = include_str!("reverse.stack");
  let mut program = Program::new();

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

  #[test]
  fn test_debug() {
    assert!(run().is_ok());
  }
}

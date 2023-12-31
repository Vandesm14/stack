use stack::Program;

fn run() -> Result<Program, stack::EvalError> {
  let string = include_str!("syscall.stack");
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

  #[test]
  fn syscall() {
    run().unwrap();
  }
}

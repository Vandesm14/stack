use stack::Program;

fn main() {
  let string = include_str!("debug.stack");
  let mut program = Program::new();

  match program.eval_string(string) {
    Ok(_) => println!("Result: {:?}", program.stack),
    Err(error) => println!("Error: {}", error),
  }
}

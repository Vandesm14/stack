use stack_core::{compiler::VM, prelude::*};

fn main() {
  let source = Source::new("", "10 2 2 + -");
  let mut lexer = Lexer::new(source);
  let exprs = parse(&mut lexer).unwrap();

  let mut vm = VM::new();
  vm.compile(exprs);

  loop {
    if let Err(err) = vm.step() {
      eprintln!("{err:?}");
      break;
    }
  }

  println!("{vm:?}");
}

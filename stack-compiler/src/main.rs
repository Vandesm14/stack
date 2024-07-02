use stack_core::{compiler::VM, prelude::*};

fn main() {
  let source = Source::new("", "(fn 2 2 +)");
  let mut lexer = Lexer::new(source);
  let exprs = parse(&mut lexer).unwrap();

  let mut vm = VM::new();
  vm.compile(exprs);

  // loop {
  //   if let Err(err) = vm.step() {
  //     eprintln!("{err:?}");
  //     break;
  //   }
  // }

  println!("{vm:?}");
}

#[cfg(test)]
mod tests {
  use stack_core::val::Val;

  use super::*;

  mod execution {
    use super::*;

    #[test]
    fn stack_ops_should_work() {
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

      assert_eq!(vm.stack(), &[Val::Integer(6)])
    }

    #[test]
    fn immediate_functions_run() {
      let source = Source::new("", "10 (fn 2 2 +) -");
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

      assert_eq!(vm.stack(), &[Val::Integer(6)])
    }
  }

  mod compilation {
    use super::*;
  }
}

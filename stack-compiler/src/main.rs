use stack_core::{compiler::VM, prelude::*};

fn main() {
  let source = Source::new("", include_str!("../../testing/test.stack"));
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

  println!("{vm:#?}");
}

#[cfg(test)]
mod tests {
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

      assert_eq!(
        vm.stack().iter().map(|expr| &expr.kind).collect::<Vec<_>>(),
        vec![&ExprKind::Integer(6)]
      )
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

      assert_eq!(
        vm.stack().iter().map(|expr| &expr.kind).collect::<Vec<_>>(),
        vec![&ExprKind::Integer(6)]
      )
    }

    #[test]
    fn lazy_functions_run() {
      let source = Source::new("", "'(fn 2 2 +) call");
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

      assert_eq!(
        vm.stack().iter().map(|expr| &expr.kind).collect::<Vec<_>>(),
        vec![&ExprKind::Integer(4)]
      )
    }
  }

  mod compilation {
    use super::*;
  }
}

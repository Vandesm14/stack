use core::{fmt, str::FromStr};

use crate::{
  compiler::{VMError, VM},
  val::Val,
};

macro_rules! intrinsics {
  ($($ident:ident => ($s:literal, $b:literal)),* $(,)?) => {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Intrinsic {
      $($ident),*
    }

    impl Intrinsic {
      /// Returns the <code>&[str]</code> name of this [`Intrinsic`].
      pub const fn as_str(self) -> &'static str {
        match self {
          $(Self::$ident => $s),*
        }
      }

      /// Returns if the args should be flipped when running this [`Intrinsic`].
      pub const fn has_flipped_s_expr_args(self) -> bool {
        match self {
          $(Self::$ident => $b),*
        }
      }

      /// Returns all of the [`Intrinsic`]s as a <code>&\[&[str]\]</code>.
      // TODO: Is there a better name than this?
      pub const fn all_as_slice() -> &'static [&'static str] {
        &[$($s),*]
      }
    }

    impl FromStr for Intrinsic {
      type Err = ParseIntrinsicError;

      fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
          $($s => Ok(Self::$ident),)*
          _ => Err(ParseIntrinsicError),
        }
      }
    }

    impl fmt::Display for Intrinsic {
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
      }
    }
  };
}

intrinsics! {
  Add => ("+",false),
  Sub => ("-", false),
  Mul => ("*", false),
  Div => ("/", false),
  Rem => ("%", false),

  Eq => ("=", false),
  Ne => ("!=", false),
  Lt => ("<", false),
  Le => ("<=", false),
  Gt => (">", false),
  Ge => (">=", false),

  Or => ("or", false),
  And => ("and", false),
  Not => ("not", false),

  Assert => ("assert", false),

  Drop => ("drop", false),
  Dupe => ("dupe", false),
  Swap => ("swap", false),
  Rot => ("rot", false),

  Len => ("len", false),
  Nth => ("nth", false),
  Split => ("split", false),
  Concat => ("concat", false),
  Push => ("push", true),
  Pop => ("pop", false),

  Insert => ("insert", true),
  Prop => ("prop", false),
  Has => ("has", false),
  Remove => ("remove", false),
  Keys => ("keys", false),
  Values => ("values", false),

  Cast => ("cast", false),
  TypeOf => ("typeof", false),
  Lazy => ("lazy", false),

  If => ("if", false),
  Halt => ("halt", false),

  Call => ("call", false),

  Let => ("let", true),
  Def => ("def", true),
  Set => ("set", true),
  Get => ("get", false),

  Debug => ("debug", false),
  // TODO: These will become STD module items.
  Print => ("print", false),
  Pretty => ("pretty", false),
  Recur => ("recur", false),

  OrElse => ("orelse", false),

  Import => ("import", false),
}

impl Intrinsic {
  pub fn run(&self, vm: &mut VM) -> Result<(), VMError> {
    match self {
      // MARK: Add
      Self::Add => {
        let rhs = vm.stack_pop()?;
        let lhs = vm.stack_pop()?;

        let result = match lhs + rhs {
          Ok(res) => res,
          Err(_) => todo!(),
        };

        vm.stack_push(result);

        Ok(())
      }
      // MARK: Sub
      Self::Sub => {
        let rhs = vm.stack_pop()?;
        let lhs = vm.stack_pop()?;

        let result = match lhs - rhs {
          Ok(res) => res,
          Err(_) => todo!(),
        };

        vm.stack_push(result);

        Ok(())
      }
      // MARK: Mul
      Self::Mul => {
        let rhs = vm.stack_pop()?;
        let lhs = vm.stack_pop()?;

        let result = match lhs * rhs {
          Ok(res) => res,
          Err(_) => todo!(),
        };

        vm.stack_push(result);

        Ok(())
      }
      // MARK: Div
      Self::Div => {
        let rhs = vm.stack_pop()?;
        let lhs = vm.stack_pop()?;

        let result = match lhs / rhs {
          Ok(res) => res,
          Err(_) => todo!(),
        };

        vm.stack_push(result);

        Ok(())
      }
      // MARK: Rem
      Self::Rem => {
        let rhs = vm.stack_pop()?;
        let lhs = vm.stack_pop()?;

        let result = match lhs % rhs {
          Ok(res) => res,
          Err(_) => todo!(),
        };

        vm.stack_push(result);

        Ok(())
      }

      // MARK: Eq
      Self::Eq => {
        let rhs = vm.stack_pop()?;
        let lhs = vm.stack_pop()?;

        let result = Val::Boolean(lhs == rhs);

        vm.stack_push(result);

        Ok(())
      }
      // MARK: Ne
      Self::Ne => {
        let rhs = vm.stack_pop()?;
        let lhs = vm.stack_pop()?;

        let result = Val::Boolean(lhs != rhs);

        vm.stack_push(result);

        Ok(())
      }
      // MARK: Lt
      Self::Lt => {
        let rhs = vm.stack_pop()?;
        let lhs = vm.stack_pop()?;

        let result = Val::Boolean(lhs < rhs);

        vm.stack_push(result);

        Ok(())
      }
      // MARK: Le
      Self::Le => {
        let rhs = vm.stack_pop()?;
        let lhs = vm.stack_pop()?;

        let result = Val::Boolean(lhs <= rhs);

        vm.stack_push(result);

        Ok(())
      }
      // MARK: Gt
      Self::Gt => {
        let rhs = vm.stack_pop()?;
        let lhs = vm.stack_pop()?;

        let result = Val::Boolean(lhs > rhs);

        vm.stack_push(result);

        Ok(())
      }
      // MARK: Ge
      Self::Ge => {
        let rhs = vm.stack_pop()?;
        let lhs = vm.stack_pop()?;

        let result = Val::Boolean(lhs >= rhs);

        vm.stack_push(result);

        Ok(())
      }

      // MARK: Or
      Self::Or => {
        // let rhs = vm.stack_pop()?;
        // let lhs = vm.stack_pop()?;

        // let result =
        //   ExprKind::Boolean(lhs.kind.is_truthy() || rhs.kind.is_truthy());

        // vm.stack_push(kind.into())?;

        // Ok(vm)

        todo!()
      }
      // MARK: And
      Self::And => {
        // let rhs = vm.stack_pop()?;
        // let lhs = vm.stack_pop()?;

        // let result =
        //   ExprKind::Boolean(lhs.kind.is_truthy() && rhs.kind.is_truthy());

        // vm.stack_push(kind.into())?;

        // Ok(vm)

        todo!()
      }
      // MARK: Not
      Self::Not => {
        // let rhs = vm.stack_pop()?;

        // let result = ExprKind::Boolean(!rhs.kind.is_truthy());

        // vm.stack_push(kind.into())?;

        // Ok(vm)

        todo!()
      }

      // MARK: Assert
      Self::Assert => {
        // let bool = context.stack_pop(&expr)?;
        // let message = context.stack_pop(&expr)?;

        // if bool.kind.is_truthy() {
        //   Ok(context)
        // } else {
        //   Err(RunError {
        //     reason: RunErrorReason::AssertionFailed,
        //     context,
        //     expr: message.kind.into(),
        //   })
        // }

        todo!()
      }

      // MARK: Drop
      Self::Drop => {
        vm.stack_pop()?;
        Ok(())
      }
      // MARK: Dupe
      Self::Dupe => {
        // let item = context.stack().last().cloned().ok_or_else(|| RunError {
        //   reason: RunErrorReason::StackUnderflow,
        //   context: context.clone(),
        //   expr,
        // })?;

        // context.stack_push(item)?;
        // Ok(context)

        todo!()
      }
      // MARK: Swap
      Self::Swap => {
        // let len = context.stack().len();

        // if len >= 2 {
        //   context.stack_mut().swap(len - 1, len - 2);
        //   Ok(context)
        // } else {
        //   Err(RunError {
        //     reason: RunErrorReason::StackUnderflow,
        //     context,
        //     expr,
        //   })
        // }

        todo!()
      }
      // MARK: Rot
      Self::Rot => {
        // let len = context.stack().len();

        // if len >= 3 {
        //   context.stack_mut().swap(len - 1, len - 3);
        //   context.stack_mut().swap(len - 2, len - 3);

        //   Ok(context)
        // } else {
        //   Err(RunError {
        //     reason: RunErrorReason::StackUnderflow,
        //     context,
        //     expr,
        //   })
        // }

        todo!()
      }

      // MARK: Len
      Self::Len => {
        // let item = context.stack_pop(&expr)?;

        // let kind = match item.kind {
        //   ExprKind::List(ref x) => {
        //     debug_assert!(x.len() <= i64::MAX as usize);
        //     ExprKind::Integer(x.len() as i64)
        //   }
        //   ExprKind::String(ref x) => {
        //     let len = x.graphemes(true).count();
        //     debug_assert!(len <= i64::MAX as usize);
        //     ExprKind::Integer(len as i64)
        //   }
        //   ExprKind::Record(ref x) => {
        //     debug_assert!(x.len() <= i64::MAX as usize);
        //     ExprKind::Integer(x.len() as i64)
        //   }
        //   _ => ExprKind::Nil,
        // };

        // context.stack_push(item.clone())?;

        // context.stack_push(kind.into())?;

        // Ok(context)

        todo!()
      }
      // MARK: Nth
      Self::Nth => {
        // let index = context.stack_pop(&expr)?;
        // let item = context.stack_pop(&expr)?;

        // let kind = match (item.kind.clone(), index.kind) {
        //   (ExprKind::List(x), ExprKind::Integer(i)) if i >= 0 => x
        //     .get(i as usize)
        //     .map(|x| x.kind.clone())
        //     .unwrap_or(ExprKind::Nil),
        //   (ExprKind::String(x), ExprKind::Integer(i)) if i >= 0 => x
        //     .as_str()
        //     .graphemes(true)
        //     .nth(i as usize)
        //     .map(|x| ExprKind::String(x.into()))
        //     .unwrap_or(ExprKind::Nil),
        //   _ => ExprKind::Nil,
        // };

        // context.stack_push(item.clone())?;

        // context.stack_push(kind.into())?;

        // Ok(context)

        todo!()
      }
      // MARK: Split
      Self::Split => {
        // let index = context.stack_pop(&expr)?;
        // let item = context.stack_pop(&expr)?;

        // match (item.kind, index.kind) {
        //   (ExprKind::List(mut x), ExprKind::Integer(i)) if i >= 0 => {
        //     if (i as usize) < x.len() {
        //       let rest = x.split_off(i as usize);

        //       context.stack_push(ExprKind::List(x).into())?;

        //       context.stack_push(ExprKind::List(rest).into())?;
        //     } else {
        //       context.stack_push(ExprKind::List(x).into())?;

        //       context.stack_push(ExprKind::Nil.into())?;
        //     }
        //   }
        //   (ExprKind::String(mut x), ExprKind::Integer(i)) if i >= 0 => {
        //     match x.as_str().grapheme_indices(true).nth(i as usize) {
        //       Some((i, _)) => {
        //         let rest = x.split_off(i);

        //         context.stack_push(ExprKind::String(x).into())?;

        //         context.stack_push(ExprKind::String(rest).into())?;
        //       }
        //       None => {
        //         context.stack_push(ExprKind::String(x).into())?;

        //         context.stack_push(ExprKind::Nil.into())?;
        //       }
        //     }
        //   }
        //   _ => {
        //     context.stack_push(ExprKind::Nil.into())?;

        //     context.stack_push(ExprKind::Nil.into())?;
        //   }
        // }

        // Ok(context)

        todo!()
      }
      // MARK: Concat
      Self::Concat => {
        // let rhs = context.stack_pop(&expr)?;
        // let lhs = context.stack_pop(&expr)?;

        // let kind = match (lhs.kind, rhs.kind) {
        //   (ExprKind::List(mut lhs), ExprKind::List(rhs)) => {
        //     lhs.extend(rhs);
        //     ExprKind::List(lhs)
        //   }
        //   (ExprKind::String(mut lhs), ExprKind::String(rhs)) => {
        //     lhs.push_str(&rhs);
        //     ExprKind::String(lhs)
        //   }
        //   _ => ExprKind::Nil,
        // };

        // context.stack_push(kind.into())?;

        // Ok(context)

        todo!()
      }
      // MARK: Push
      Self::Push => {
        // let list = context.stack_pop(&expr)?;
        // let item = context.stack_pop(&expr)?;

        // let kind = match (list.kind.clone(), item.kind.clone()) {
        //   (ExprKind::List(mut x), i) => {
        //     x.push(Expr {
        //       kind: i,
        //       info: item.info.clone(),
        //     });
        //     ExprKind::List(x)
        //   }
        //   (ExprKind::String(mut x), ExprKind::String(s)) => {
        //     x.push_str(&s);
        //     ExprKind::String(x)
        //   }
        //   (ExprKind::String(mut x), ExprKind::Integer(c))
        //     if c >= 0 && c <= u32::MAX as i64 =>
        //   {
        //     if let Some(c) = char::from_u32(c as u32) {
        //       x.push(c);
        //       ExprKind::String(x)
        //     } else {
        //       ExprKind::Nil
        //     }
        //   }
        //   _ => ExprKind::Nil,
        // };

        // context.stack_push(kind.into())?;

        // Ok(context)

        todo!()
      }
      // MARK: Pop
      Self::Pop => {
        // let list = context.stack_pop(&expr)?;

        // match list.kind.clone() {
        //   ExprKind::List(mut x) => {
        //     let e = x.pop().unwrap_or(ExprKind::Nil.into());

        //     context.stack_push(ExprKind::List(x).into())?;
        //     context.stack_push(e)?;
        //   }
        //   ExprKind::String(mut x) => {
        //     let e = x
        //       .pop()
        //       .map(|e| ExprKind::String(e.to_compact_string()).into())
        //       .unwrap_or(ExprKind::Nil.into());

        //     context.stack_push(ExprKind::String(x).into())?;
        //     context.stack_push(e)?;
        //   }
        //   _ => {
        //     context.stack_push(list.clone())?;
        //     context.stack_push(ExprKind::Nil.into())?;
        //   }
        // }

        // Ok(context)

        todo!()
      }

      // MARK: Insert
      Self::Insert => {
        // let record = context.stack_pop(&expr)?;
        // let name = context.stack_pop(&expr)?;
        // let value = context.stack_pop(&expr)?;

        // match record.kind {
        //   ExprKind::Record(ref record) => {
        //     let symbol: Symbol = name.kind.into();

        //     let mut new_record = record.clone();
        //     new_record.insert(symbol, value);

        //     context.stack_push(ExprKind::Record(new_record).into())?;

        //     Ok(())
        //   }
        //   _ => context.stack_push(ExprKind::Nil.into()),
        // }
        // .map(|_| context)

        todo!()
      }
      // MARK: Prop
      Self::Prop => {
        // let name = context.stack_pop(&expr)?;
        // let record = context.stack_pop(&expr)?;

        // match record.kind {
        //   ExprKind::Record(ref r) => {
        //     let symbol: Symbol = name.kind.into();

        //     let result = r.get(&symbol).unwrap_or_else(|| &Expr {
        //       info: None,
        //       kind: ExprKind::Nil,
        //     });

        //     context.stack_push(record.clone())?;
        //     context.stack_push(result.clone())?;

        //     Ok(())
        //   }
        //   _ => context.stack_push(ExprKind::Nil.into()),
        // }
        // .map(|_| context)

        todo!()
      }
      // MARK: Has
      Self::Has => {
        // let name = context.stack_pop(&expr)?;
        // let record = context.stack_pop(&expr)?;

        // match record.kind {
        //   ExprKind::Record(ref r) => {
        //     let symbol: Symbol = name.kind.into();

        //     let result = r.contains_key(&symbol);

        //     context.stack_push(record.clone())?;
        //     context.stack_push(ExprKind::Boolean(result).into())?;

        //     Ok(())
        //   }
        //   _ => context.stack_push(ExprKind::Nil.into()),
        // }
        // .map(|_| context)

        todo!()
      }
      // MARK: Remove
      Self::Remove => {
        // let name = context.stack_pop(&expr)?;
        // let record = context.stack_pop(&expr)?;

        // match record.kind {
        //   ExprKind::Record(ref record) => {
        //     let symbol: Symbol = name.kind.into();

        //     let mut new_record = record.clone();
        //     new_record.remove(&symbol);

        //     context.stack_push(ExprKind::Record(new_record).into())?;

        //     Ok(())
        //   }
        //   _ => context.stack_push(ExprKind::Nil.into()),
        // }
        // .map(|_| context)

        todo!()
      }
      // MARK: Keys
      Self::Keys => {
        // let record = context.stack_pop(&expr)?;

        // match record.kind {
        //   ExprKind::Record(ref r) => {
        //     let result = r
        //       .keys()
        //       .copied()
        //       .map(|s| Expr {
        //         info: None,
        //         kind: ExprKind::Symbol(s),
        //       })
        //       .collect::<Vec<_>>();

        //     context.stack_push(record.clone())?;
        //     context.stack_push(ExprKind::List(result).into())?;

        //     Ok(())
        //   }
        //   _ => context.stack_push(ExprKind::Nil.into()),
        // }
        // .map(|_| context)

        todo!()
      }
      // MARK: Values
      Self::Values => {
        // let record = context.stack_pop(&expr)?;

        // match record.kind {
        //   ExprKind::Record(ref r) => {
        //     let result = r.values().cloned().collect::<Vec<_>>();

        //     context.stack_push(record.clone())?;
        //     context.stack_push(ExprKind::List(result).into())?;

        //     Ok(())
        //   }
        //   _ => context.stack_push(ExprKind::Nil.into()),
        // }
        // .map(|_| context)

        todo!()
      }

      // MARK: Cast
      Self::Cast => {
        // let ty = context.stack_pop(&expr)?;
        // let item = context.stack_pop(&expr)?;

        // // TODO: Can these eager clones be removed?
        // let kind = match ty.kind {
        //   ExprKind::String(ref x) => match (item.kind.clone(), x.as_str()) {
        //     (ExprKind::Nil, "boolean") => ExprKind::Boolean(false),
        //     (ExprKind::Boolean(x), "boolean") => ExprKind::Boolean(x),
        //     (ExprKind::Integer(x), "boolean") => ExprKind::Boolean(x != 0),
        //     (ExprKind::Float(x), "boolean") => ExprKind::Boolean(x == 0.0),

        //     (ExprKind::Nil, "integer") => ExprKind::Integer(0),
        //     (ExprKind::Boolean(x), "integer") => ExprKind::Integer(x as i64),
        //     (ExprKind::Integer(x), "integer") => ExprKind::Integer(x),
        //     (ExprKind::Float(x), "integer") => {
        //       let x = x.floor();

        //       match x.classify() {
        //         FpCategory::Zero => ExprKind::Integer(0),
        //         FpCategory::Normal
        //           if x >= i64::MIN as f64 && x <= i64::MAX as f64 =>
        //         {
        //           ExprKind::Integer(x as i64)
        //         }
        //         _ => ExprKind::Nil,
        //       }
        //     }

        //     (ExprKind::Nil, "float") => ExprKind::Float(0.0),
        //     (ExprKind::Boolean(x), "float") => ExprKind::Float(x as i64 as f64),
        //     (ExprKind::Integer(x), "float") => ExprKind::Float(x as f64),
        //     (ExprKind::Float(x), "float") => ExprKind::Float(x),

        //     (ExprKind::Nil, "string") => ExprKind::String("nil".into()),
        //     (ExprKind::Boolean(x), "string") => {
        //       ExprKind::String(x.to_compact_string())
        //     }
        //     (ExprKind::Integer(x), "string") => {
        //       ExprKind::String(x.to_compact_string())
        //     }
        //     (ExprKind::Float(x), "string") => {
        //       ExprKind::String(x.to_compact_string())
        //     }
        //     (ExprKind::String(x), "string") => ExprKind::String(x),
        //     (ExprKind::Symbol(x), "string") => {
        //       ExprKind::String(x.as_str().into())
        //     }

        //     // TODO: Make sure these are correct, because the logic is pretty
        //     //       nuanced in terms of when to choose a Symbol or Intrinsic.
        //     (ExprKind::Nil, "symbol") => ExprKind::Nil,
        //     (ExprKind::Boolean(x), "symbol") => ExprKind::Boolean(x),
        //     // TODO: Handle conversion into `fn` and `fn!`.
        //     (ExprKind::String(x), "symbol") => ExprKind::Symbol(Symbol::new(x)),
        //     (ExprKind::Symbol(x), "symbol") => ExprKind::Symbol(x),

        //     (ExprKind::Record(x), "record") => ExprKind::Record(x),
        //     (ExprKind::Record(x), "list") => {
        //       let mut list: Vec<Expr> = Vec::new();
        //       x.into_iter().for_each(|(key, value)| {
        //         list.push(
        //           ExprKind::List(vec![ExprKind::Symbol(key).into(), value])
        //             .into(),
        //         );
        //       });

        //       ExprKind::List(list)
        //     }

        //     (ExprKind::List(x), "record") => {
        //       let mut record: HashMap<Symbol, Expr> = HashMap::new();
        //       x.into_iter().for_each(|item| {
        //         if let ExprKind::List(chunk) = item.kind {
        //           let key =
        //             Symbol::from_ref(chunk[0].kind.to_string().as_str());
        //           let value = &chunk[1];
        //           record.insert(key, value.clone());
        //         }
        //       });

        //       ExprKind::Record(record)
        //     }

        //     _ => ExprKind::Nil,
        //   },
        //   _ => ExprKind::Nil,
        // };

        // context.stack_push(kind.into())?;

        // Ok(context)

        todo!()
      }
      // MARK: TypeOf
      Self::TypeOf => {
        // let expr = context.stack_pop(&expr)?;

        // context
        //   .stack_push(ExprKind::String(expr.kind.type_of().into()).into())?;

        // Ok(context)

        todo!()
      }

      // MARK: Lazy
      Self::Lazy => {
        // let expr = context.stack_pop(&expr)?;

        // context.stack_push(ExprKind::Lazy(Box::new(expr)).into())?;

        // Ok(context)

        todo!()
      }

      // MARK: If
      Self::If => {
        // let body = context.stack_pop(&expr)?;
        // let cond = context.stack_pop(&expr)?;

        // if cond.kind.is_truthy() {
        //   context = engine.call_expr(context, body)?;
        // }

        // Ok(context)

        todo!()
      }
      // MARK: Halt
      Self::Halt => Err(VMError::Halt),

      // MARK: Call
      Self::Call => {
        // let item = context.stack_pop(&expr)?;
        // engine.call_expr(context, item)

        todo!()
      }

      // MARK: Let
      Self::Let => {
        // let names = context.stack_pop(&expr)?;
        // let body = context.stack_pop(&expr)?;

        // match names.kind {
        //   ExprKind::List(x) => {
        //     let x_len = x.len();

        //     let n = x.into_iter().try_fold(
        //       Vec::with_capacity(x_len),
        //       |mut v, x| match x.kind {
        //         ExprKind::Symbol(x) => {
        //           v.push(x);
        //           Ok(v)
        //         }
        //         _ => Err(RunError {
        //           reason: RunErrorReason::InvalidLet,
        //           context: context.clone(),
        //           expr: expr.clone(),
        //         }),
        //       },
        //     )?;

        //     let mut scope = context.scope().duplicate();
        //     for name in n.into_iter().rev() {
        //       let expr = context.stack_pop(&expr)?;
        //       scope.define(name, expr);
        //     }

        //     if let Some(journal) = context.journal_mut() {
        //       journal.commit();
        //       journal.push_op(JournalOp::ScopelessFnStart(expr.info.clone()));
        //     }

        //     context.push_scope(scope);
        //     context = engine.call_expr(context, body)?;

        //     if context.journal().is_some() {
        //       let scope = context.scope().clone();
        //       let journal = context.journal_mut().as_mut().unwrap();
        //       journal.commit();
        //       journal
        //         .push_op(JournalOp::FnEnd(expr.info.clone(), scope.into()));
        //     }

        //     context.pop_scope();

        //     Ok(context)
        //   }
        //   _ => Err(RunError {
        //     reason: RunErrorReason::InvalidLet,
        //     context: context.clone(),
        //     expr: expr.clone(),
        //   }),
        // }

        todo!()
      }

      // MARK: Def
      Self::Def => {
        // let name = context.stack_pop(&expr)?;
        // let value = context.stack_pop(&expr)?;

        // match name.kind {
        //   ExprKind::Symbol(symbol) => {
        //     context.def_scope_item(symbol, value);

        //     Ok(context)
        //   }
        //   _ => Err(RunError {
        //     reason: RunErrorReason::InvalidDefinition,
        //     context: context.clone(),
        //     expr: expr.clone(),
        //   }),
        // }

        todo!()
      }

      // MARK: Set
      Self::Set => {
        // let name = context.stack_pop(&expr)?;
        // let value = context.stack_pop(&expr)?;

        // match name.kind {
        //   ExprKind::Symbol(symbol) => {
        //     context.set_scope_item(symbol, value).map(|_| context)
        //   }
        //   _ => Err(RunError {
        //     reason: RunErrorReason::InvalidDefinition,
        //     context: context.clone(),
        //     expr: expr.clone(),
        //   }),
        // }

        todo!()
      }

      // MARK: Get
      Self::Get => {
        // let name = context.stack_pop(&expr)?;

        // match name.kind {
        //   ExprKind::Symbol(symbol) => {
        //     // Lets take precedence over scoped vars
        //     let item = context.scope_item(symbol);

        //     context
        //       .stack_push(item.ok_or_else(|| RunError {
        //         context: context.clone(),
        //         expr,
        //         reason: RunErrorReason::UnknownCall,
        //       })?)
        //       .map(|_| context)
        //   }
        //   _ => Err(RunError {
        //     reason: RunErrorReason::UnknownCall,
        //     context: context.clone(),
        //     expr: expr.clone(),
        //   }),
        // }

        todo!()
      }

      // MARK: Debug
      Self::Debug => {
        // if let Some(debug_hook) = engine.debug_hook() {
        //   debug_hook(
        //     context
        //       .stack()
        //       .last()
        //       .cloned()
        //       .unwrap_or(Expr {
        //         kind: ExprKind::Nil,
        //         info: None,
        //       })
        //       .to_string(),
        //   );
        // }
        // Ok(context)

        todo!()
      }
      // MARK: Print
      Self::Print => {
        // let val = context.stack_pop(&expr)?;

        // println!("{}", val);

        // Ok(context)

        todo!()
      }
      // MARK: Pretty
      Self::Pretty => {
        // let val = context.stack_pop(&expr)?;

        // println!("{:#}", val);

        // Ok(context)

        todo!()
      }
      // MARK: Recur
      // Functionality is implemented in [`Engine::call_fn`]
      Self::Recur => {
        // context
        //   .stack_push(ExprKind::Symbol(Symbol::from_ref("recur")).into())?;

        // Ok(context)

        todo!()
      }

      // MARK: OrElse
      Self::OrElse => {
        // let rhs = context.stack_pop(&expr)?;
        // let lhs = context.stack_pop(&expr)?;

        // match lhs.kind {
        //   ExprKind::Nil => context.stack_push(rhs)?,
        //   _ => context.stack_push(lhs)?,
        // }

        // Ok(context)

        todo!()
      }

      // MARK: Import
      Self::Import => {
        // let path = context.stack_pop(&expr)?;

        // // Imports should trigger a new commit
        // if let Some(journal) = context.journal_mut() {
        //   journal.commit();
        //   journal.push_op(JournalOp::ScopelessFnStart(expr.info.clone()));
        // }

        // match path.kind {
        //   ExprKind::String(str) => {
        //     if let Ok(source) = Source::from_path(str.as_str()) {
        //       context.add_source(source.clone());
        //       let mut lexer = Lexer::new(source);
        //       if let Ok(exprs) = parse(&mut lexer) {
        //         let mut result = engine.run(context, exprs);

        //         if let Ok(ref mut context) = result {
        //           if context.journal().is_some() {
        //             let scope = context.scope().clone();
        //             let journal = context.journal_mut().as_mut().unwrap();
        //             journal.commit();
        //             journal.push_op(JournalOp::FnEnd(
        //               expr.info.clone(),
        //               scope.into(),
        //             ));
        //           }
        //         }

        //         return result;
        //       }
        //     }
        //   }
        //   _ => {
        //     todo!()
        //   }
        // }

        // Ok(context)

        todo!()
      }
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParseIntrinsicError;

impl std::error::Error for ParseIntrinsicError {}

impl fmt::Display for ParseIntrinsicError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "unknown intrinsic")
  }
}

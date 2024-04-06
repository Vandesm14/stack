use core::cell::RefCell;
use std::{collections::HashMap, rc::Rc};

use lasso::Spur;

use crate::{interner::interner, EvalError, Expr, ExprKind, Program};

pub fn module(program: &mut Program) -> Result<(), EvalError> {
  program.funcs.insert(
    interner().get_or_intern_static("map/new"),
    |program, _| {
      program.push(
        ExprKind::UserData(Rc::new(RefCell::new(HashMap::<Spur, Expr>::new())))
          .into_expr(),
      )?;
      Ok(())
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("map/insert"),
    |program, trace_expr| {
      let key = program.pop(trace_expr)?;
      let item = program.pop(trace_expr)?;
      let map = program.pop(trace_expr)?;

      program.push(map.clone())?;

      match map.val {
        ExprKind::UserData(map) => {
          match map.borrow_mut().downcast_mut::<HashMap<Spur, Expr>>() {
            Some(map) => match key.val {
              ExprKind::Call(key) | ExprKind::String(key) => {
                map.insert(key, item);
              }
              found => {
                return Err(EvalError {
                  program: program.clone(),
                  expr: trace_expr.clone(),
                  message: format!(
                    "expected call or string, found {}",
                    found.type_of()
                  ),
                })
              }
            },
            None => {
              return Err(EvalError {
                program: program.clone(),
                expr: trace_expr.clone(),
                message: "unable to downcast userdata into map".into(),
              })
            }
          }
        }
        found => {
          return Err(EvalError {
            program: program.clone(),
            expr: trace_expr.clone(),
            message: format!("expected userdata, found {}", found.type_of()),
          })
        }
      }

      Ok(())
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("map/remove"),
    |program, trace_expr| {
      let key = program.pop(trace_expr)?;
      let map = program.pop(trace_expr)?;

      program.push(map.clone())?;

      match map.val {
        ExprKind::UserData(map) => {
          match map.borrow_mut().downcast_mut::<HashMap<Spur, Expr>>() {
            Some(map) => match key.val {
              ExprKind::Call(ref key) | ExprKind::String(ref key) => {
                map.remove(key);
              }
              found => {
                return Err(EvalError {
                  program: program.clone(),
                  expr: trace_expr.clone(),
                  message: format!(
                    "expected call or string, found {}",
                    found.type_of()
                  ),
                })
              }
            },
            None => {
              return Err(EvalError {
                program: program.clone(),
                expr: trace_expr.clone(),
                message: "unable to downcast userdata into map".into(),
              })
            }
          }
        }
        found => {
          return Err(EvalError {
            program: program.clone(),
            expr: trace_expr.clone(),
            message: format!("expected userdata, found {}", found.type_of()),
          })
        }
      }

      Ok(())
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("map/get"),
    |program, trace_expr| {
      let key = program.pop(trace_expr)?;
      let map = program.pop(trace_expr)?;

      program.push(map.clone())?;

      match map.val {
        ExprKind::UserData(map) => {
          match map.borrow().downcast_ref::<HashMap<Spur, Expr>>() {
            Some(map) => match key.val {
              ExprKind::Call(ref key) | ExprKind::String(ref key) => {
                program.push(
                  map.get(key).cloned().unwrap_or(ExprKind::Nil.into_expr()),
                )?;
              }
              found => {
                return Err(EvalError {
                  program: program.clone(),
                  expr: trace_expr.clone(),
                  message: format!(
                    "expected call or string, found {}",
                    found.type_of()
                  ),
                })
              }
            },
            None => {
              return Err(EvalError {
                program: program.clone(),
                expr: trace_expr.clone(),
                message: "unable to downcast userdata into map".into(),
              })
            }
          }
        }
        found => {
          return Err(EvalError {
            program: program.clone(),
            expr: trace_expr.clone(),
            message: format!("expected userdata, found {}", found.type_of()),
          })
        }
      }

      Ok(())
    },
  );

  Ok(())
}

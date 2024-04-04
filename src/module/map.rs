use core::cell::RefCell;
use std::{collections::HashMap, rc::Rc};

use lasso::Spur;

use crate::{interner::interner, Ast, AstIndex, EvalError, Expr, Program};

pub fn module(program: &mut Program) -> Result<(), EvalError> {
  program.funcs.insert(
    interner().get_or_intern_static("map/new"),
    |program, _| {
      program.push_expr(Expr::UserData(Rc::new(RefCell::new(HashMap::<
        Spur,
        Expr,
      >::new(
      )))))?;
      Ok(())
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("map/insert"),
    |program, trace_expr| {
      let key = program.pop_expr(trace_expr)?;
      let item = program.pop(trace_expr)?;
      let (map, map_index) = program.pop_with_index(trace_expr)?;

      program.push(map_index)?;

      match map {
        Expr::UserData(map) => {
          match map.borrow_mut().downcast_mut::<HashMap<Spur, AstIndex>>() {
            Some(map) => match key {
              Expr::Call(key) | Expr::String(key) => {
                map.insert(*key, item);
              }
              found => {
                return Err(EvalError {
                  program: program.clone(),
                  expr: trace_expr.clone(),
                  message: format!(
                    "expected call or string, found {}",
                    found.type_of(&program.ast)
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
            message: format!(
              "expected userdata, found {}",
              found.type_of(&program.ast)
            ),
          })
        }
      }

      Ok(())
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("map/remove"),
    |program, trace_expr| {
      let key = program.pop_expr(trace_expr)?;
      let (map, map_index) = program.pop_with_index(trace_expr)?;

      program.push(map_index)?;

      match map {
        Expr::UserData(map) => {
          match map.borrow_mut().downcast_mut::<HashMap<Spur, AstIndex>>() {
            Some(map) => match key {
              Expr::Call(ref key) | Expr::String(ref key) => {
                map.remove(key);
              }
              found => {
                return Err(EvalError {
                  program: program.clone(),
                  expr: trace_expr.clone(),
                  message: format!(
                    "expected call or string, found {}",
                    found.type_of(&program.ast)
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
            message: format!(
              "expected userdata, found {}",
              found.type_of(&program.ast)
            ),
          })
        }
      }

      Ok(())
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("map/get"),
    |program, trace_expr| {
      let key = program.pop_expr(trace_expr)?;
      let (map, map_index) = program.pop_with_index(trace_expr)?;

      program.push(map_index)?;

      match map {
        Expr::UserData(map) => {
          match map.borrow().downcast_ref::<HashMap<Spur, AstIndex>>() {
            Some(map) => match key {
              Expr::Call(ref key) | Expr::String(ref key) => {
                program.push(map.get(key).cloned().unwrap_or(Ast::NIL))?;
              }
              found => {
                return Err(EvalError {
                  program: program.clone(),
                  expr: trace_expr.clone(),
                  message: format!(
                    "expected call or string, found {}",
                    found.type_of(&program.ast)
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
            message: format!(
              "expected userdata, found {}",
              found.type_of(&program.ast)
            ),
          })
        }
      }

      Ok(())
    },
  );

  Ok(())
}

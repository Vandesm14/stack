use stack_core::prelude::*;
use std::{collections::HashMap, mem, rc::Rc, sync::Mutex};

use serde::{Deserialize, Serialize};
use ws::Message;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Incoming {
  Run(RunPayload),
  RunNew(RunPayload),
  Stack(BasePayload),
  Scope(BasePayload),
  ClearStack(BasePayload),
  ClearScope(BasePayload),
  Clear(BasePayload),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RunPayload {
  pub id: u32,
  pub code: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BasePayload {
  pub id: u32,
}

impl Incoming {
  pub fn id(&self) -> u32 {
    match self {
      Incoming::Run(payload) | Incoming::RunNew(payload) => payload.id,
      Incoming::Stack(payload)
      | Incoming::Scope(payload)
      | Incoming::ClearStack(payload)
      | Incoming::ClearScope(payload)
      | Incoming::Clear(payload) => payload.id,
    }
  }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "error", content = "value", rename_all = "lowercase")]
pub enum OutgoingError {
  /// Error from the Engine
  #[serde(rename = "run_error")]
  RunError(RunError),

  /// Error from the command reader
  #[serde(rename = "command_error")]
  CommandError(String),
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", content = "value", rename_all = "lowercase")]
pub enum Outgoing {
  /// A Expr
  #[serde(rename = "ok")]
  Single(Expr),

  /// A Null Response
  #[serde(rename = "ok")]
  Null(()),

  /// A Vec of Exprs
  #[serde(rename = "ok")]
  Many(Vec<Expr>),

  /// A Map of Exprs
  #[serde(rename = "ok")]
  Map(HashMap<String, Expr>),

  /// An Error
  Error(OutgoingError),
}

pub fn listen() {
  let eng_mutex = Rc::new(Mutex::new(Engine::new()));
  let ctx_mutex = Rc::new(Mutex::new(Context::new()));

  ws::listen("127.0.0.1:5001", |out| {
    let eng_mutex = eng_mutex.clone();
    let ctx_mutex = ctx_mutex.clone();

    move |msg| {
      if let Message::Text(string) = msg {
        let request = serde_json::from_str::<Incoming>(&string);

        match request {
          Ok(incoming) => match incoming {
            Incoming::RunNew(RunPayload { id, code }) => {
              let source = Source::new("runner", code);
              let mut lexer = Lexer::new(source);
              let exprs = parse(&mut lexer).unwrap();

              match (eng_mutex.try_lock(), ctx_mutex.try_lock()) {
                (Ok(engine), Ok(mut guard)) => {
                  let _ = mem::replace(&mut *guard, Context::new());

                  let context = mem::take(&mut *guard);
                  let result = engine.run(context, exprs);

                  match result {
                    Ok(ctx) => {
                      *guard = ctx;

                      match guard.stack().last().cloned() {
                        Some(expr) => out.send(
                          serde_json::to_string(&Outgoing::Single(expr))
                            .unwrap(),
                        ),
                        None => out.send(
                          serde_json::to_string(&Outgoing::Null(())).unwrap(),
                        ),
                      }
                    }
                    Err(error) => out.send(
                      serde_json::to_string(&Outgoing::Error(
                        OutgoingError::RunError(error),
                      ))
                      .unwrap(),
                    ),
                  }
                }
                _ => todo!("mutex not lock"),
              }
            }
            Incoming::Run(RunPayload { id, code }) => {
              let source = Source::new("runner", code);
              let mut lexer = Lexer::new(source);
              let exprs = parse(&mut lexer).unwrap();

              match (eng_mutex.try_lock(), ctx_mutex.try_lock()) {
                (Ok(engine), Ok(mut guard)) => {
                  let context = mem::take(&mut *guard);
                  let result = engine.run(context, exprs);

                  match result {
                    Ok(ctx) => {
                      *guard = ctx;

                      let expr = guard
                        .stack()
                        .last()
                        .cloned()
                        .unwrap_or_else(|| ExprKind::Nil.into());

                      out.send(
                        serde_json::to_string(&Outgoing::Single(expr)).unwrap(),
                      )
                    }
                    Err(error) => out.send(
                      serde_json::to_string(&Outgoing::Error(
                        OutgoingError::RunError(error),
                      ))
                      .unwrap(),
                    ),
                  }
                }
                _ => todo!("mutex not lock"),
              }
            }

            Incoming::Stack(BasePayload { id }) => match ctx_mutex.try_lock() {
              Ok(context) => out.send(
                serde_json::to_string(&Outgoing::Many(
                  context.stack().to_vec(),
                ))
                .unwrap(),
              ),
              Err(_) => todo!(),
            },
            Incoming::Scope(BasePayload { id }) => match ctx_mutex.try_lock() {
              Ok(context) => {
                let mut scope: HashMap<String, Expr> = HashMap::new();
                for (k, v) in context.scope().items.iter() {
                  scope
                    .insert(k.to_string(), v.borrow().val().clone().unwrap());
                }

                out.send(serde_json::to_string(&Outgoing::Map(scope)).unwrap())
              }
              Err(_) => todo!(),
            },

            Incoming::ClearStack(BasePayload { id }) => todo!(),
            Incoming::ClearScope(BasePayload { id }) => todo!(),
            Incoming::Clear(BasePayload { id }) => todo!(),
          },
          Err(parse_error) => out.send(
            serde_json::to_string(&Outgoing::Error(
              OutgoingError::CommandError(parse_error.to_string()),
            ))
            .unwrap(),
          ),
        }
      } else {
        todo!("message not text")
      }
    }
  })
  .unwrap();
}

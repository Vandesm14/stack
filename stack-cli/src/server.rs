use stack_core::prelude::*;
use std::{collections::HashMap, mem, rc::Rc, sync::Mutex};

use serde::{Deserialize, Serialize};
use ws::Message;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Incoming {
  // Execution
  Run(RunPayload),
  RunNew(RunPayload),

  // Querying
  Stack(BasePayload),
  Scope(BasePayload),
  Context(BasePayload),
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
      | Incoming::Context(payload) => payload.id,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum OutgoingError {
  RunError(RunErrorPayload),
  CommandError(CommandErrorPayload),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunErrorPayload {
  pub for_id: u32,
  pub value: RunError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandErrorPayload {
  pub for_id: u32,
  pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Outgoing {
  Ok(OkPayload),
  Error(OutgoingError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OkPayload {
  Single(SinglePayload),
  Null(NullPayload),
  Many(ManyPayload),
  Map(MapPayload),
  Context(ContextPayload),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SinglePayload {
  pub for_id: u32,
  pub value: Expr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NullPayload {
  pub for_id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManyPayload {
  pub for_id: u32,
  pub value: Vec<Expr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapPayload {
  pub for_id: u32,
  pub value: HashMap<String, Expr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPayload {
  pub for_id: u32,
  pub value: Context,
}

impl Outgoing {
  pub fn for_id(&self) -> u32 {
    match self {
      Outgoing::Ok(payload) => match payload {
        OkPayload::Single(p) => p.for_id,
        OkPayload::Null(p) => p.for_id,
        OkPayload::Many(p) => p.for_id,
        OkPayload::Map(p) => p.for_id,
        OkPayload::Context(p) => p.for_id,
      },
      Outgoing::Error(error) => match error {
        OutgoingError::RunError(p) => p.for_id,
        OutgoingError::CommandError(p) => p.for_id,
      },
    }
  }
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
                          serde_json::to_string(&Outgoing::Ok(
                            OkPayload::Single(SinglePayload {
                              for_id: id,
                              value: expr,
                            }),
                          ))
                          .unwrap(),
                        ),
                        None => out.send(
                          serde_json::to_string(&Outgoing::Ok(
                            OkPayload::Null(NullPayload { for_id: id }),
                          ))
                          .unwrap(),
                        ),
                      }
                    }
                    Err(error) => out.send(
                      serde_json::to_string(&Outgoing::Error(
                        OutgoingError::RunError(RunErrorPayload {
                          for_id: id,
                          value: error,
                        }),
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

                      match guard.stack().last().cloned() {
                        Some(expr) => out.send(
                          serde_json::to_string(&Outgoing::Ok(
                            OkPayload::Single(SinglePayload {
                              for_id: id,
                              value: expr,
                            }),
                          ))
                          .unwrap(),
                        ),
                        None => out.send(
                          serde_json::to_string(&Outgoing::Ok(
                            OkPayload::Null(NullPayload { for_id: id }),
                          ))
                          .unwrap(),
                        ),
                      }
                    }
                    Err(error) => out.send(
                      serde_json::to_string(&Outgoing::Error(
                        OutgoingError::RunError(RunErrorPayload {
                          for_id: id,
                          value: error,
                        }),
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
                serde_json::to_string(&Outgoing::Ok(OkPayload::Many(
                  ManyPayload {
                    for_id: id,
                    value: context.stack().to_vec(),
                  },
                )))
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

                out.send(
                  serde_json::to_string(&Outgoing::Ok(OkPayload::Map(
                    MapPayload {
                      for_id: id,
                      value: scope,
                    },
                  )))
                  .unwrap(),
                )
              }
              Err(_) => todo!(),
            },
            Incoming::Context(BasePayload { id }) => {
              match ctx_mutex.try_lock() {
                Ok(context) => out.send(
                  serde_json::to_string(&Outgoing::Ok(OkPayload::Context(
                    ContextPayload {
                      for_id: id,
                      value: context.clone(),
                    },
                  )))
                  .unwrap(),
                ),
                Err(_) => todo!(),
              }
            }
          },
          Err(parse_error) => out.send(
            serde_json::to_string(&Outgoing::Error(
              OutgoingError::CommandError(CommandErrorPayload {
                // TODO: we don't get an ID here so this is a special case
                for_id: 0,
                value: parse_error.to_string(),
              }),
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

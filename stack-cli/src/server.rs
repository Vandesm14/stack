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
#[serde(tag = "error", rename_all = "snake_case")]
pub enum OutgoingError {
  RunError(RunErrorPayload),
  CommandError(CommandErrorPayload),
}

#[derive(Debug, Clone, Serialize)]
pub struct PublicContext {
  pub stack: Vec<Expr>,
  pub scopes: Vec<HashMap<String, Expr>>,
  pub sources: HashMap<String, Source>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PublicRunError {
  pub reason: RunErrorReason,
  pub context: Vec<Expr>,
  pub expr: Expr,
}

impl From<RunError> for PublicRunError {
  fn from(value: RunError) -> Self {
    Self {
      reason: value.reason,
      context: value.context.stack().to_vec(),
      expr: value.expr,
    }
  }
}

#[derive(Debug, Clone, Serialize)]
pub struct RunErrorPayload {
  pub for_id: u32,
  pub value: PublicRunError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandErrorPayload {
  pub for_id: u32,
  pub value: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Outgoing {
  Ok(OkPayload),
  Error(OutgoingError),
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OkPayload {
  Single(SinglePayload),
  Null(NullPayload),
  Many(ManyPayload),
  Map(MapPayload),
}

#[derive(Debug, Clone, Serialize)]
pub struct SinglePayload {
  pub for_id: u32,
  pub value: Expr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NullPayload {
  pub for_id: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ManyPayload {
  pub for_id: u32,
  pub value: Vec<Expr>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MapPayload {
  pub for_id: u32,
  pub value: HashMap<String, Expr>,
}

impl Outgoing {
  pub fn for_id(&self) -> u32 {
    match self {
      Outgoing::Ok(payload) => match payload {
        OkPayload::Single(p) => p.for_id,
        OkPayload::Null(p) => p.for_id,
        OkPayload::Many(p) => p.for_id,
        OkPayload::Map(p) => p.for_id,
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
                          value: error.into(),
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
                          value: error.into(),
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

            Incoming::ClearStack(BasePayload { id }) => todo!(),
            Incoming::ClearScope(BasePayload { id }) => todo!(),
            Incoming::Clear(BasePayload { id }) => todo!(),
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

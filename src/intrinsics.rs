use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Intrinsic {
  // Arithmetic
  Add,
  Subtract,
  Multiply,
  Divide,
  Remainder,

  // Comparison
  Equal,
  NotEqual,
  GreaterThan,
  LessThan,
  Or,
  And,

  // Code/IO
  Parse,
  ReadFile,
  Print,
  Syscall { arity: u8 },

  // List
  Explode,
  Length,
  Nth,
  Join,
  Insert,
  ListPop,
  ListShift,
  Concat,
  Unwrap,

  // Control Flow
  IfElse,
  If,
  While,
  Halt,

  // Scope
  Set,
  Get,
  Unset,

  // Stack
  Collect,
  Clear,
  Pop,
  Dup,
  Swap,
  Rot,

  // Functions/Data
  Call,
  CallNative,
  Lazy,
  Noop,

  // Type
  ToBoolean,
  ToInteger,
  ToFloat,
  ToPointer,
  ToList,
  ToString,
  ToCall,
  TypeOf,
}

impl TryFrom<&str> for Intrinsic {
  type Error = ();

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      // Arithmetic
      "+" => Ok(Self::Add),
      "-" => Ok(Self::Subtract),
      "*" => Ok(Self::Multiply),
      "/" => Ok(Self::Divide),
      "%" => Ok(Self::Remainder),

      // Comparison
      "=" => Ok(Self::Equal),
      "!=" => Ok(Self::NotEqual),
      ">" => Ok(Self::GreaterThan),
      "<" => Ok(Self::LessThan),
      "or" => Ok(Self::Or),
      "and" => Ok(Self::And),

      // Code/IO
      "parse" => Ok(Self::Parse),
      "read-file" => Ok(Self::ReadFile),
      "print" => Ok(Self::Print),
      "syscall0" => Ok(Self::Syscall { arity: 0 }),
      "syscall1" => Ok(Self::Syscall { arity: 1 }),
      "syscall2" => Ok(Self::Syscall { arity: 2 }),
      "syscall3" => Ok(Self::Syscall { arity: 3 }),
      "syscall4" => Ok(Self::Syscall { arity: 4 }),
      "syscall5" => Ok(Self::Syscall { arity: 5 }),
      "syscall6" => Ok(Self::Syscall { arity: 6 }),

      // List
      "explode" => Ok(Self::Explode),
      "len" => Ok(Self::Length),
      "nth" => Ok(Self::Nth),
      "join" => Ok(Self::Join),
      "insert" => Ok(Self::Insert),
      "list-pop" => Ok(Self::ListPop),
      "list-shift" => Ok(Self::ListShift),
      "concat" => Ok(Self::Concat),
      "unwrap" => Ok(Self::Unwrap),

      // Control Flow
      "ifelse" => Ok(Self::IfElse),
      "if" => Ok(Self::If),
      "while" => Ok(Self::While),
      "halt" => Ok(Self::Halt),

      // Scope
      "set" => Ok(Self::Set),
      "get" => Ok(Self::Get),
      "unset" => Ok(Self::Unset),

      // Stack
      "collect" => Ok(Self::Collect),
      "clear" => Ok(Self::Clear),
      "pop" => Ok(Self::Pop),
      "dup" => Ok(Self::Dup),
      "swap" => Ok(Self::Swap),
      "rot" => Ok(Self::Rot),

      // Functions/Data
      "call" => Ok(Self::Call),
      "call_native" => Ok(Self::CallNative),
      "lazy" => Ok(Self::Lazy),

      // Type
      "toboolean" => Ok(Self::ToBoolean),
      "tointeger" => Ok(Self::ToInteger),
      "tofloat" => Ok(Self::ToFloat),
      "topointer" => Ok(Self::ToPointer),
      "tolist" => Ok(Self::ToList),
      "tostring" => Ok(Self::ToString),
      "tocall" => Ok(Self::ToCall),
      "typeof" => Ok(Self::TypeOf),

      _ => Err(()),
    }
  }
}

impl fmt::Display for Intrinsic {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.as_str())
  }
}

impl Intrinsic {
  pub fn as_str(self) -> &'static str {
    match self {
      // Arithmetic
      Self::Add => "+",
      Self::Subtract => "-",
      Self::Multiply => "*",
      Self::Divide => "/",
      Self::Remainder => "%",

      // Comparison
      Self::Equal => "=",
      Self::NotEqual => "!=",
      Self::GreaterThan => ">",
      Self::LessThan => "<",
      Self::Or => "or",
      Self::And => "and",

      // Code/IO
      Self::Parse => "parse",
      Self::ReadFile => "read-file",
      Self::Print => "print",
      Self::Syscall { arity } => match arity {
        0 => "syscall0",
        1 => "syscall1",
        2 => "syscall2",
        3 => "syscall3",
        4 => "syscall4",
        5 => "syscall5",
        6 => "syscall6",
        arity => unimplemented!("invalid syscall arity: {arity}"),
      },

      // List
      Self::Explode => "explode",
      Self::Length => "len",
      Self::Nth => "nth",
      Self::Join => "join",
      Self::Insert => "insert",
      Self::ListPop => "list-pop",
      Self::ListShift => "list-shift",
      Self::Concat => "concat",
      Self::Unwrap => "unwrap",

      // Control Flow
      Self::IfElse => "ifelse",
      Self::If => "if",
      Self::While => "while",
      Self::Halt => "halt",

      // Scope
      Self::Set => "set",
      Self::Get => "get",
      Self::Unset => "unset",

      // Stack
      Self::Collect => "collect",
      Self::Clear => "clear",
      Self::Pop => "pop",
      Self::Dup => "dup",
      Self::Swap => "swap",
      Self::Rot => "rot",

      // Functions/Data
      Self::Call => "call",
      Self::CallNative => "call_native",
      Self::Lazy => "lazy",
      Self::Noop => "noop",

      // Type
      Self::ToBoolean => "toboolean",
      Self::ToInteger => "tointeger",
      Self::ToFloat => "tofloat",
      Self::ToPointer => "topointer",
      Self::ToList => "tolist",
      Self::ToString => "tostring",
      Self::ToCall => "tocall",
      Self::TypeOf => "typeof",
    }
  }
}

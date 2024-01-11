use std::fs;

use enum_iterator::all;
use stack::Intrinsic;

fn main() {
  let string = include_str!("template.tmLanguage.json");
  // let items = all::<Intrinsic>().join("|");
  let mut items: Vec<String> = Vec::new();

  for intrinsic in all::<Intrinsic>() {
    if let Intrinsic::Syscall { arity } = intrinsic {
      if arity > 6 {
        continue;
      }
    }

    items.push(intrinsic.as_str().to_string());
  }

  let items = items.join("|");

  let string = string.replace("${template}", &items);

  fs::write("theme.tmLanguage.json", string).unwrap();
}

use std::fs;

use stack::Program;

#[test]
fn run_tests() {
  let result = fs::read_dir("tests").unwrap();

  for entry in result {
    let entry = entry.unwrap();
    let path = entry.path();

    if let Some(name) = path.file_name() {
      if name.to_str().unwrap().ends_with(".stack") {
        let contents = fs::read_to_string(&path).unwrap();

        let mut program = Program::new()
          .with_core()
          // .unwrap()
          // .with_module(map::module)
          .unwrap();
        let result = program.eval_string(contents.as_str());

        assert!(
          result.is_ok(),
          "{}: {}",
          name.to_string_lossy(),
          result.err().unwrap()
        );
      }
    }
  }
}

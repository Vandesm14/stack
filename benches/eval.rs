use criterion::{criterion_group, criterion_main, Criterion};
use stack::Program;

fn cold_start_with_core() {
  Program::new().with_core().unwrap();
}

fn running_code() {
  let mut program = Program::new().with_core().unwrap();

  program
    .eval_string(
      "
  '(fn
    0 'i def
  
    '(fn
      i 1 + 'i set
    ) 'inc def
  
    '(fn i) 'value def
  
    '()
    'inc export
    'value export
  ) 'counter def
  
  counter 'my-counter use
  
  my-counter/inc
  my-counter/inc
  my-counter/inc
  my-counter/value",
    )
    .unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
  c.bench_function("cold start with core", |b| {
    b.iter(|| cold_start_with_core())
  });

  c.bench_function("running code", |b| b.iter(|| running_code()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

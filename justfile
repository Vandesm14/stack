run file:
  cargo run -p stack-cli -- --enable-all run {{file}}

watch file:
  cargo run -p stack-cli -- --enable-all run --watch {{file}}

debug file:
  cargo run -p stack-debugger --release -- --enable-all {{file}}

serve:
  cd docs; mdbook serve --open

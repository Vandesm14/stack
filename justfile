run file:
  cargo run -p stack-cli -- --enable-all run {{file}}

watch file:
  cargo run -p stack-cli -- --enable-all run --watch {{file}}

debug file:
  cargo run -p stack-debugger --release -- --enable-all {{file}}

serve:
  cd docs; mdbook serve --open

ws-serve:
  cargo run -p stack-cli -- serve

ws-connect:
  rlwrap websocat ws://localhost:5001

watch file:
    cargo run -p stack-cli -- --enable-all run --watch {{file}}

debug file:
    cargo run -p stack-debugger --release -- --enable-all {{file}}

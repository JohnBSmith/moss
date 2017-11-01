
# cargo check
cargo build
# cargo build --release
# cargo rustc --release --bin moss -- -C prefer-dynamic

# cargo rustc --release --bin moss --\
#   -C opt-level=3 -C lto -C relocation-model=static

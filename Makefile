.PHONY: build

# Optimized release binary: target/release/patch-builder.exe (profile in Cargo.toml).
build:
	cargo build --release

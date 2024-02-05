fmt:
	cargo +nightly fmt

lint: fmt
	cargo clippy --tests --workspace -- -D warnings

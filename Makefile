all: build
build:
	cargo build --release
test:
	cargo test --tests
cov:
	cargo llvm-cov

fmt:
	cargo fmt --all

test:
	cargo test

check:
	cargo check

cli:
	cargo run -p proxio -- --help

ui:
	cargo run -p proxio-ui

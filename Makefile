fmt-sync:
	cp humidity-monitor/rustfmt.toml humidity-core/
	cp humidity-monitor/rustfmt.toml ble-client/

fmt:
	cd humidity-core && cargo +nightly fmt
	cd humidity-monitor && cargo +nightly fmt
	cd ble-client && cargo +nightly fmt

check:
	cd humidity-core && cargo check
	cd humidity-monitor && cargo check
	cd ble-client && cargo check

clippy:
	cd humidity-core && cargo clippy

run-client:
	cargo run --manifest-path ./ble-client/Cargo.toml --release

run-s3:
	cargo run --manifest-path ./humidity-monitor/Cargo.toml --release

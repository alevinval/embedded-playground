fmt-sync:
	cp humidity-monitor/rustfmt.toml humidity-core/
	cp humidity-monitor/rustfmt.toml humidity-monitor-c6/
	cp humidity-monitor/rustfmt.toml ble-client/

fmt:
	cd humidity-core && cargo +nightly fmt
	cd humidity-monitor && cargo +nightly fmt
	cd humidity-monitor-c6 && cargo +nightly fmt
	cd ble-client && cargo +nightly fmt

check:
	cd humidity-core && cargo check
	cd humidity-monitor && cargo check
	cd humidity-monitor-c6 && cargo check
	cd ble-client && cargo check

clippy:
	cd humidity-core && cargo clippy
	cd humidity-monitor && cargo clippy
	cd humidity-monitor-c6 && cargo clippy

doc:
	cargo doc --manifest-path ./humidity-core/Cargo.toml --no-deps --open

run-client:
	cargo run --manifest-path ./ble-client/Cargo.toml --release

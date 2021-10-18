
override ENV += RUSTFLAGS="-Ctarget-cpu=native"

.PHONY: all build
all build:
	$(ENV) cargo build

.PHONY: test
test:
	$(ENV) cargo test --lib
	$(ENV) cargo run --example crc
	$(ENV) cargo run --example shamir
	$(ENV) cargo run --example raid

.PHONY: bench
bench:
	$(ENV) cargo +nightly bench --features use-nightly-features

.PHONY: clean
clean:
	$(ENV) cargo clean

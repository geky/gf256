
override ENV += RUSTFLAGS="-Ctarget-cpu=native"

.PHONY: all build
all build:
	$(ENV) cargo +nightly build --features nightly

.PHONY: test-targets
test-targets:
	$(ENV) cargo test --lib
	$(ENV) cargo test --features no-xmul,crc --lib
	$(ENV) cargo test --features crc --lib
	$(ENV) cargo +nightly test --features nightly,crc --lib

.PHONY: test
test:
	$(ENV) cargo +nightly test --features nightly,crc --lib
	$(ENV) cargo +nightly test --features nightly,crc --example find-p
	$(ENV) cargo +nightly run --features nightly,crc --example find-p -- -w9 -n4 -m1
	$(ENV) cargo +nightly run --features nightly,crc --example lfsr
	$(ENV) cargo +nightly run --features nightly,crc --example crc
	$(ENV) cargo +nightly run --features nightly,crc --example shamir
	$(ENV) cargo +nightly run --features nightly,crc --example raid
	$(ENV) cargo +nightly run --features nightly,crc --example rs

.PHONY: bench
bench:
	$(ENV) cargo +nightly bench --features nightly,crc --bench xmul   -- --noplot
	$(ENV) cargo +nightly bench --features nightly,crc --bench gf     -- --noplot
	$(ENV) cargo +nightly bench --features nightly,crc --bench find-p -- --noplot
	$(ENV) cargo +nightly bench --features nightly,crc --bench lfsr   -- --noplot
	$(ENV) cargo +nightly bench --features nightly,crc --bench crc    -- --noplot
	$(ENV) cargo +nightly bench --features nightly,crc --bench shamir -- --noplot
	$(ENV) cargo +nightly bench --features nightly,crc --bench raid   -- --noplot
	$(ENV) cargo +nightly bench --features nightly,crc --bench rs     -- --noplot

.PHONY: clean
clean:
	$(ENV) cargo clean

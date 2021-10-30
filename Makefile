
override ENV += RUSTFLAGS="-Ctarget-cpu=native"

.PHONY: all build
all build:
	$(ENV) cargo +nightly build --features nightly

.PHONY: test-targets
test-targets:
	$(ENV) cargo test --features no-xmul --lib
	$(ENV) cargo test --lib
	$(ENV) cargo +nightly test --features nightly --lib

.PHONY: test
test:
	$(ENV) cargo +nightly test --features nightly --lib
	$(ENV) cargo +nightly test --features nightly --example find-p
	$(ENV) cargo +nightly run --features nightly --example find-p -- -w9 -n4 -m1
	$(ENV) cargo +nightly run --features nightly --example lfsr
	$(ENV) cargo +nightly run --features nightly --example crc
	$(ENV) cargo +nightly run --features nightly --example shamir
	$(ENV) cargo +nightly run --features nightly --example raid
	$(ENV) cargo +nightly run --features nightly --example rs

.PHONY: bench
bench:
	$(ENV) cargo +nightly bench --features nightly --bench xmul   -- --noplot
	$(ENV) cargo +nightly bench --features nightly --bench gf     -- --noplot
	$(ENV) cargo +nightly bench --features nightly --bench find-p -- --noplot
	$(ENV) cargo +nightly bench --features nightly --bench lfsr   -- --noplot
	$(ENV) cargo +nightly bench --features nightly --bench crc    -- --noplot
	$(ENV) cargo +nightly bench --features nightly --bench shamir -- --noplot
	$(ENV) cargo +nightly bench --features nightly --bench raid   -- --noplot
	$(ENV) cargo +nightly bench --features nightly --bench rs     -- --noplot

.PHONY: clean
clean:
	$(ENV) cargo clean

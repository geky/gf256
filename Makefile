
override ENV += RUSTFLAGS="-Ctarget-cpu=native"

.PHONY: all build
all build:
	$(ENV) cargo +nightly build --no-default-features --features nightly

.PHONY: test-features
test-features:
	$(ENV) cargo test --lib
	$(ENV) cargo test --features no-xmul,thread-rng,crc,shamir --lib
	$(ENV) cargo test --features thread-rng,crc,shamir --lib
	$(ENV) cargo +nightly test --features nightly,thread-rng,crc,shamir --lib

.PHONY: test
test:
	$(ENV) cargo +nightly test --features nightly,thread-rng,crc,shamir --lib
	$(ENV) cargo +nightly test --features nightly,thread-rng,crc,shamir --example find-p
	$(ENV) cargo +nightly run --features nightly,thread-rng,crc,shamir --example find-p -- -w9 -n4 -m1
	$(ENV) cargo +nightly run --features nightly,thread-rng,crc,shamir --example lfsr
	$(ENV) cargo +nightly run --features nightly,thread-rng,crc,shamir --example crc
	$(ENV) cargo +nightly run --features nightly,thread-rng,crc,shamir --example shamir
	$(ENV) cargo +nightly run --features nightly,thread-rng,crc,shamir --example raid
	$(ENV) cargo +nightly run --features nightly,thread-rng,crc,shamir --example rs

.PHONY: bench
bench:
	$(ENV) cargo +nightly bench --features nightly,thread-rng,crc,shamir --bench xmul   -- --noplot
	$(ENV) cargo +nightly bench --features nightly,thread-rng,crc,shamir --bench gf     -- --noplot
	$(ENV) cargo +nightly bench --features nightly,thread-rng,crc,shamir --bench find-p -- --noplot
	$(ENV) cargo +nightly bench --features nightly,thread-rng,crc,shamir --bench lfsr   -- --noplot
	$(ENV) cargo +nightly bench --features nightly,thread-rng,crc,shamir --bench crc    -- --noplot
	$(ENV) cargo +nightly bench --features nightly,thread-rng,crc,shamir --bench shamir -- --noplot
	$(ENV) cargo +nightly bench --features nightly,thread-rng,crc,shamir --bench raid   -- --noplot
	$(ENV) cargo +nightly bench --features nightly,thread-rng,crc,shamir --bench rs     -- --noplot

.PHONY: clean
clean:
	$(ENV) cargo clean


override ENV += RUSTFLAGS="-Ctarget-cpu=native"

.PHONY: all build
all build:
	$(ENV) cargo +nightly build --features nightly

.PHONY: test-features
test-features:
	$(ENV) cargo test --lib
	$(ENV) cargo test --features no-xmul,thread-rng,lfsr,crc,shamir --lib
	$(ENV) cargo test --features thread-rng,lfsr,crc,shamir --lib
	$(ENV) cargo +nightly test --features nightly,thread-rng,lfsr,crc,shamir --lib

.PHONY: test
test:
	$(ENV) cargo +nightly test --features nightly,thread-rng,lfsr,crc,shamir --lib
	$(ENV) cargo +nightly test --features nightly,thread-rng,lfsr,crc,shamir --example find-p
	$(ENV) cargo +nightly run --features nightly,thread-rng,lfsr,crc,shamir --example find-p -- -w9 -n4 -m1
	$(ENV) cargo +nightly run --features nightly,thread-rng,lfsr,crc,shamir --example lfsr
	$(ENV) cargo +nightly run --features nightly,thread-rng,lfsr,crc,shamir --example crc
	$(ENV) cargo +nightly run --features nightly,thread-rng,lfsr,crc,shamir --example shamir
	$(ENV) cargo +nightly run --features nightly,thread-rng,lfsr,crc,shamir --example raid
	$(ENV) cargo +nightly run --features nightly,thread-rng,lfsr,crc,shamir --example rs

.PHONY: bench
bench:
	$(ENV) cargo +nightly bench --features nightly,thread-rng,lfsr,crc,shamir --bench xmul   -- --noplot
	$(ENV) cargo +nightly bench --features nightly,thread-rng,lfsr,crc,shamir --bench gf     -- --noplot
	$(ENV) cargo +nightly bench --features nightly,thread-rng,lfsr,crc,shamir --bench find-p -- --noplot
	$(ENV) cargo +nightly bench --features nightly,thread-rng,lfsr,crc,shamir --bench lfsr   -- --noplot
	$(ENV) cargo +nightly bench --features nightly,thread-rng,lfsr,crc,shamir --bench crc    -- --noplot
	$(ENV) cargo +nightly bench --features nightly,thread-rng,lfsr,crc,shamir --bench shamir -- --noplot
	$(ENV) cargo +nightly bench --features nightly,thread-rng,lfsr,crc,shamir --bench raid   -- --noplot
	$(ENV) cargo +nightly bench --features nightly,thread-rng,lfsr,crc,shamir --bench rs     -- --noplot

.PHONY: clean
clean:
	$(ENV) cargo clean

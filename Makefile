
override ENV += RUSTFLAGS="-Ctarget-cpu=native"

.PHONY: all build
all build:
	$(ENV) cargo +nightly build --features nightly

.PHONY: test-features
test-features:
	$(ENV) cargo test --lib
	$(ENV) cargo test --features no-xmul,thread-rng,lfsr,crc,shamir,raid --lib
	$(ENV) cargo test --features thread-rng,lfsr,crc,shamir,raid --lib
	$(ENV) cargo +nightly test --features nightly,thread-rng,lfsr,crc,shamir,raid --lib

.PHONY: test
test:
	$(ENV) cargo +nightly test --features nightly,thread-rng,lfsr,crc,shamir,raid --lib
	$(ENV) cargo +nightly test --features nightly,thread-rng,lfsr,crc,shamir,raid --example find-p
	$(ENV) cargo +nightly run --features nightly,thread-rng,lfsr,crc,shamir,raid --example find-p -- -w9 -n4 -m1
	$(ENV) cargo +nightly run --features nightly,thread-rng,lfsr,crc,shamir,raid --example lfsr
	$(ENV) cargo +nightly run --features nightly,thread-rng,lfsr,crc,shamir,raid --example crc
	$(ENV) cargo +nightly run --features nightly,thread-rng,lfsr,crc,shamir,raid --example shamir
	$(ENV) cargo +nightly run --features nightly,thread-rng,lfsr,crc,shamir,raid --example raid
	$(ENV) cargo +nightly run --features nightly,thread-rng,lfsr,crc,shamir,raid --example rs

.PHONY: bench
bench:
	$(ENV) cargo +nightly bench --features nightly,thread-rng,lfsr,crc,shamir,raid --bench xmul   -- --noplot
	$(ENV) cargo +nightly bench --features nightly,thread-rng,lfsr,crc,shamir,raid --bench gf     -- --noplot
	$(ENV) cargo +nightly bench --features nightly,thread-rng,lfsr,crc,shamir,raid --bench find-p -- --noplot
	$(ENV) cargo +nightly bench --features nightly,thread-rng,lfsr,crc,shamir,raid --bench lfsr   -- --noplot
	$(ENV) cargo +nightly bench --features nightly,thread-rng,lfsr,crc,shamir,raid --bench crc    -- --noplot
	$(ENV) cargo +nightly bench --features nightly,thread-rng,lfsr,crc,shamir,raid --bench shamir -- --noplot
	$(ENV) cargo +nightly bench --features nightly,thread-rng,lfsr,crc,shamir,raid --bench raid   -- --noplot
	$(ENV) cargo +nightly bench --features nightly,thread-rng,lfsr,crc,shamir,raid --bench rs     -- --noplot

.PHONY: doc
doc:
	$(ENV) cargo +nightly doc --no-deps --features nightly,thread-rng,lfsr,crc,shamir,raid

.PHONY: clean
clean:
	$(ENV) cargo clean

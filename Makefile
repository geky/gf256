
override ENV += RUSTFLAGS="-Ctarget-cpu=native"

.PHONY: all build
all build:
	$(ENV) cargo +nightly build --features nightly

.PHONY: test-configs
test-configs:
	$(ENV) cargo test --lib
	$(ENV) cargo test --features no-xmul,thread-rng,lfsr,crc,shamir,raid,rs --lib
	$(ENV) cargo test --features thread-rng,lfsr,crc,shamir,raid,rs --lib
	$(ENV) cargo +nightly test --features nightly,thread-rng,lfsr,crc,shamir,raid,rs --lib

.PHONY: test
test:
	$(ENV) cargo +nightly test --features nightly,thread-rng,lfsr,crc,shamir,raid,rs --lib
	$(ENV) cargo +nightly test --features nightly,thread-rng,lfsr,crc,shamir,raid,rs --example find-p
	$(ENV) cargo +nightly run --features nightly,thread-rng,lfsr,crc,shamir,raid,rs --example find-p -- -w9 -n4 -m1
	$(ENV) cargo +nightly run --features nightly,thread-rng,lfsr,crc,shamir,raid,rs --example lfsr
	$(ENV) cargo +nightly run --features nightly,thread-rng,lfsr,crc,shamir,raid,rs --example crc
	$(ENV) cargo +nightly run --features nightly,thread-rng,lfsr,crc,shamir,raid,rs --example shamir
	$(ENV) cargo +nightly run --features nightly,thread-rng,lfsr,crc,shamir,raid,rs --example raid
	$(ENV) cargo +nightly run --features nightly,thread-rng,lfsr,crc,shamir,raid,rs --example rs

.PHONY: bench
bench:
	$(ENV) cargo +nightly bench --features nightly,thread-rng,lfsr,crc,shamir,raid,rs --bench xmul   -- --noplot
	$(ENV) cargo +nightly bench --features nightly,thread-rng,lfsr,crc,shamir,raid,rs --bench gf     -- --noplot
	$(ENV) cargo +nightly bench --features nightly,thread-rng,lfsr,crc,shamir,raid,rs --bench find-p -- --noplot
	$(ENV) cargo +nightly bench --features nightly,thread-rng,lfsr,crc,shamir,raid,rs --bench lfsr   -- --noplot
	$(ENV) cargo +nightly bench --features nightly,thread-rng,lfsr,crc,shamir,raid,rs --bench crc    -- --noplot
	$(ENV) cargo +nightly bench --features nightly,thread-rng,lfsr,crc,shamir,raid,rs --bench shamir -- --noplot
	$(ENV) cargo +nightly bench --features nightly,thread-rng,lfsr,crc,shamir,raid,rs --bench raid   -- --noplot
	$(ENV) cargo +nightly bench --features nightly,thread-rng,lfsr,crc,shamir,raid,rs --bench rs     -- --noplot

.PHONY: doc
doc:
	$(ENV) cargo +nightly doc --no-deps --features nightly,thread-rng,lfsr,crc,shamir,raid,rs

.PHONY: clean
clean:
	$(ENV) cargo clean

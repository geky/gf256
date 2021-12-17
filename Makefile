
override RUSTFLAGS += -Ctarget-cpu=native

# this indirection is so commands with env are easily copied on the terminal
override ENV += RUSTFLAGS="$(RUSTFLAGS)"

.PHONY: all build
all build:
	$(ENV) cargo +nightly build --features nightly,thread-rng,lfsr,crc,shamir,raid,rs

.PHONY: test
test:
	$(ENV) cargo +nightly test --features nightly,thread-rng,lfsr,crc,shamir,raid,rs --lib
	$(ENV) cargo +nightly test --features nightly,thread-rng,lfsr,crc,shamir,raid,rs --example find-p
	$(ENV) cargo +nightly run --features nightly,thread-rng,lfsr,crc,shamir,raid,rs --example find-p -- -w9 -n4 -m1 -q
	$(ENV) cargo +nightly run --features nightly,thread-rng,lfsr,crc,shamir,raid,rs --example lfsr
	$(ENV) cargo +nightly run --features nightly,thread-rng,lfsr,crc,shamir,raid,rs --example crc
	$(ENV) cargo +nightly run --features nightly,thread-rng,lfsr,crc,shamir,raid,rs --example shamir
	$(ENV) cargo +nightly run --features nightly,thread-rng,lfsr,crc,shamir,raid,rs --example raid
	$(ENV) cargo +nightly run --features nightly,thread-rng,lfsr,crc,shamir,raid,rs --example rs

.PHONY: test-configs
test-configs:
	$(ENV) cargo test --lib
	$(ENV) cargo test --features thread-rng,lfsr,crc,shamir,raid,rs --lib
	$(ENV) cargo test --features no-xmul,thread-rng,lfsr,crc,shamir,raid,rs --lib
	$(ENV) cargo test --features no-tables,thread-rng,lfsr,crc,shamir,raid,rs --lib
	$(ENV) cargo test --features small-tables,thread-rng,lfsr,crc,shamir,raid,rs --lib
	$(ENV) cargo +nightly test --features nightly,thread-rng,lfsr,crc,shamir,raid,rs --lib

.PHONY: docs
docs:
	$(ENV) cargo +nightly doc --no-deps --features nightly,thread-rng,lfsr,crc,shamir,raid,rs
	$(ENV) cargo +nightly test --features nightly,thread-rng,lfsr,crc,shamir,raid,rs --doc

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

.PHONY: bench-no-xmul
bench-no-xmul:
	$(ENV) cargo +nightly bench --features nightly,no-xmul,thread-rng,lfsr,crc,shamir,raid,rs --bench xmul   -- --noplot
	$(ENV) cargo +nightly bench --features nightly,no-xmul,thread-rng,lfsr,crc,shamir,raid,rs --bench gf     -- --noplot
	$(ENV) cargo +nightly bench --features nightly,no-xmul,thread-rng,lfsr,crc,shamir,raid,rs --bench find-p -- --noplot
	$(ENV) cargo +nightly bench --features nightly,no-xmul,thread-rng,lfsr,crc,shamir,raid,rs --bench lfsr   -- --noplot
	$(ENV) cargo +nightly bench --features nightly,no-xmul,thread-rng,lfsr,crc,shamir,raid,rs --bench crc    -- --noplot
	$(ENV) cargo +nightly bench --features nightly,no-xmul,thread-rng,lfsr,crc,shamir,raid,rs --bench shamir -- --noplot
	$(ENV) cargo +nightly bench --features nightly,no-xmul,thread-rng,lfsr,crc,shamir,raid,rs --bench raid   -- --noplot
	$(ENV) cargo +nightly bench --features nightly,no-xmul,thread-rng,lfsr,crc,shamir,raid,rs --bench rs     -- --noplot

.PHONY: clean
clean:
	$(ENV) cargo clean


override RUSTFLAGS += -Ctarget-cpu=native

# this indirection is so commands with env are easily copied on the terminal
CARGO ?= RUSTFLAGS="$(RUSTFLAGS)" cargo +nightly-2023-06-28

.PHONY: all build
all build:
	$(CARGO) build --features thread-rng,lfsr,crc,shamir,raid,rs

.PHONY: test
test:
	$(CARGO) test --features thread-rng,lfsr,crc,shamir,raid,rs --lib
	$(CARGO) test --features thread-rng,lfsr,crc,shamir,raid,rs --example find-p
	$(CARGO) run --features thread-rng,lfsr,crc,shamir,raid,rs --example find-p -- -w9 -n4 -m1 -q
	$(CARGO) run --features thread-rng,lfsr,crc,shamir,raid,rs --example lfsr
	$(CARGO) run --features thread-rng,lfsr,crc,shamir,raid,rs --example crc
	$(CARGO) run --features thread-rng,lfsr,crc,shamir,raid,rs --example shamir
	$(CARGO) run --features thread-rng,lfsr,crc,shamir,raid,rs --example raid
	$(CARGO) run --features thread-rng,lfsr,crc,shamir,raid,rs --example rs

.PHONY: test-configs
test-configs:
	$(CARGO) test --lib
	$(CARGO) test --features thread-rng,lfsr,crc,shamir,raid,rs --lib
	$(CARGO) test --features no-xmul,thread-rng,lfsr,crc,shamir,raid,rs --lib
	$(CARGO) test --features no-tables,thread-rng,lfsr,crc,shamir,raid,rs --lib
	$(CARGO) test --features small-tables,thread-rng,lfsr,crc,shamir,raid,rs --lib

.PHONY: docs
docs:
	$(CARGO) doc --no-deps --features thread-rng,lfsr,crc,shamir,raid,rs
	$(CARGO) test --features thread-rng,lfsr,crc,shamir,raid,rs --doc

.PHONY: bench
bench:
	$(CARGO) bench --features thread-rng,lfsr,crc,shamir,raid,rs --bench xmul   -- --noplot
	$(CARGO) bench --features thread-rng,lfsr,crc,shamir,raid,rs --bench gf     -- --noplot
	$(CARGO) bench --features thread-rng,lfsr,crc,shamir,raid,rs --bench find-p -- --noplot
	$(CARGO) bench --features thread-rng,lfsr,crc,shamir,raid,rs --bench lfsr   -- --noplot
	$(CARGO) bench --features thread-rng,lfsr,crc,shamir,raid,rs --bench crc    -- --noplot
	$(CARGO) bench --features thread-rng,lfsr,crc,shamir,raid,rs --bench shamir -- --noplot
	$(CARGO) bench --features thread-rng,lfsr,crc,shamir,raid,rs --bench raid   -- --noplot
	$(CARGO) bench --features thread-rng,lfsr,crc,shamir,raid,rs --bench rs     -- --noplot

.PHONY: bench-no-xmul
bench-no-xmul:
	$(CARGO) bench --features no-xmul,thread-rng,lfsr,crc,shamir,raid,rs --bench xmul   -- --noplot
	$(CARGO) bench --features no-xmul,thread-rng,lfsr,crc,shamir,raid,rs --bench gf     -- --noplot
	$(CARGO) bench --features no-xmul,thread-rng,lfsr,crc,shamir,raid,rs --bench find-p -- --noplot
	$(CARGO) bench --features no-xmul,thread-rng,lfsr,crc,shamir,raid,rs --bench lfsr   -- --noplot
	$(CARGO) bench --features no-xmul,thread-rng,lfsr,crc,shamir,raid,rs --bench crc    -- --noplot
	$(CARGO) bench --features no-xmul,thread-rng,lfsr,crc,shamir,raid,rs --bench shamir -- --noplot
	$(CARGO) bench --features no-xmul,thread-rng,lfsr,crc,shamir,raid,rs --bench raid   -- --noplot
	$(CARGO) bench --features no-xmul,thread-rng,lfsr,crc,shamir,raid,rs --bench rs     -- --noplot

.PHONY: clean
clean:
	$(CARGO) clean

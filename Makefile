
override ENV += RUSTFLAGS="-Ctarget-cpu=native"

.PHONY: all build
all build:
	$(ENV) cargo build

.PHONY: test
define TEST_EXAMPLE
	$(ENV) cargo run --example $(1)

endef
test:
	$(ENV) cargo test --lib
	$(patsubst examples/%.rs,$(call TEST_EXAMPLE,%),$(wildcard examples/*.rs))


.PHONY: bench
bench:
	$(ENV) cargo +nightly bench --features use-nightly-features

.PHONY: clean
clean:
	$(ENV) cargo clean

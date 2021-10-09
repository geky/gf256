
override ENV += RUSTFLAGS=-Ctarget-cpu=native

.PHONY: all build
all build:
	$(ENV) cargo build

.PHONY: test
test:
	$(ENV) cargo test --lib

.PHONY: clean
clean:
	$(ENV) cargo clean

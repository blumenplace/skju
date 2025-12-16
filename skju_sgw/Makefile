.PHONY: run clean fmt deny



deny:
	@cargo deny check


fmt:
	@cargo fmt


build:
	@cargo build --release


run:
	 probe-rs attach --chip nRF9161_xxAA --defmt

clean:
	@cargo clean

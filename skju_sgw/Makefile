CONTAINER := $(shell command -v podman >/dev/null 2>&1 && echo podman || (command -v docker >/dev/null 2>&1 && echo docker))
ifeq ($(strip $(CONTAINER)),)
  $(error Neither 'podman' nor 'docker' found in PATH)
endif

FIRMWARE_PATH := ./target/thumbv8m.main-none-eabihf/debug/skju_sgw

.PHONY: run clean fmt deny renode

deny:
	@cargo deny check

fmt:
	@cargo fmt --check

build: deny fmt
	@cargo build

renode: build
	@$(CONTAINER) run -it --rm \
	  -v "$(PWD)":/app -w /app \
	  -e FIRMWARE_PATH="/app/$(FIRMWARE_PATH)" \
	  antmicro/renode:latest \
	  renode /app/scripts/nrf9161dk.resc

flash:
	@probe-rs attach --chip nRF9161_xxAA --defmt

clean:
	@cargo clean

CONTAINER := $(shell command -v podman >/dev/null 2>&1 && echo podman || (command -v docker >/dev/null 2>&1 && echo docker))
ifeq ($(strip $(CONTAINER)),)
  $(error Neither 'podman' nor 'docker' found in PATH)
endif

FIRMWARE_PATH := ./target/thumbv7em-none-eabihf/debug/skju_sn

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
	  -e FIRMWARE_PATH?="/app/$(FIRMWARE_PATH)" \
	  -p 3333:3333 \
	  antmicro/renode:latest \
	  renode --disable-gui --console -e 'set FIRMWARE_PATH "/app/$(FIRMWARE_PATH)"; start @/app/scripts/nrf52840dk.resc'

flash:
	@probe-rs attach --chip nRF52840_xxAA --defmt

gdb: build
	arm-none-eabi-gdb -x gdb-commands.gdb ${FIRMWARE_PATH}

clean:
	@cargo clean

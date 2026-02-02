RENODE_VERSION:=1.16.0
ARCH:=$(shell uname -m)
ifeq ($(ARCH),arm64)
	RENODE_PKG:=linux-arm64-portable-dotnet
	RENODE_PLATFORM:=linux/arm64
else
	RENODE_PKG:=linux-portable-dotnet
	RENODE_PLATFORM:=linux/amd64
endif

CONTAINER := $(shell command -v podman >/dev/null 2>&1 && echo podman || (command -v docker >/dev/null 2>&1 && echo docker))
ifeq ($(strip $(CONTAINER)),)
  $(error Neither 'podman' nor 'docker' found in PATH)
endif

FIRMWARE_PATH := ./target/thumbv7em-none-eabihf/debug/skju_sn

.PHONY: run clean fmt deny renode renode-img logs

deny:
	@cargo deny check

fmt:
	@cargo fmt --check

build: deny fmt
	@DEFMT_LOG=trace cargo build

renode: build
	@$(CONTAINER) run -it --rm \
	  -v "$(PWD)":/workspace \
	  -w /workspace \
	  -e FIRMWARE_PATH?="/workspace/$(FIRMWARE_PATH)" \
	  -p 3333:3333 \
	  -p 19021:19021 \
	  renode-skju:latest \
	  renode --disable-gui --console -e 'set FIRMWARE_PATH "/workspace/$(FIRMWARE_PATH)"; start @/workspace/scripts/nrf52840dk.resc'

logs:
	@defmt-print -e ${FIRMWARE_PATH} tcp --host localhost --port 19021

flash:
	@probe-rs attach --chip nRF52840_xxAA --defmt

gdb: build
	@arm-none-eabi-gdb -x gdb-commands.gdb ${FIRMWARE_PATH}

clean:
	@cargo clean


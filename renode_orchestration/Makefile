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

SENSOR_FIRMWARE_PATH := skju_sn/target/thumbv7em-none-eabihf/debug/skju_sn
GATEWAY_FIRMWARE_PATH := skju_sgw/target/thumbv8m.main-none-eabihf/debug/skju_sgw

build-sensor:
	@(cd ../skju_sn && cargo build)

build-gateway:
	@(cd ../skju_sgw && cargo build)

renode-img:
	@$(CONTAINER) build --platform $(RENODE_PLATFORM) --build-arg RENODE_VERSION=$(RENODE_VERSION) --build-arg RENODE_PKG=$(RENODE_PKG) -t skju-renode -f Dockerfile.renode .

renode: renode-img build-sensor build-gateway
	@$(CONTAINER) run -it --rm \
	  -v "$(PWD)/..":/workspace \
      -w /workspace/renode_orchestration \
	  -e SENSOR_FIRMWARE_PATH?="/workspace/$(SENSOR_FIRMWARE_PATH)" \
	  -e GATEWAY_FIRMWARE_PATH?="/workspace/$(GATEWAY_FIRMWARE_PATH)" \
	  -p 3333:3333 \
	  -p 3334:3334 \
	  skju-renode:latest \
	  renode --disable-gui --console -e 'set SENSOR_FIRMWARE_PATH "/workspace/$(SENSOR_FIRMWARE_PATH)"; set GATEWAY_FIRMWARE_PATH "/workspace/$(GATEWAY_FIRMWARE_PATH)"; start @/workspace/renode_orchestration/orchestration.resc'

GRPC_HEALTH_PROBE_VERSION ?= v0.4.48


.PHONY: build-container
build-container: CONTAINER?=$(shell command -v podman >/dev/null 2>&1 && echo podman || ((command -v docker >/dev/null 2>&1 && echo docker) || echo "false"))
build-container:
	@echo "Building Dockerfile image"
	@$(CONTAINER) build \
		-f ./Dockerfile \
		--target build \
		-v "$(PWD)/../":/workspace \
		-t skju_heim:latest \
                --build-arg GRPC_HEALTH_PROBE_VERSION=$(strip ${GRPC_HEALTH_PROBE_VERSION}) \
                --build-arg EXPORT_CACHEBUST="$$(date +%s)" .

BUILD ?= debug
BUILD_PARAMS :=
ifeq ($(BUILD), release)
	BUILD_PARAMS := --release
endif

RENODE_VERSION := 1.16.0
ifeq ($(shell uname -m),arm64)
	RENODE_PKG := linux-arm64-portable-dotnet
	RENODE_PLATFORM := linux/arm64
else
	RENODE_PKG := linux-portable-dotnet
	RENODE_PLATFORM := linux/amd64
endif

ifeq ($(shell uname -s),Linux)
    LIB_EXT = so
    LIB_PREFIX = lib
else ifeq ($(shell uname -s),Darwin)
    LIB_EXT = dylib
    LIB_PREFIX = lib
else
    LIB_EXT = dll
    LIB_PREFIX =
endif

TARGET_LIB := target/$(BUILD)/$(LIB_PREFIX)defmt_cs.$(LIB_EXT)


.PHONY: clean img

all: $(TARGET_LIB) bindings/defmt_cs.cs

$(TARGET_LIB): src/lib.rs Cargo.toml
	@cargo build $(BUILD_PARAMS)

bindings/defmt_cs.cs: $(TARGET_LIB)
	@uniffi-bindgen-cs --library $(TARGET_LIB) --config uniffi-bindgen-cs.toml --out-dir ./bindings
	@printf '%s\n' \
    		'<Project Sdk="Microsoft.NET.Sdk">' \
    		'	<PropertyGroup>' \
    		'		<TargetFramework>net8.0</TargetFramework>' \
    		'		<ImplicitUsings>enable</ImplicitUsings>' \
    		'		<Nullable>enable</Nullable>' \
    		'		<AllowUnsafeBlocks>true</AllowUnsafeBlocks>' \
    		'	</PropertyGroup>' \
    		'</Project>' \
    		> ./bindings/DefmtBindings.csproj

img:
	@podman build \
		-f ./Dockerfile  \
		--platform $(RENODE_PLATFORM) \
		--build-arg RENODE_VERSION=$(RENODE_VERSION) \
		--build-arg RENODE_PKG=$(RENODE_PKG) \
		-t renode-skju .

clean:
	@rm -rf ./bindings
	@cargo clean

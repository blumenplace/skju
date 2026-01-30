BUILD ?= debug
BUILD_PARAMS:=
ifeq ($(BUILD), release)
	BUILD_PARAMS:=--release
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

TARGET_LIB:=target/$(BUILD)/$(LIB_PREFIX)defmt_cs.$(LIB_EXT)


.PHONY: clean

all: $(TARGET_LIB) bindings/defmt_cs.cs

$(TARGET_LIB): src/lib.rs Cargo.toml
	@cargo build $(BUILD_PARAMS)

bindings/defmt_cs.cs: $(TARGET_LIB)
	@uniffi-bindgen-cs --library $(TARGET_LIB) --out-dir ./bindings

clean:
	@rm -rf ./bindings
	@cargo clean

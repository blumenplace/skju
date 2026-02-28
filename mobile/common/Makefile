BUILD ?= release
IOS_PARAMS := $(if ($(strip $(BUILD)),release),--release,)
SDK = $(shell xcrun --sdk iphonesimulator --show-sdk-path)


RUST_LIB_IOS_DIR := ./target/aarch64-apple-ios/$(strip $(BUILD))
RUST_LIB_IOS_SIM_DIR := ./target/aarch64-apple-ios-sim/$(strip $(BUILD))
RUST_LIB_IOS := $(RUST_LIB_IOS_DIR)/libskju_common.a
RUST_LIB_IOS_SIM := $(RUST_LIB_IOS_SIM_DIR)/libskju_common.a


SWIFT := xcrun --sdk iphoneos swiftc
SWIFT_SIM := xcrun --sdk iphonesimulator swiftc

SWIFT_SDK_IOS := $(shell xcrun --sdk iphoneos --show-sdk-path)
SWIFT_SDK_SIM := $(shell xcrun --sdk iphonesimulator --show-sdk-path)

SWIFT_MODULE := SkjuCommon
SWIFT_PRODUCT := $(SWIFT_MODULE)
SWIFT_BUILD_DIR := ./target/SkjuCommon-build
SWIFT_SRC_DIR := ./target/SkjuCommon-sources
SWIFT_SRC := $(SWIFT_SRC_DIR)/SkjuCommon.swift \
	$(SWIFT_SRC_DIR)/SkjuCommonFFI.h \
	$(SWIFT_SRC_DIR)/module.modulemap


define swift_build_commands
	$(if $(findstring simulator,$(1)),$(eval SWC=$(SWIFT_SIM)),$(eval SWC=$(SWIFT)))
	$(if $(findstring simulator,$(1)),$(eval SWIFT_TRIPLE=arm64-apple-ios-simulator),$(eval SWIFT_TRIPLE=arm64-apple-ios))
	$(if $(findstring simulator,$(1)),$(eval SWIFT_TARGET=arm64-apple-ios18.0-simulator),$(eval SWIFT_TARGET=arm64-apple-ios18.0))
	$(if $(findstring simulator,$(1)),$(eval RUST_LIB_DIR=$(RUST_LIB_IOS_SIM_DIR)),$(eval RUST_LIB_DIR=$(RUST_LIB_IOS_DIR)))
	@echo "compiling Swift library for $(1)..."
	@mkdir -p $(SWIFT_BUILD_DIR)/$(1)/$(SWIFT_MODULE).framework/Modules/$(SWIFT_MODULE).swiftmodule
	@$(SWC) \
		-target $(SWIFT_TARGET) \
		-parse-as-library \
		-module-name $(SWIFT_MODULE) \
		-I $(SWIFT_SRC_DIR) \
		-L $(RUST_LIB_DIR) \
		-lskju_common \
		-emit-library -static \
		-emit-module -emit-module-path $(SWIFT_BUILD_DIR)/$(1)/$(SWIFT_MODULE).framework/Modules/$(SWIFT_MODULE).swiftmodule/$(SWIFT_TRIPLE).swiftmodule \
		-emit-module-interface -emit-module-interface-path $(SWIFT_BUILD_DIR)/$(1)/$(SWIFT_MODULE).framework/Modules/$(SWIFT_MODULE).swiftmodule/$(SWIFT_TRIPLE).swiftinterface \
		-enable-library-evolution \
		-module-cache-path $(SWIFT_BUILD_DIR)/ModuleCache \
		-o $(SWIFT_BUILD_DIR)/$(1)/$(SWIFT_MODULE).framework/$(SWIFT_PRODUCT) \
		$(SWIFT_SRC_DIR)/$(SWIFT_MODULE).swift
	@mkdir -p $(SWIFT_BUILD_DIR)/$(1)/$(SWIFT_MODULE).framework/Headers
	@cp -R $(SWIFT_SRC_DIR)/Headers/* $(SWIFT_BUILD_DIR)/$(1)/$(SWIFT_MODULE).framework/Headers/
	@cp $(SWIFT_SRC_DIR)/module.modulemap $(SWIFT_BUILD_DIR)/$(1)/$(SWIFT_MODULE).framework/Modules/module.modulemap
	@echo "merging Rust static library into Swift static library for $(1)..."
	@xcrun libtool -static \
		-o $(SWIFT_BUILD_DIR)/$(1)/$(SWIFT_MODULE).framework/$(SWIFT_PRODUCT).merged \
		$(SWIFT_BUILD_DIR)/$(1)/$(SWIFT_MODULE).framework/$(SWIFT_PRODUCT) \
		$(RUST_LIB_DIR)/libskju_common.a
	@mv $(SWIFT_BUILD_DIR)/$(1)/$(SWIFT_MODULE).framework/$(SWIFT_PRODUCT).merged \
		$(SWIFT_BUILD_DIR)/$(1)/$(SWIFT_MODULE).framework/$(SWIFT_PRODUCT)
	@xcrun ranlib $(SWIFT_BUILD_DIR)/$(1)/$(SWIFT_MODULE).framework/$(SWIFT_PRODUCT)
endef


$(SWIFT_BUILD_DIR):
	@echo "ensure Swift build directory folder exists..."
	@mkdir -p $(SWIFT_BUILD_DIR)


$(RUST_LIB_IOS_SIM): ./src/lib.rs
	@echo "building static libraries for iOS simulator..."
	@cargo build $(strip $(IOS_PARAMS)) --target aarch64-apple-ios-sim


$(RUST_LIB_IOS): ./src/lib.rs
	@echo "building static libraries for iOS..."
	@cargo build $(strip $(IOS_PARAMS)) --target aarch64-apple-ios


$(SWIFT_SRC): $(RUST_LIB_IOS)
	@echo "generate Swift sources..."
	@uniffi-bindgen-swift $(RUST_LIB_IOS) $(SWIFT_SRC_DIR) --swift-sources
	@uniffi-bindgen-swift $(RUST_LIB_IOS) \
		$(SWIFT_SRC_DIR)/Headers \
		--headers \
		--module-name $(SWIFT_MODULE)FFI
	@uniffi-bindgen-swift $(RUST_LIB_IOS) \
		$(SWIFT_SRC_DIR) \
		--xcframework --modulemap \
		--module-name $(SWIFT_MODULE)FFI \
		--modulemap-filename module.modulemap


swift-build-ios: $(RUST_LIB_IOS) $(SWIFT_SRC)
	$(call swift_build_commands,arm64-apple-ios18.0)


swift-build-ios-sim: $(RUST_LIB_IOS_SIM) $(SWIFT_SRC)
	$(call swift_build_commands,arm64-apple-ios18.0-simulator)


$(SWIFT_BUILD_DIR)/$(SWIFT_MODULE).xcframework: swift-build-ios swift-build-ios-sim
	@rm -rf $(SWIFT_BUILD_DIR)/$(SWIFT_MODULE).xcframework
	@xcodebuild -create-xcframework \
		-framework $(SWIFT_BUILD_DIR)/arm64-apple-ios18.0-simulator/$(SWIFT_MODULE).framework \
		-framework $(SWIFT_BUILD_DIR)/arm64-apple-ios18.0/$(SWIFT_MODULE).framework \
		-output $(SWIFT_BUILD_DIR)/$(SWIFT_MODULE).xcframework


xcframework: $(SWIFT_BUILD_DIR)/$(SWIFT_MODULE).xcframework
	@echo "xcframework generated at $(SWIFT_BUILD_DIR)/$(SWIFT_MODULE).xcframework"


.PHONY: fmt
fmt:
	@echo "Check code formatting"
	@cargo fmt --check


.PHONY: swift-clean
swift-clean:
	@rm -rf $(SWIFT_SRC_DIR)
	@rm -rf $(SWIFT_BUILD_DIR)


.PHONY: clean
clean: swift-clean
	@rm -rf ./target
	@cargo clean


check-swc: xcframework
	@echo "import SkjuCommon\n" > $(SWIFT_BUILD_DIR)/demo.swift
	@xcrun -sdk iphonesimulator swiftc \
		-target arm64-apple-ios18.0-simulator \
		-F $(SWIFT_BUILD_DIR)/SkjuCommon.xcframework/ios-arm64-simulator \
		-I $(SWIFT_SRC_DIR) \
		-framework SkjuCommon \
		-o $(SWIFT_BUILD_DIR)/demo.swift.o \
		$(SWIFT_BUILD_DIR)/demo.swift

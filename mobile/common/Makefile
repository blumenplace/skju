BUILD ?= release
IOS_PARAMS := $(if ($(strip $(BUILD)),release),--release,)
SDK = $(shell xcrun --sdk iphonesimulator --show-sdk-path)


RUST_LIB_IOS_DIR := ./target/aarch64-apple-ios/$(strip $(BUILD))
RUST_LIB_IOS_SIM_DIR := ./target/aarch64-apple-ios-sim/$(strip $(BUILD))
RUST_LIB_IOS := $(RUST_LIB_IOS_DIR)/libskju_common.a
RUST_LIB_IOS_SIM := $(RUST_LIB_IOS_SIM_DIR)/libskju_common.a
SWIFT_SOURCES := ./target/swift-build \
	./target/swift/SkjuCommon.swift \
	./target/swift/SkjuCommonFFI.h \
	./target/swift/module.modulemap



build: ./target/swift-build/SkjuCommon.xcframework


./target/swift:
	@echo "Ensure Swift headers folder exists..."
	@mkdir -p ./target/swift


$(RUST_LIB_IOS_SIM): ./target/swift
	@echo "Building static libraries for iOS simulator..."
	@cargo build $(strip $(IOS_PARAMS)) --target aarch64-apple-ios-sim


$(RUST_LIB_IOS): ./target/swift
	@echo "Building static libraries for iOS..."
	@cargo build $(strip $(IOS_PARAMS)) --target aarch64-apple-ios


./target/swift/SkjuCommon.swift: $(RUST_LIB_IOS)
	@echo "Generate Swift sources..."
	@uniffi-bindgen-swift $(RUST_LIB_IOS) ./target/swift --swift-sources


./target/swift/SkjuCommonFFI.h: $(RUST_LIB_IOS)
	@echo "Generate Swift FFI interface..."
	@uniffi-bindgen-swift $(RUST_LIB_IOS) ./target/swift --headers


./target/swift/module.modulemap: $(RUST_LIB_IOS)
	@echo "Generate Swifti modulemap..."
	@uniffi-bindgen-swift $(RUST_LIB_IOS) \
		./target/swift \
		--modulemap \
		--module-name SkjuCommonFFI \
		--modulemap-filename module.modulemap


./target/swift-build:
	@echo "Creating output directory for iPhone simulator..."
	@mkdir -p ./target/swift-build



SWIFT_MODULE := SkjuCommon
SWIFT_PRODUCT := lib$(SWIFT_MODULE).a
SWIFT_BUILD_DIR := ./target/swift-build

swift-build-ios: $(RUST_LIB_IOS) ./target/swift-build $(SWIFT_SOURCES)
	@echo "Compiling Swift library for iOS"
	@xcrun -sdk iphoneos swiftc \
		-target arm64-apple-ios18.0 \
		-parse-as-library \
		-module-name SkjuCommon \
		-I ./target/swift \
		-Xcc -I./target/swift \
		-L $(RUST_LIB_IOS_DIR) \
		-lcommont \
		-emit-library -static \
		-emit-module \
		-emit-module-path ./target/swift-build/ios/Modules/SkjuCommon.swiftmodule/arm64-apple-ios.swiftmodule \
		-emit-module-interface \
  		-emit-module-interface-path ./target/swift-build/ios/Modules/SkjuCommon.swiftmodule/arm64-apple-ios.swiftinterface \
		-enable-library-evolution \
		-module-cache-path ./target/module-cache-swift \
		-o ./target/swift-build/ios/libSkjuCommon.a \
		./target/swift/SkjuCommon.swift


swift-build-ios-sim: $(RUST_LIB_IOS_SIM) ./target/swift-build $(SWIFT_SOURCES)
	@echo "Compiling Swift library for iOS simulator..."
	@xcrun -sdk iphonesimulator swiftc \
		-target arm64-apple-ios18.0-simulator \
		-parse-as-library \
		-module-name SkjuCommon \
		-I ./target/swift \
		-Xcc -I./target/swift \
		-L $(RUST_LIB_IOS_SIM_DIR) \
		-lcommont \
		-emit-library -static \
		-emit-module \
		-emit-module-path ./target/swift-build/ios-sim/Modules/SkjuCommon.swiftmodule/arm64-apple-ios-simulator.swiftmodule \
		-emit-module-interface \
  		-emit-module-interface-path ./target/swift-build/ios-sim/Modules/SkjuCommon.swiftmodule/arm64-apple-ios-simulator.swiftinterface \
		-enable-library-evolution \
		-module-cache-path ./target/module-cache-swift \
		-o ./target/swift-build/ios-sim/libSkjuCommon.a \
		./target/swift/SkjuCommon.swift


./target/swift-build/SkjuCommon.xcframework: swift-build-ios-sim swift-build-ios
	@echo "Swift build complete"
	@echo "Generating XCFramework..."
	@xcodebuild -create-xcframework \
		-library ./target/swift-build/ios/libSkjuCommon.a \
		-library ./target/swift-build/ios-sim/libSkjuCommon.a \
		-output ./target/swift-build/SkjuCommon.xcframework


.PHONY: swift-build
swift-build: ./target/swift-build/SkjuCommon.xcframework
	@echo "Swift build complete"


# ./target/CommontLib.xcframework: ./target/aarch64-apple-ios/$(strip $(BUILD))/libcommont.a ./target/aarch64-apple-ios-sim/$(strip $(BUILD))/libcommont.a $(SWIFT_SOURCES)
# 	@echo "Create XCFramework"
# 	@rm -rf ./target/CommontLib.xcframework
# 	@xcodebuild -create-xcframework \
#   		-library ./target/aarch64-apple-ios-sim/$(strip $(BUILD))/libcommont.a \
#   		-headers ./target/swift \
# 		-library ./target/aarch64-apple-ios/$(strip $(BUILD))/libcommont.a \
# 		-headers ./target/swift \
# 		-output ./target/CommontLib.xcframework


.PHONY: fmt
fmt:
	@echo "Check code formatting"
	@cargo fmt --check


.PHONY: clean
clean:
	@rm -rf ./target
	@cargo clean

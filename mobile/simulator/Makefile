APP_NAME=SkjuSimulator
BUNDLE_ID=place.blumen.skju
SIMULATOR_NAME="iPad"
ARCH=$(shell uname -m)

SDK_PATH := $(shell xcrun --sdk iphonesimulator --show-sdk-path 2>/dev/null)

SKJU_COMMON_BUILD_DIR := ../common/target/swift-build
SKJU_COMMON_XCFRAMEWORK := $(strip $(SKJU_COMMON_BUILD_DIR))/SkjuCommon.xcframework

ifeq ($(strip $(SDK_PATH)),)
    XCODE_DEV_DIR := /Applications/Xcode.app/Contents/Developer
    ifneq ($(wildcard $(XCODE_DEV_DIR)),)
        SDK_PATH := $(shell export DEVELOPER_DIR=$(XCODE_DEV_DIR); xcrun --sdk iphonesimulator --show-sdk-path 2>/dev/null)
        export DEVELOPER_DIR := $(XCODE_DEV_DIR)
    endif
endif


all: run


$(APP_NAME).app/Info.plist:
	@mkdir -p $(APP_NAME).app
	@{ \
		echo '<?xml version="1.0" encoding="UTF-8"?>'; \
		echo '<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">'; \
		echo '<plist version="1.0"><dict>'; \
		echo '    <key>CFBundleExecutable</key><string>$(APP_NAME)</string>'; \
		echo '    <key>CFBundleIdentifier</key><string>$(BUNDLE_ID)</string>'; \
		echo '    <key>CFBundleShortVersionString</key><string>1.0</string>'; \
		echo '    <key>CFBundleVersion</key><string>1</string>'; \
		echo '    <key>LSRequiresIPhoneOS</key><true/>'; \
		echo '    <key>UIDeviceFamily</key>'; \
		echo '    <array>'; \
		echo '        <integer>2</integer>'; \
		echo '    </array>'; \
		echo '    <key>UILaunchScreen</key><dict/>'; \
		echo '    <key>LSMinimumSystemVersion</key>'; \
		echo '    <string>18.0</string>'; \
		echo '</dict></plist>'; \
	} > $(APP_NAME).app/Info.plist
	@echo "generated $(APP_NAME).app/Info.plist"



build: $(APP_NAME).app/Info.plist
	@echo "building for $(ARCH) using SDK at $(SDK_PATH)..."
	@if [ -z "$(SDK_PATH)" ]; then \
		echo "Error: SDK Path not found."; \
		echo "1. Ensure Xcode is installed from the App Store."; \
		echo "2. Run 'sudo xcode-select -s /Applications/Xcode.app' to select it."; \
		exit 1; \
	fi
	@echo "compile $(APP_NAME)"
	@echo "SEARCH: $(SKJU_COMMON_XCFRAMEWORK)/ios-arm64-simulator"
	@xcrun -sdk iphonesimulator swiftc \
		-target arm64-apple-ios18.0-simulator \
		-parse-as-library \
		-F $(SKJU_COMMON_BUILD_DIR) \
		-F $(SKJU_COMMON_XCFRAMEWORK)/ios-arm64-simulator \
		-framework SkjuCommon \
		-o $(APP_NAME).app/$(APP_NAME) \
		Sources/$(APP_NAME)/Main.swift


# Does not work: -I $(SKJU_COMMON_XCFRAMEWORK)/ios-arm64-simulator/SkjuCommon.framework/Headers \
# module SkjuCommonFFI [system] {
# umbrella "SkjuCommonFFI.h"
# export *
# }
# Or with umbrella directory:
# module SkjuCommonFFI { umbrella header "SkjuCommonFFI.h"; export *; }


.PHONY: boot
boot:
	@echo "booting simulator..."
	xcrun simctl boot $(SIMULATOR_NAME) || true


.PHONY: install
install:
	@echo "installing $(APP_NAME)..."
	xcrun simctl install $(SIMULATOR_NAME) $(APP_NAME).app


.PHONY: launch
launch:
	@echo "launching $(APP_NAME)..."
	xcrun simctl launch $(SIMULATOR_NAME) $(BUNDLE_ID)


fmt:
	xcrun swift-format --in-place ./Sources/$(APP_NAME)/Main.swift


.PHONY: run
run: fmt build boot install launch


.PHONY: clean
clean:
	@echo "cleaning build directories..."
	@rm -rf $(APP_NAME).app

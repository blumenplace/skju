APP_NAME=SkjuIOS
BUNDLE_ID=place.blumen.skju
SIMULATOR_NAME="iPad"
ARCH=$(shell uname -m)

# 1. Try to find SDK in current environment
SDK_PATH := $(shell xcrun --sdk iphonesimulator --show-sdk-path 2>/dev/null)

# 2. If not found, try forcing the standard Xcode path
ifeq ($(SDK_PATH),)
    XCODE_DEV_DIR := /Applications/Xcode.app/Contents/Developer
    ifneq ($(wildcard $(XCODE_DEV_DIR)),)
        SDK_PATH := $(shell export DEVELOPER_DIR=$(XCODE_DEV_DIR); xcrun --sdk iphonesimulator --show-sdk-path 2>/dev/null)
        export DEVELOPER_DIR := $(XCODE_DEV_DIR)
    endif
endif

all: run

build:
	@if [ -z "$(SDK_PATH)" ]; then \
		echo "Error: SDK Path not found."; \
		echo "1. Ensure Xcode is installed from the App Store."; \
		echo "2. Run 'sudo xcode-select -s /Applications/Xcode.app' to select it."; \
		exit 1; \
	fi
	@echo "Building for $(ARCH) using SDK at $(SDK_PATH)..."
	mkdir -p $(APP_NAME).app

	# Generate Info.plist
	@echo '<?xml version="1.0" encoding="UTF-8"?>' > $(APP_NAME).app/Info.plist
	@echo '<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">' >> $(APP_NAME).app/Info.plist
	@echo '<plist version="1.0"><dict>' >> $(APP_NAME).app/Info.plist
	@echo '    <key>CFBundleExecutable</key><string>$(APP_NAME)</string>' >> $(APP_NAME).app/Info.plist
	@echo '    <key>CFBundleIdentifier</key><string>$(BUNDLE_ID)</string>' >> $(APP_NAME).app/Info.plist
	@echo '    <key>CFBundleShortVersionString</key><string>1.0</string>' >> $(APP_NAME).app/Info.plist
	@echo '    <key>CFBundleVersion</key><string>1</string>' >> $(APP_NAME).app/Info.plist
	@echo '    <key>LSRequiresIPhoneOS</key><true/>' >> $(APP_NAME).app/Info.plist
	@echo '    <key>UIDeviceFamily</key>' >> $(APP_NAME).app/Info.plist
	@echo '    <array>' >> $(APP_NAME).app/Info.plist
    #@echo '        <integer>1</integer>' >> $(APP_NAME).app/Info.plist
	@echo '        <integer>2</integer>' >> $(APP_NAME).app/Info.plist
	@echo '    </array>' >> $(APP_NAME).app/Info.plist
	@echo '    <key>UILaunchScreen</key><dict/>' >> $(APP_NAME).app/Info.plist
	@echo '    <key>LSMinimumSystemVersion</key>' >> $(APP_NAME).app/Info.plist
	@echo '    <string>17.0</string>' >> $(APP_NAME).app/Info.plist
	@echo '</dict></plist>' >> $(APP_NAME).app/Info.plist

	# Compile with -parse-as-library
	# Compile using xcrun to ensure SDKROOT is set correctly for clang/linker
	xcrun -sdk iphonesimulator swiftc Sources/SkjuIOS/Main.swift \
		-target $(ARCH)-apple-ios17.0-simulator \
		-parse-as-library \
		-o $(APP_NAME).app/$(APP_NAME)

boot:
	@echo "Booting Simulator..."
	xcrun simctl boot $(SIMULATOR_NAME) || true

install:
	@echo "Installing App..."
	xcrun simctl install $(SIMULATOR_NAME) $(APP_NAME).app

launch:
	@echo "Launching App..."
	xcrun simctl launch $(SIMULATOR_NAME) $(BUNDLE_ID)

run: build boot install launch

clean:
	rm -rf $(APP_NAME).app
APP_NAME=SkjuIOS
BUNDLE_ID=place.blumen.skju
SIMULATOR_NAME="iPad"
ARCH=$(shell uname -m)

SDK_PATH := $(shell xcrun --sdk iphonesimulator --show-sdk-path 2>/dev/null)

ifeq ($(strip $(SDK_PATH)),)
    XCODE_DEV_DIR := /Applications/Xcode.app/Contents/Developer
    ifneq ($(wildcard $(XCODE_DEV_DIR)),)
        SDK_PATH := $(shell export DEVELOPER_DIR=$(XCODE_DEV_DIR); xcrun --sdk iphonesimulator --show-sdk-path 2>/dev/null)
        export DEVELOPER_DIR := $(XCODE_DEV_DIR)
    endif
endif


all: run


Binaries/CommontLib.xcframework:
	#@cd ../commont && make swift-build
	#@cp -r ../commont/target/swift-build/CommontLib.xcframework Binaries/
	@echo "Do nothing for now"


$(APP_NAME).app/Info.plist:
	@mkdir -p $(APP_NAME).app
	@echo "Generate $(APP_NAME).app/Info.plist"
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
	@echo '        <integer>2</integer>' >> $(APP_NAME).app/Info.plist
	@echo '    </array>' >> $(APP_NAME).app/Info.plist
	@echo '    <key>UILaunchScreen</key><dict/>' >> $(APP_NAME).app/Info.plist
	@echo '    <key>LSMinimumSystemVersion</key>' >> $(APP_NAME).app/Info.plist
	@echo '    <string>18.0</string>' >> $(APP_NAME).app/Info.plist
	@echo '</dict></plist>' >> $(APP_NAME).app/Info.plist


build: Binaries/CommontLib.xcframework $(APP_NAME).app/Info.plist
	@echo "Building for $(ARCH) using SDK at $(SDK_PATH)..."
	@if [ -z "$(SDK_PATH)" ]; then \
		echo "Error: SDK Path not found."; \
		echo "1. Ensure Xcode is installed from the App Store."; \
		echo "2. Run 'sudo xcode-select -s /Applications/Xcode.app' to select it."; \
		exit 1; \
	fi
	@echo "Compile SkjuIOS simulator app"
	@xcrun -sdk iphonesimulator swiftc Sources/SkjuIOS/Main.swift \
		-target arm64-apple-ios18.0-simulator \
		-parse-as-library \
		-F Binaries \
		-framework CommontLib \
		-o $(APP_NAME).app/$(APP_NAME)


.PHONY: boot
boot:
	@echo "Booting Simulator..."
	xcrun simctl boot $(SIMULATOR_NAME) || true


.PHONY: install
install:
	@echo "Installing App..."
	xcrun simctl install $(SIMULATOR_NAME) $(APP_NAME).app


.PHONY: launch
launch:
	@echo "Launching App..."
	xcrun simctl launch $(SIMULATOR_NAME) $(BUNDLE_ID)


fmt:
	xcrun swift-format --in-place ./Sources/SkjuIOS/Main.swift


.PHONY: run
run: fmt build boot install launch


.PHONY: clean
clean:
	@echo "Cleaning build directories..."
	@rm -rf $(APP_NAME).app
	@rm -rf ./Binaries
	@rm -rf ./Sources/CommontLib

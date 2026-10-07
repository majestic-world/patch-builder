.DEFAULT_GOAL := build
POWERSHELL ?= pwsh
BUILD_SCRIPT := tools/build.ps1

.PHONY: build

# Optimized release binary, published as dist/Builder.exe (logs in target/logs).
build:
	$(POWERSHELL) -NoProfile -ExecutionPolicy Bypass -File "$(BUILD_SCRIPT)"

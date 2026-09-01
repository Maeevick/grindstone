SHELL := bash
.SHELLFLAGS := -euo pipefail -c
.DEFAULT_GOAL := help

.PHONY: help dev test check test-ui mutants android import-draft-android mac import-draft-mac

APK := src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk
DRAFT_ON_THE_PHONE := /sdcard/Android/data/com.maeevick.grindstone/files/Documents/grindstone-draft.json
DRAFT_ON_THE_MAC := $(HOME)/Documents/grindstone-draft.json
TEST_SUPPORT := core/src/testing\.rs
WIRING := (src-tauri/src/(main|lib|configurator)|ui/src/(main|app|tauri|screens)|ui/src/screens/[a-z_]+)\.rs

help: ## list the targets
	@grep -E '^[a-zA-Z0-9_-]+:.*## ' $(MAKEFILE_LIST) | awk -F':.*## ' '{printf "  %-21s %s\n", $$1, $$2}'

dev: ## run the app on the Mac, reloading on change
	cargo tauri dev

test: ## every Rust test
	cargo test --workspace

check: ## formatting, lints, tests and no uncovered line outside the wiring and the test support, as before a commit
	cargo fmt --all --check
	cargo clippy --workspace --all-targets -- -D warnings
	cargo clippy -p grind-ui --target wasm32-unknown-unknown --all-targets -- -D warnings
	cargo llvm-cov --workspace --ignore-filename-regex '$(WIRING)|$(TEST_SUPPORT)' --fail-uncovered-lines 0 --summary-only

test-ui: ## the screen's tests in a headless Chrome, driven by the ChromeDriver of the installed Chrome rather than the newest
	driver=$$(scripts/chromedriver-for-installed-chrome) && wasm-pack test --headless --chrome --chromedriver "$$driver" ui

mutants: ## mutation testing of the core, at the end of a slice
	cargo mutants -p grind-core

android: ## build the release APK and install it on the phone plugged over USB, keeping its data
	cargo tauri android build --apk --target aarch64
	adb install -r $(APK)

import-draft-android: ## push a draft backup to the phone over USB, for Settings to import: make import-draft-android BACKUP=path/to/backup.json
	@test -f "$(BACKUP)" || { echo "usage: make import-draft-android BACKUP=path/to/backup.json"; exit 1; }
	chmod 644 "$(BACKUP)"
	adb push "$(BACKUP)" $(DRAFT_ON_THE_PHONE)

mac: ## build the Mac app and install it in /Applications, keeping its data
	cargo tauri build --bundles app
	rm -rf /Applications/Grindstone.app
	cp -R target/release/bundle/macos/Grindstone.app /Applications/

import-draft-mac: ## copy a draft backup where the Mac app reads it, for Settings to import: make import-draft-mac BACKUP=path/to/backup.json
	@test -f "$(BACKUP)" || { echo "usage: make import-draft-mac BACKUP=path/to/backup.json"; exit 1; }
	install -m 644 "$(BACKUP)" "$(DRAFT_ON_THE_MAC)"

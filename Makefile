SHELL := bash
.SHELLFLAGS := -euo pipefail -c
.DEFAULT_GOAL := help

.PHONY: help dev test check test-ui mutants mutants-all android import-draft-android

APK := src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk
BASE ?= origin/main
DRAFT_ON_THE_PHONE := /sdcard/Android/data/com.maeevick.grindstone/files/Documents/grindstone-draft.json
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

mutants: ## mutation testing of the core lines changed since BASE (origin/main: what is not pushed yet), four at a time, at the end of a slice
	mkdir -p target
	git diff $(BASE) -- core > target/mutants.diff
	cargo mutants -p grind-core --in-diff target/mutants.diff --jobs 4

mutants-all: ## mutation testing of the whole core, four at a time, for an occasional sweep
	cargo mutants -p grind-core --jobs 4

android: ## build the release APK and install it on the phone plugged over USB, keeping its data
	cargo tauri android build --apk --target aarch64
	adb install -r $(APK)

import-draft-android: ## push a draft backup to the phone over USB, for Settings to import: make import-draft-android BACKUP=path/to/backup.json
	@test -f "$(BACKUP)" || { echo "usage: make import-draft-android BACKUP=path/to/backup.json"; exit 1; }
	chmod 644 "$(BACKUP)"
	adb push "$(BACKUP)" $(DRAFT_ON_THE_PHONE)

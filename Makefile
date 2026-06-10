SHELL := bash
.SHELLFLAGS := -euo pipefail -c
.DEFAULT_GOAL := help

.PHONY: help dev test check test-ui mutants android mac

APK := src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk
WIRING := (src-tauri/src/(main|lib|configurator)|ui/src/(main|app|tauri))\.rs

help: ## list the targets
	@grep -E '^[a-zA-Z0-9_-]+:.*## ' $(MAKEFILE_LIST) | awk -F':.*## ' '{printf "  %-10s %s\n", $$1, $$2}'

dev: ## run the app on the Mac, reloading on change
	cargo tauri dev

test: ## every Rust test
	cargo test --workspace

check: ## formatting, lints, tests and no uncovered line outside the wiring, as before a commit
	cargo fmt --all --check
	cargo clippy --workspace --all-targets -- -D warnings
	cargo clippy -p grind-ui --target wasm32-unknown-unknown --all-targets -- -D warnings
	cargo llvm-cov --workspace --ignore-filename-regex '$(WIRING)' --fail-uncovered-lines 0 --summary-only

test-ui: ## the screen's tests in a headless Chrome, driven by the ChromeDriver of the installed Chrome rather than the newest
	driver=$$(scripts/chromedriver-for-installed-chrome) && wasm-pack test --headless --chrome --chromedriver "$$driver" ui

mutants: ## mutation testing of the core, at the end of a slice
	cargo mutants -p grind-core

android: ## build the release APK and install it on the phone plugged over USB, keeping its data
	cargo tauri android build --apk --target aarch64
	adb install -r $(APK)

mac: ## build the Mac app and install it in /Applications, keeping its data
	cargo tauri build --bundles app
	rm -rf /Applications/Grindstone.app
	cp -R target/release/bundle/macos/Grindstone.app /Applications/

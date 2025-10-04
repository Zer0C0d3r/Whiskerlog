# Whiskerlog Makefile

# Variables
BINARY_NAME := whiskerlog
TARGET_DIR := target
RELEASE_DIR := $(TARGET_DIR)/release
DEBUG_DIR := $(TARGET_DIR)/debug
INSTALL_DIR := /usr/local/bin
CONFIG_DIR := ~/.config/whiskerlog
DATA_DIR := ~/.local/share/whiskerlog

# Default target
.PHONY: all
all: build

# Build targets
.PHONY: build
build:
	cargo build

.PHONY: release
release:
	cargo build --release

.PHONY: debug
debug:
	cargo build

# Development targets
.PHONY: dev
dev:
	cargo run

.PHONY: watch
watch:
	cargo watch -x run

# Testing targets
.PHONY: test
test:
	cargo test

.PHONY: test-release
test-release:
	cargo test --release

.PHONY: test-verbose
test-verbose:
	cargo test -- --nocapture

# Code quality targets
.PHONY: check
check:
	cargo check

.PHONY: clippy
clippy:
	cargo clippy --all-targets --all-features -- -D warnings

.PHONY: fmt
fmt:
	cargo fmt --all

.PHONY: fmt-check
fmt-check:
	cargo fmt --all -- --check

.PHONY: audit
audit:
	cargo audit

# Documentation targets
.PHONY: doc
doc:
	cargo doc --no-deps --open

.PHONY: doc-private
doc-private:
	cargo doc --no-deps --document-private-items --open

# Benchmarking
.PHONY: bench
bench:
	cargo bench

# Installation targets
.PHONY: install
install: release
	sudo cp $(RELEASE_DIR)/$(BINARY_NAME) $(INSTALL_DIR)/
	sudo chmod +x $(INSTALL_DIR)/$(BINARY_NAME)
	@echo "Installed $(BINARY_NAME) to $(INSTALL_DIR)"

.PHONY: install-user
install-user: release
	mkdir -p ~/.local/bin
	cp $(RELEASE_DIR)/$(BINARY_NAME) ~/.local/bin/
	chmod +x ~/.local/bin/$(BINARY_NAME)
	@echo "Installed $(BINARY_NAME) to ~/.local/bin"
	@echo "Add ~/.local/bin to your PATH if not already done"

.PHONY: uninstall
uninstall:
	sudo rm -f $(INSTALL_DIR)/$(BINARY_NAME)
	@echo "Uninstalled $(BINARY_NAME) from $(INSTALL_DIR)"

.PHONY: uninstall-user
uninstall-user:
	rm -f ~/.local/bin/$(BINARY_NAME)
	@echo "Uninstalled $(BINARY_NAME) from ~/.local/bin"

# Cleanup targets
.PHONY: clean
clean:
	cargo clean

.PHONY: clean-data
clean-data:
	rm -rf $(CONFIG_DIR) $(DATA_DIR)
	@echo "Cleaned user data and config"

.PHONY: clean-all
clean-all: clean clean-data

# Development setup
.PHONY: setup
setup:
	rustup component add clippy rustfmt
	cargo install cargo-watch cargo-audit
	@echo "Development environment setup complete"

# CI/CD targets
.PHONY: ci
ci: fmt-check clippy test

.PHONY: ci-release
ci-release: fmt-check clippy test-release

# Cross-compilation targets
.PHONY: build-linux
build-linux:
	cargo build --release --target x86_64-unknown-linux-gnu

.PHONY: build-macos
build-macos:
	cargo build --release --target x86_64-apple-darwin

.PHONY: build-windows
build-windows:
	cargo build --release --target x86_64-pc-windows-gnu

.PHONY: build-all
build-all: build-linux build-macos build-windows

# Docker targets
.PHONY: docker-build
docker-build:
	docker build -t whiskerlog:latest .

.PHONY: docker-run
docker-run:
	docker run -it --rm -v ~/.bash_history:/root/.bash_history whiskerlog:latest

# Package targets
.PHONY: package
package: release
	mkdir -p dist
	tar -czf dist/$(BINARY_NAME)-linux-x86_64.tar.gz -C $(RELEASE_DIR) $(BINARY_NAME)
	@echo "Package created: dist/$(BINARY_NAME)-linux-x86_64.tar.gz"

# Performance targets
.PHONY: profile
profile:
	cargo build --release
	perf record --call-graph=dwarf $(RELEASE_DIR)/$(BINARY_NAME)
	perf report

.PHONY: flamegraph
flamegraph:
	cargo flamegraph --bin $(BINARY_NAME)

# Security targets
.PHONY: security
security: audit
	cargo deny check

# Help target
.PHONY: help
help:
	@echo "Whiskerlog Makefile Commands:"
	@echo ""
	@echo "Build:"
	@echo "  build         Build debug version"
	@echo "  release       Build release version"
	@echo "  build-all     Cross-compile for all platforms"
	@echo ""
	@echo "Development:"
	@echo "  dev           Run in development mode"
	@echo "  watch         Watch and rebuild on changes"
	@echo "  setup         Setup development environment"
	@echo ""
	@echo "Testing:"
	@echo "  test          Run tests"
	@echo "  test-release  Run tests in release mode"
	@echo "  bench         Run benchmarks"
	@echo ""
	@echo "Code Quality:"
	@echo "  check         Check code without building"
	@echo "  clippy        Run clippy linter"
	@echo "  fmt           Format code"
	@echo "  fmt-check     Check code formatting"
	@echo "  audit         Security audit"
	@echo ""
	@echo "Installation:"
	@echo "  install       Install globally (requires sudo)"
	@echo "  install-user  Install to ~/.local/bin"
	@echo "  uninstall     Uninstall globally"
	@echo ""
	@echo "Cleanup:"
	@echo "  clean         Clean build artifacts"
	@echo "  clean-data    Clean user data"
	@echo "  clean-all     Clean everything"
	@echo ""
	@echo "CI/CD:"
	@echo "  ci            Run CI checks"
	@echo "  package       Create distribution package"
	@echo ""
	@echo "Documentation:"
	@echo "  doc           Generate and open docs"
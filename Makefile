# Makefile for Rust Backend Development

# Default target
.PHONY: help
help:
	@echo "Available commands:"
	@echo "  make dev          - Run with hot reload (auto-restart on changes)"
	@echo "  make run          - Run normally (no auto-reload)"
	@echo "  make build        - Build the project"
	@echo "  make check        - Check for compilation errors"
	@echo "  make clean        - Clean build artifacts"
	@echo ""
	@echo "Note: For hot reload, ensure cargo-watch is installed:"
	@echo "  cargo install cargo-watch"

# Run with hot reload - automatically restarts on file changes
.PHONY: dev
dev:
	cd app && cargo watch -x run

# Run normally without hot reload
.PHONY: run
run:
	cd app && cargo run

# Build the project
.PHONY: build
build:
	cargo build --release

# Check for errors without running
.PHONY: check
check:
	cargo check

# Clean build artifacts
.PHONY: clean
clean:
	cargo clean

# Install cargo-watch if not present
.PHONY: install-watch
install-watch:
	cargo install cargo-watch

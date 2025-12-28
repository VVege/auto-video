.PHONY: help build release check test clean run install fmt clippy

help:
	@echo "Auto-Video Makefile"
	@echo ""
	@echo "Available targets:"
	@echo "  build    - Build debug version"
	@echo "  release  - Build optimized release version"
	@echo "  check    - Check code without building"
	@echo "  test     - Run tests"
	@echo "  clean    - Clean build artifacts"
	@echo "  run      - Run with example text"
	@echo "  install  - Install binary to system"
	@echo "  fmt      - Format code"
	@echo "  clippy   - Run clippy linter"

build:
	cargo build

release:
	cargo build --release
	@strip target/release/auto-video 2>/dev/null || true
	@echo "Binary built: target/release/auto-video"

check:
	cargo check

test:
	cargo test

clean:
	cargo clean
	rm -rf output/
	rm -f *.mp4 *.mp3 *.png

run:
	cargo run -- --file example.txt --output example.mp4

install: release
	cp target/release/auto-video /usr/local/bin/
	@echo "Installed to /usr/local/bin/auto-video"

fmt:
	cargo fmt

clippy:
	cargo clippy -- -D warnings

# 开发辅助命令
dev-setup:
	@echo "Setting up development environment..."
	@command -v cargo >/dev/null 2>&1 || { echo "Rust not installed. Installing..."; curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh; }
	@command -v ffmpeg >/dev/null 2>&1 || { echo "FFmpeg not installed. Please install it manually."; exit 1; }
	@test -f .env || { echo "Creating .env from .env.example..."; cp .env.example .env; echo "Please edit .env and add your API key."; }
	@echo "Development environment ready!"

# 示例运行
example-short:
	cargo run -- \
		--text "这是一个美丽的春天，花儿开放，鸟儿歌唱。" \
		--output spring.mp4

example-file:
	cargo run -- --file example.txt --output example.mp4

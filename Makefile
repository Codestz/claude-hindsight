.PHONY: dev build clean web-build web-dev check publish

# Default: build everything
all: build

# Run both servers in development mode (requires tmux or run manually)
dev:
	@echo "Starting development servers..."
	@echo "  Terminal 1: cargo run -- serve --port 7227"
	@echo "  Terminal 2: cd web && npm run dev"
	@echo ""
	@echo "Tip: run 'make dev-rust' and 'make dev-web' in separate terminals"

dev-rust:
	cargo run -- serve --port 7227

dev-web:
	cd web && npm run dev

# Build Next.js static bundle + Rust release binary (binary embeds the bundle)
build: web-build
	cargo build --release

# Build only the Next.js static bundle
web-build:
	cd web && npm install && npm run build

# Clean build artifacts
clean:
	cargo clean
	rm -rf web/.next web/out

# Quick verification: start server and check health + static asset
check:
	@echo "Starting server for health check..."
	cargo run -- serve --port 7227 &
	@sleep 2
	@curl -s http://localhost:7227/api/health | python3 -m json.tool || echo "Server not responding"
	@curl -sI http://localhost:7227/ | head -5 || echo "Static serve not responding"
	@pkill -f "hindsight serve" 2>/dev/null || true

# Publish to crates.io (requires CRATES_IO_TOKEN or prior `cargo login`)
publish: web-build
	cargo publish

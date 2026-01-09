.PHONY: help build test clean docker-build docker-up docker-down migrate dev

# Default target
help:
	@echo "ArchiveStream - Available Commands:"
	@echo ""
	@echo "  make build         - Build all Rust binaries"
	@echo "  make test          - Run all tests"
	@echo "  make clean         - Clean build artifacts"
	@echo "  make docker-build  - Build all Docker images"
	@echo "  make docker-up     - Start all services with Docker Compose"
	@echo "  make docker-down   - Stop all services"
	@echo "  make migrate       - Run database migrations"
	@echo "  make dev           - Start development environment"
	@echo "  make fmt           - Format code"
	@echo "  make lint          - Run linters"
	@echo ""

# Build all Rust binaries
build:
	cargo build --release

# Run all tests
test:
	cargo test --workspace

# Clean build artifacts
clean:
	cargo clean
	cd apps/web-ui && rm -rf .next node_modules

# Build Docker images
docker-build:
	docker build -f Dockerfile.crawler -t archivestream/crawler:latest .
	docker build -f Dockerfile.api -t archivestream/api:latest .
	docker build -f Dockerfile.indexer -t archivestream/indexer:latest .
	docker build -f Dockerfile.ui -t archivestream/ui:latest .

# Start all services
docker-up:
	docker-compose -f docker-compose.prod.yml up -d

# Stop all services
docker-down:
	docker-compose -f docker-compose.prod.yml down

# Run database migrations
migrate:
	@if [ -z "$$DATABASE_URL" ]; then \
		echo "Error: DATABASE_URL not set"; \
		exit 1; \
	fi
	@for migration in infra/migrations/*.sql; do \
		echo "Running $$migration..."; \
		psql $$DATABASE_URL < $$migration; \
	done

# Start development environment
dev:
	@echo "Starting infrastructure..."
	docker-compose -f infra/compose.yml up -d
	@echo "Waiting for services to be ready..."
	sleep 5
	@echo "Running migrations..."
	make migrate
	@echo "Starting services..."
	@echo "Run these in separate terminals:"
	@echo "  cargo run --bin crawler"
	@echo "  cargo run --bin indexer"
	@echo "  cargo run --bin archive-api"
	@echo "  cd apps/web-ui && npm run dev"

# Format code
fmt:
	cargo fmt
	cd apps/web-ui && npm run format

# Run linters
lint:
	cargo clippy -- -D warnings
	cd apps/web-ui && npm run lint

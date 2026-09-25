# TimeTable — development commands.
# Recipes are kept compatible with both sh and cmd.exe (GNU make on Windows).

# Override to use a watcher, e.g. `make dev BACKEND_RUN="cargo watch -x run"`.
BACKEND_RUN ?= cargo run

.DEFAULT_GOAL := help
.PHONY: help env install dev dev-frontend dev-backend build build-frontend build-backend run \
        check test fmt redis-up redis-down docker-build docker-up docker-down docker-logs clean

help: ## Show this help
	@node -e "require('fs').readFileSync('Makefile','utf8').split('\n').forEach(l=>{const m=l.match(/^([a-z-]+):.*?## (.*)/);if(m)console.log(m[1].padEnd(15),m[2])})"

env: ## Create .env from .env.example (if missing)
	@node -e "const fs=require('fs');if(fs.existsSync('.env'))console.log('.env already exists');else{fs.copyFileSync('.env.example','.env');console.log('created .env')}"

install: env ## Install frontend and backend dependencies
	cd frontend && pnpm install
	cd backend && cargo fetch

dev: ## Run frontend (http://localhost:4321, hot reload) and backend together
	$(MAKE) -j2 dev-backend dev-frontend

dev-frontend: ## Run the Astro dev server (proxies /api to the backend)
	cd frontend && pnpm dev

dev-backend: ## Run the backend (in-memory cache unless REDIS_URL is set)
	cd backend && $(BACKEND_RUN)

build: build-frontend build-backend ## Build frontend (frontend/dist) and backend (release)

build-frontend:
	cd frontend && pnpm build

build-backend:
	cd backend && cargo build --release

run: build-frontend ## Build the frontend and run the release backend serving it (production-like)
	cd backend && cargo run --release

check: ## Lint and type-check everything, run tests
	cd backend && cargo fmt --check
	cd backend && cargo clippy --all-targets -- -D warnings
	cd backend && cargo test
	cd frontend && pnpm check

test: ## Run backend tests
	cd backend && cargo test

fmt: ## Format backend code
	cd backend && cargo fmt

redis-up: ## Start a throwaway local Redis on :6379 (then set REDIS_URL=redis://127.0.0.1:6379)
	docker run -d --rm --name timetable-redis-dev -p 6379:6379 redis:8-alpine

redis-down: ## Stop the local Redis
	docker stop timetable-redis-dev

docker-build: ## Build the docker image
	docker compose build

docker-up: ## Start the compose stack (service + redis)
	docker compose up -d --build

docker-down: ## Stop the compose stack
	docker compose down

docker-logs: ## Follow the compose logs
	docker compose logs -f

clean: ## Remove build outputs
	cd backend && cargo clean
	node -e "for(const d of ['frontend/dist','frontend/.astro'])require('fs').rmSync(d,{recursive:true,force:true})"

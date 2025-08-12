DOCKER_COMPOSE := docker compose -f docker/docker-compose.yml

# Configurables (puedes sobreescribirlos en la línea de comandos)
FALKOR_URL ?= redis://127.0.0.1:6379
GRAPH_NAME ?= code_graph
ANALYZE_PATH ?= ./examples/python
MESSAGE ?= demo run

.PHONY: docker-up docker-down docker-logs docker-restart

## Start FalkorDB + RedisInsight stack
docker-up:
	$(DOCKER_COMPOSE) up -d

## Stop and remove stack
docker-down:
	$(DOCKER_COMPOSE) down

## Tail logs from both services
docker-logs:
	$(DOCKER_COMPOSE) logs -f --tail=200

## Restart stack
docker-restart: docker-down docker-up

.PHONY: app-build app-analyze app-analyze-path app-query demo graph-clean smoke

## Build the CLI in release mode
app-build:
	cargo build --release

## Analyze a sample project using the FalkorDB from docker-compose
app-analyze: app-build
	FALKORDB_URL=$(FALKOR_URL) ./target/release/ccg analyze --path ./examples/python --message "demo run"

## Analyze a custom path with optional commit message
app-analyze-path: app-build
	FALKORDB_URL=$(FALKOR_URL) ./target/release/ccg analyze --path $(ANALYZE_PATH) --message "$(MESSAGE)"

## Run a sample ad-hoc query against the graph
app-query: app-build
	FALKORDB_URL=$(FALKOR_URL) ./target/release/ccg query --question "List functions and their imports"

## One-liner demo: start services, analyze, and open dashboard URL
demo: docker-up app-analyze
	@echo "Open RedisInsight at: http://localhost:5540"

## Clear graph contents (MATCH (n) DETACH DELETE n)
graph-clean:
	@echo "Clearing graph '$(GRAPH_NAME)' on $(FALKOR_URL)"
	@docker exec falkordb redis-cli -h 127.0.0.1 -p 6379 GRAPH.QUERY "$(GRAPH_NAME)" "MATCH (n) DETACH DELETE n" >/dev/null 2>&1 || true

## End-to-end smoke test: start stack, clear graph, analyze custom path, basic query
smoke: docker-up graph-clean app-analyze-path
	FALKORDB_URL=$(FALKOR_URL) ./target/release/ccg query --question "Show functions" || (echo "Smoke test query failed" && exit 1)
	@echo "Smoke test completed. Open http://localhost:5540 to inspect the graph."

LAB ?= lab1_produce_consume
CONFIG_DIR := labs/$(LAB)
CONFIG_KEYED := $(CONFIG_DIR)/config_keyed.toml
CONFIG_RR    := $(CONFIG_DIR)/config_roundrobin.toml
COMPOSE = docker compose

# Default groups (override with: make consumer-keyed GROUP=my-group)
GROUP ?= demo-consumer-group
GROUP_RR ?= demo-consumer-group-rr

up: down
	$(COMPOSE) up -d

down:
	$(COMPOSE) down -v

build:
	cargo build

# ----- Convenience runners for Lab 1 -----

consumer-keyed:
	cargo run -p $(LAB) --bin consumer -- --config $(CONFIG_KEYED) --group-id $(GROUP)

producer-keyed:
	cargo run -p $(LAB) --bin producer -- --config $(CONFIG_KEYED)

consumer-rr:
	cargo run -p $(LAB) --bin consumer -- --config $(CONFIG_RR) --group-id $(GROUP_RR)

producer-rr:
	cargo run -p $(LAB) --bin producer -- --config $(CONFIG_RR)
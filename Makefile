# Default lab and group
LAB ?= lab1_produce_consume
PROFILE ?= lab1.keyed
GROUP ?= demo-consumer-group
CONFIG := shared/config.toml
COMPOSE = docker compose

up: down
	$(COMPOSE) up -d

down:
	$(COMPOSE) down -v

build:
	cargo build

# ----- Generic runners with PROFILE=lab2.default, etc -----

consumer:
	cargo run -p $(LAB) --bin consumer -- \
		--profile $(PROFILE) \
		$(if $(GROUP),--group-id $(GROUP),) \
		$(if $(FAIL_MOD),--fail-mod $(FAIL_MOD),) \
		$(if $(FAIL_ACTION),--fail-action $(FAIL_ACTION),)

producer:
	cargo run -p $(LAB) --bin producer -- --profile $(PROFILE)

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
	
# ---------- Lab 4: Delivery semantics ----------
# At-most-once: commit pre-processing
l4-consumer-atmost:
	$(MAKE) consumer \
		LAB=lab4_delivery_semantics \
		PROFILE=lab4.atmostonce \
		GROUP=lab4-atmost \
		ARGS="--commit-mode pre --fail-mod 2"

l4-consumer-atmost-no-crashing:
	$(MAKE) consumer \
		LAB=lab4_delivery_semantics \
		PROFILE=lab4.atmostonce \
		GROUP=lab4-atmost \
		ARGS="--commit-mode pre --fail-mod 0"

l4-producer-atmost:
	$(MAKE) producer \
		LAB=lab4_delivery_semantics \
		PROFILE=lab4.atmostonce

# At-least-once: commit post-processing
l4-consumer-atleast:
	$(MAKE) consumer \
		LAB=lab4_delivery_semantics \
		PROFILE=lab4.atleastonce \
		GROUP=lab4-atleast \
		ARGS="--commit-mode post --fail-mod 2"

l4-consumer-atleast-no-crashing:
	$(MAKE) consumer \
		LAB=lab4_delivery_semantics \
		PROFILE=lab4.atleastonce \
		GROUP=lab4-atleast \
		ARGS="--commit-mode post --fail-mod 0"

l4-producer-atleast:
	$(MAKE) producer \
		LAB=lab4_delivery_semantics \
		PROFILE=lab4.atleastonce

# EOS (conceptual/stub in current lab)
l4-consumer-eos:
	$(MAKE) consumer \
		LAB=lab4_delivery_semantics \
		PROFILE=lab4.exactlyonce \
		GROUP=lab4-eos \
		ARGS="--commit-mode txn"

l4-producer-eos:
	$(MAKE) producer \
		LAB=lab4_delivery_semantics \
		PROFILE=lab4.exactlyonce

# ----- Generic runners with PROFILE=lab2.default, etc -----

consumer:
	cargo run -p $(LAB) --bin consumer -- \
		--profile $(PROFILE) \
		$(if $(GROUP),--group-id $(GROUP),) \
		$(if $(FAIL_MOD),--fail-mod $(FAIL_MOD),) \
		$(if $(FAIL_ACTION),--fail-action $(FAIL_ACTION),) \
		$(ARGS)

producer:
	cargo run -p $(LAB) --bin producer -- --profile $(PROFILE) \
		$(ARGS)

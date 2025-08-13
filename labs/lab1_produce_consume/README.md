# Lab 1 – Basic Kafka Producer/Consumer in Rust

## Overview
This lab implements a Kafka producer and consumer in Rust using the `rdkafka` crate.  
Configuration is loaded from a TOML file. Both producer and consumer can be pointed to different config files at runtime.

Two partitioning modes are supported for the producer:
- **Keyed**: Messages include a key, ensuring the same key always goes to the same partition.
- **Round-robin**: Messages have no key and are distributed across partitions in round-robin order.


## Project structure
- **shared/** – Common helpers and config loading.
- **labs/lab1_produce_consume/** – Producer and consumer binaries.
- **docker-compose.yml** (root) – Local Kafka broker setup.
- **Makefile** (root) – Commands to start/stop Kafka and create topics.
- **config_keyed.toml** – Producer sends messages with keys (per-key partitioning).
- **config_roundrobin.toml** – Producer sends messages without keys (round-robin partitioning).

## Prerequisites
- Docker & Docker Compose
- Rust (`cargo`, `rustup`)

## Setup
1. Start Kafka:
```bash
make up
```
2. Build the workspace:
```bash
make build
```
## Running
### 1. Keyed mode (per-key partitioning)
```bash
# Terminal A
make consumer-keyed

# Terminal B
make producer-keyed
```

Type messages in the producer:
```bash
u1 click 10
u2 purchase 99
u1 view 1
```

### 2. Round-robin mode (no key)
```bash
# Terminal A
make consumer-rr

# Terminal B
make producer-rr
```

Type messages in the producer:
```bash
u1 click 10
u2 purchase 99
u1 view 1
```

## Key takeaways
1. **Keys determine partition assignment**
    - Messages with the same key always go to the same partition (ensuring per-key ordering).
    - Messages without a key are distributed round-robin across partitions.
2. **Offsets are per partition**
    - Offsets increment independently in each partition.
    - They are not global sequence numbers for the whole topic.
3. **Consumer group state is tied to group.id**
    - Consumers in the same group share the work of reading partitions.
    - Change the group.id and Kafka will treat you as a completely new group starting from the configured auto.offset.reset.
4. **Producer send is asynchronous with delivery reports**
    - The FutureProducer returns a delivery result later, indicating partition & offset on success, or the failed message and error on failure.
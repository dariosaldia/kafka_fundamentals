# Lab 1 - Exploring Kafka Partitioning in Rust

This lab demonstrates how Kafka distributes messages across partitions and how partitioning strategies influence message ordering and parallelism. Using a Rust producer and consumer built with the rdkafka crate, it compares two common approaches:
    - **Keyed partitioning**: Messages include a key, ensuring all messages with the same key are routed to the same partition, preserving their order.
    - **Round-robin partitioning**: Messages have no key and are evenly distributed across partitions, maximizing throughput but without per-key ordering guarantees.

The lab is designed to provide a clear, hands-on view of how partitioning affects message flow in Kafka.

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
## 🧪 Running the lab
### 1. Keyed mode (per-key partitioning)

In this mode, the producer attaches a **key** (here, `user_id`) to each message.  
Kafka's default partitioner will always send messages with the same key to the **same partition**.  
This preserves the order for that key.

```bash
# Terminal A: Run consumer
make consumer

# Terminal B: Run producer with keyed profile
make producer
```

Enter messages in the producer terminal:
```bash
u1 click 10
u2 purchase 99
u1 view 1
```

**What to look for:**
- Messages from u1 will always go to the same partition (e.g., p0 or p1).
- Messages from u2 will consistently go to a different partition.
- In the consumer output, note that offsets are per partition. For example:

```
partition=2 @ offset=0 key=Some("u1") => Event {...}
partition=0 @ offset=0 key=Some("u2") => Event {...}
partition=2 @ offset=1 key=Some("u1") => Event {...}
```

### 2. Round-robin mode (no key)

In this mode, the producer **does not send a key** with messages.
Kafka’s default partitioner distributes them evenly across partitions in **round-robin** order.

> **INFO**: You might want to run `make up` before proceeding to start fresh with Kafka

```bash
# Terminal A: Run consumer (same profile works regardless of keying)
make consumer

# Terminal B: Run producer with round-robin profile
make producer PROFILE=lab1.roundrobin
```

Enter messages in the producer terminal:
```bash
u1 click 10
u2 purchase 99
u1 view 1
```

**What to look for:**
- Messages will be assigned to partitions in a rotating sequence:

```
partition=1 @ offset=0 key=None => Event {...}
partition=2 @ offset=0 key=None => Event {...}
partition=0 @ offset=0 key=None => Event {...}
```

Because there is no key, **ordering across messages for the same user_id is not guaranteed**.

## 💡 Key takeaways

1. **Keys determine partition assignment**  
    - Messages with the same key always go to the same partition, preserving per-key ordering.  
    - Messages without a key are distributed round-robin across partitions.
2. **Offsets are per partition**  
    - Each partition maintains its own offset sequence.  
    - Offsets are not global for the entire topic.
3. **Partition choice impacts ordering and parallelism**  
    - Keyed partitioning preserves order but can limit parallelism for hot keys.  
    - Round-robin partitioning maximizes throughput but does not guarantee ordering across messages without a key.  
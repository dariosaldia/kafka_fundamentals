# Kafka Fundamentals – Recap

This recap gathers the essential concepts, clarifications, and lessons from the Kafka Fundamentals labs. It contains no code — only explanations and takeaways.

## Core Concepts

### Topics, Partitions, and Offsets
- A **topic** is a named stream of records.
- Each topic is split into **partitions**. Partitions are append-only logs.
- Each record within a partition has a unique, sequential **offset**.
- **Offsets are per-partition**, not global across the whole topic.

### Keys and Partitioning
- Producers can send messages with or without keys.
  - **With key** → all records with the same key go to the same partition (guaranteeing per-key ordering).
  - **Without key** → records are distributed round-robin across partitions.

### Consumer Groups
- A **consumer group** is a set of consumers sharing a group.id.
- Kafka assigns partitions of a topic to consumers in the group.
- Consumers in the same group split the work; different groups each receive the full stream.
- Changing the group.id makes Kafka treat the consumer as a new independent group.

### Producer Basics
- Producers write to topics and receive acknowledgments (`acks`).
- Kafka producers retry on failure by default.
- Delivery reports tell the producer whether a record was successfully written and to which partition/offset.

## Delivery Semantics

Kafka supports three broad semantics:

1. **At-most-once** – Messages may be lost, but are never processed twice.  
2. **At-least-once (default)** – Messages are never lost, but may be processed more than once (duplicates possible).  
3. **Exactly-once** – Each message is processed once and only once (requires additional configuration).

### How Kafka Achieves At-Least-Once (Default)

**Producer side**
- Retries are enabled by default. If a send fails transiently, the producer retries until a broker acknowledges it.
- This avoids silent data loss but can create **duplicates** if the first attempt actually succeeded but the ack was lost.

**Consumer side**
- `enable.auto.commit = true` by default → periodic **commits after polling** (and typically after application processing).
- If the app **crashes after processing but before the next commit**, Kafka doesn’t see the offset and **re-delivers** the message on restart.
- Result: **no message loss**, but **duplicates are possible** (producer retries; consumer replay).

**Net effect:** Kafka’s defaults produce **at-least-once** semantics end to end.

### How to Implement At-Most-Once (Deliberately)

Goal: **never process a message twice**, accepting that some messages might be **lost**.

**Consumer pattern**
- **Commit before processing** each message or batch.
- Or keep `enable.auto.commit=true` with very frequent commits, and process immediately after poll (risk still exists within the commit interval).

**Producer pattern (optional)**
- Consider reducing reliability to avoid duplicates at the source:
  - Disable retries (`retries=0`) and/or set `acks=0` to avoid uncertain duplicate writes.
  - ⚠️ This increases **loss risk** if brokers are transiently unavailable.

**Trade-offs**
- Pros: No duplicates in the application.
- Cons: **Data loss** is possible if the app crashes after commit but before completing the work; with `acks=0` or no retries, messages can also be lost **before** they reach Kafka.

**When it fits**
- Fire-and-forget metrics, ephemeral telemetry, or cases where occasional loss is acceptable and deduplicating would be more costly than dropping.

### Exactly-once Semantics (EOS)

- Requires **idempotent producers** and **transactions**.
- Producers begin a transaction, send records, and commit offsets within that transaction.
- Kafka commits or aborts atomically, ensuring no duplicates and no loss.
- EOS is more advanced and requires careful configuration.

## Further Reading
- [Confluent Docs – Kafka Message Delivery Guarantees](https://docs.confluent.io/kafka/design/delivery-semantics.html#ak-message-delivery-guarantees)

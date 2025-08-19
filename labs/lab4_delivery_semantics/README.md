# Lab 4 – Delivery Semantics in Kafka

This lab explores how **delivery semantics** (at-most-once, at-least-once, exactly-once) manifest in Kafka, and how they depend on **when offsets are committed** relative to message processing. It demonstrates how consumer logic and configuration affect whether messages are lost, duplicated, or delivered exactly once.

## Setup

**1. Start Kafka:**
```bash
make up
```

**2. Build the workspace:**
```bash
make build
```

## 🧪 Running the Lab

### 1. At-most-once delivery (commit *before* processing)

Run the consumer:
```bash
make l4-consumer-atmost
```

Run the producer:
```bash
make l4-producer-atmost
```

Type inputs like:
```bash
u1 click 1
u2 purchase 2
u3 view 3
```

Restart the consumer without crashing and wait a few seconds. No activity should be displayed in the terminal.
```bash
make l4-consumer-atmost-no-crashing
```

**Expected behavior:**
- Offsets are committed *before* processing.
- If processing fails, the message is lost (not redelivered).
- Example:
```
✅ COMMIT (pre) p2 @ 0
✅ PROCESSED (pre) p2 @ 0 key=Some("u1") => Event { user_id: "u1", action: "click", value: 1 }

✅ COMMIT (pre) p0 @ 0
❌ PROCESSING FAILED p0 @ 0 key=Some("u2") => Event { user_id: "u2", action: "purchase", value: 2 }

✅ COMMIT (pre) p2 @ 1
✅ PROCESSED (pre) p2 @ 1 key=Some("u3") => Event { user_id: "u3", action: "view", value: 3 }
```
Notice how the failed message will not be retried, even after restarting the consumer.

### 2. At-least-once delivery (commit *after* processing)

Run the consumer:
```bash
make l4-consumer-atleast
```

Run the producer:
```bash
make l4-producer-atleast
```

Type inputs like:
```bash
u1 click 1
u2 purchase 2
u3 view 3
```

**Expected behavior:**
- Messages are processed first, then committed.
- If processing fails, the message is retried on restart (duplicates possible).
- Example:
```
✅ PROCESSED (post) p2 @ 0 key=Some("u1") => Event { user_id: "u1", action: "click", value: 1 }
✅ COMMIT (post) p2 @ 0

❌ PROCESSING FAILED p0 @ 0 key=Some("u2") => Event { user_id: "u2", action: "purchase", value: 2 }

✅ PROCESSED (post) p2 @ 1 key=Some("u3") => Event { user_id: "u3", action: "view", value: 3 }
✅ COMMIT (post) p2 @ 1
```

Restart the consumer without crashing and wait until the failed message gets reprocessed
```bash
make l4-consumer-atleast-no-crashing
```

After a few seconds...
```
✅ PROCESSED (post) p0 @ 0 key=Some("u2") => Event { user_id: "u2", action: "purchase", value: 2 }
✅ COMMIT (post) p0 @ 0
```
Duplicates occur because offset commit lags behind processing.

### 3. Exactly-once delivery (conceptual)

Exactly-once semantics require **idempotent producers** and **transactions**:
- The producer starts a transaction.
- Writes messages and consumer offsets as part of the transaction.
- Commits or aborts the transaction atomically.

This lab sets up the configuration (`lab4.exactlyonce`) but does not fully implement EOS. A later lab will extend this.

## 🧼 Behavior & Expected Output

| Mode            | Commit timing | Failure effect                          | Crash effect                         |
|-----------------|---------------|-----------------------------------------|---------------------------------------|
| At-most-once    | Before        | Message lost (never retried)            | Already committed, so skipped         |
| At-least-once   | After         | Message retried → possible duplicates   | Uncommitted messages replayed         |
| Exactly-once    | Transactional | No loss, no duplicates (with EOS)       | Atomic commit/abort ensures consistency |

## 💡 Key Takeaways

- **Kafka defaults to at-least-once delivery.**
  - Because offsets are committed *after* processing by default.
  - Guarantees no message loss, but duplicates may occur.
- **At-most-once** is possible by committing offsets *before* processing.
  - Simpler, but risks data loss.
- **Exactly-once semantics** (EOS) require idempotent producers and transactions.
  - Kafka ensures this by atomically committing both writes and offset progress.
- Delivery semantics are not hard-coded in Kafka; they depend on **consumer commit logic**.


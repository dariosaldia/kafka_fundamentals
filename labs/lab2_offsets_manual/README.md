# Lab 2 – Manual Offsets & Re-delivery

This lab demonstrates how Kafka’s offset management influences message re-delivery.  
Kafka tracks the position of each consumer within its assigned partitions using offsets. By default, these offsets are committed automatically, but in this lab, automatic commits are disabled so offsets are only committed after message processing is confirmed.

Through controlled message failures, the lab illustrates:
- How uncommitted messages are re-delivered after a restart or rebalance.
- How manual offset control can ensure messages are acknowledged only after successful processing.
- Why newer messages may still be delivered before earlier uncommitted ones, as determined by Kafka’s delivery guarantees.

The exercises provide a practical view of how offset commits, delivery guarantees, and consumer behavior interact in real-world conditions.

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

### 1. Run the manual consumer

Start the consumer specifying the profile and group ID:

```bash
make consumer \
  LAB=lab2_offsets_manual \
  PROFILE=lab2.default \
  GROUP=lab2-consumer
```
You can simulate failures in two different ways:

**When `value` is divisible by a number**
```bash
make consumer \
  LAB=lab2_offsets_manual \
  PROFILE=lab2.default \
  GROUP=lab2-test \
  -- FAIL_MOD=3
```
**When `action` equals "something"**
```bash
make consumer \
  LAB=lab2_offsets_manual \
  PROFILE=lab2.default \
  GROUP=lab2-test \
  -- FAIL_ACTION=purchase
```
> **INFO**: By default, a failure will occur when `FAIL_MOD=5`

### 2. Send messages using the producer

Run the producer with the same profile:

```bash
make producer \
  LAB=lab2_offsets_manual \
  PROFILE=lab2.default
```

Type inputs like:

```bash
u1 click 42
u2 purchase 30
u1 view 10
```

Each line corresponds to a message with user_id, action, and value.

## 🧼 Behavior & Expected Output

- Messages that simulate failure are not committed and will be re-delivered.
- Restarting the consumer will reprocess only the failed messages.
- You’ll see outputs like:

```bash
✅ COMMIT p1 @ 4 key=Some("u1") => Event { ... }
❌ Simulated failure p1 @ 5 key=Some("u2") => Event { ... }
```

Then on restart:

```bash
❌ Simulated failure p1 @ 5 ...
✅ COMMIT p1 @ 5 ...
```

## 💡 Key Takeaways

- ✅ **Manual offset control** is essential when processing might fail and you only want to commit after success.  
- 🌀 **Re-delivery** happens for uncommitted messages when restarting the consumer or rebalancing the group.  
- 🎯 **Message ordering is preserved per partition**, and Kafka will continue delivering subsequent messages in the same partition even if earlier ones have not been committed, unless the consumer explicitly stops polling or seeks.  
- 🔄 **Offset commits** decide the starting point for a consumer after it restarts, but while the consumer is running, commits do not change the order in which messages are delivered.
- 🔑 **Group IDs** determine the consumer’s identity. Changing the group will re-read from the beginning (if offset reset is earliest).

## 🔍 Common Misunderstanding: Why Kafka Still Delivers Uncommitted Messages


Kafka does **not** pause message delivery just because previous messages are uncommitted.
As long as the consumer keeps polling, the broker will continue sending newer messages, even if earlier ones haven’t had their offsets committed yet.

### 🧪 Example: Reproducing Delivery of Uncommitted Messages

You can see this behavior in action with the following steps:

1. **Start the producer** in a terminal:
   ```bash
   make producer \
     LAB=lab2_offsets_manual \
     PROFILE=lab2.default
   ```

2. **Start the consumer** in a separate terminal, simulating a failure when the value is divisible by 2:
   ```bash
   make consumer \
     LAB=lab2_offsets_manual \
     PROFILE=lab2.default \
     GROUP=lab2-test \
     -- FAIL_MOD=2
   ```

3. **Send three messages with the same key** (e.g., "uX") from the producer:
   ```
   uX click 1
   uX click 2
   uX click 3
   ```

4. **Observe the output** in the consumer terminal. You’ll see:
   ```
   ✅ COMMIT p1 @ 0 key=Some("uX") => Event { ... }
   ❌ Simulated failure p1 @ 1 key=Some("uX") => Event { ... }
   ✅ COMMIT p1 @ 2 key=Some("uX") => Event { ... }
   ```
   Here, the message at offset 1 (value=2) fails and is not committed, but the message at offset 2 (value=3) is still delivered and processed by the consumer.

> **Note:** This demonstrates that Kafka continues to deliver subsequent messages (offset 2) even when earlier messages (offset 1) have not been committed due to a simulated failure.

This is a consequence of Kafka's **delivery guarantees**:
- **At-least-once delivery** (default): Messages may be delivered more than once if their offsets are not committed.
- **Exactly-once semantics**: Possible, but requires extra configuration and coordination.

It’s important to distinguish:
- **Delivery guarantees** -> how often Kafka sends a message.
- **Processing guarantees** -> how often the application processes or applies that message.
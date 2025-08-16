# Lab 3 – Consumer Groups & Partition Rebalancing

This lab demonstrates how Kafka distributes work among consumers in the same group, and what happens when consumers join or leave.  
The focus is on how **consumer groups** scale message processing and how **partition rebalancing** redistributes assignments.
 

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

### 1. Start the producer

Run the producer to send messages into the topic:

```bash
make producer \
  LAB=lab3_consumer_groups \
  PROFILE=lab3.default
```

Type inputs like:

```bash
u1 click 1
u2 purchase 2
u3 view 3
```

### 2. Run a single consumer

Run the consumer with the same profile:

```bash
make consumer \
  LAB=lab3_consumer_groups \
  PROFILE=lab3.default \
  GROUP=lab3-group
```

**Expected behavior:**

All partitions are assigned to this one consumer.

### 3. Add another consumer to the same group

Open a new terminal and run:

```bash
make consumer \
  LAB=lab3_consumer_groups \
  PROFILE=lab3.default \
  GROUP=lab3-group
```

**Expected behavior:**

- Kafka rebalances partitions across the two consumers.
- Each consumer now handles a subset of the partitions.
- Logs show which consumer processed each message.

### 4. Stop one consumer

Stop one of the consumers and observe.

**Expected behavior:**
- Kafka rebalances again, assigning all partitions to the remaining consumer.
- No messages are lost, and processing continues.

## 🧼 Behavior & Expected Output

When the first consumer is up, all partitions will be assigned to it

```bash
[lab3-consumer--1] assignment: [p0, p1, p2]
[lab3-consumer--1] assignment: [p0, p1, p2]
[lab3-consumer--1] assignment: [p0, p1, p2]
```

After the second consumer is up, rebalancing happens

In the terminal of the first consumer:
```bash
[lab3-consumer--1] assignment: [p0, p1]
[lab3-consumer--1] assignment: [p0, p1]
[lab3-consumer--1] assignment: [p0, p1]
```

In the terminal of the second consumer:
```bash
[lab3-consumer--2] assignment: [p2]
[lab3-consumer--2] assignment: [p2]
[lab3-consumer--2] assignment: [p2]
```

Let's say we stop the first consumer, then in the terminal of the second consumer all the partitions will be assigned to the only running consumer:

```bash
[lab3-consumer--2] assignment: [p0, p1, p2]
[lab3-consumer--2] assignment: [p0, p1, p2]
[lab3-consumer--2] assignment: [p0, p1, p2]
```

## 💡 Key Takeaways

- 👥 **Consumer groups** enable horizontal scaling: partitions are split across consumers in the same group.
- 🔄 **Rebalancing** occurs whenever consumers join or leave a group.
- 📦 **Different groups** consume the same topic independently, allowing multiple applications to process the same data without interfering.

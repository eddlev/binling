# BinLing Benchmarks

**Status:** Definition Phase (Targeting v0.1 Implementation)

This directory contains the specifications for the **normative benchmark suite** of the BinLing language and Levin VM.

**Disclaimer:**
Architectural Concepts
While BinLing uses biological terminology ("Mitosis", "DNA", "Capsule") to describe its architecture, it is strictly a **Deterministic Virtual Machine**.

| Biological Metaphor | Technical Reality |
| :--- | :--- |
| **Capsule** | Self-contained Process / Actor |
| **Payload (DNA)** | Bytecode Instruction Stream |
| **Mitosis** | Recursive Process Forking |
| **Lattice** | Distributed Mesh Network / DHT |

This naming convention captures the system's *resilience* and *autonomy*, not its implementation.

To ensure credibility, all benchmarks defined here must be:
1.  **Runnable**: Must compile and execute.
2.  **Reproducible**: Must use statistical harnessing (Criterion.rs).
3.  **Comparative**: Must measure against a defined baseline (JSON/Protobuf), not arbitrary numbers.

---

## 1. The Harness

All benchmarks **MUST** use the **[Criterion.rs](https://github.com/bheisler/criterion.rs)** framework.
* **Statistical Confidence:** 95% confidence interval.
* **Sample Size:** Minimum 100 iterations per sample.
* **Warm-up:** Mandatory warm-up period to eliminate JIT/Cache noise.

---

## 2. Benchmark Suite A: The Codec (BLE)

These tests measure the efficiency of the **BinLing Encoding (BLE)** wire format.

### A1. Encode/Decode Throughput
* **Metric:** Gigabytes/second (GB/s).
* **Scenario:** Serialize and Deserialize a "Standard Capsule" (SS_64).
* **Payload:** Randomly generated BLIR ops (non-compressible) to test raw framing overhead.
* **Baseline:** `serde_json` serialization of an equivalent JSON object.
* **Goal:** BLE must outperform JSON by >10x in throughput.

### A2. Header Overhead
* **Metric:** Bytes (Overhead per Capsule).
* **Scenario:** Encode an empty capsule (Payload=0).
* **Goal:** Verify fixed overhead matches `HEADER_LEN` (122 bytes) exactly.

---

## 3. Benchmark Suite B: The Runtime (Levin VM)

These tests measure the **deterministic execution speed** of the Lattice VM.

### B1. Activation Latency (The "Firing Dot")
* **Metric:** Nanoseconds (ns) per activation.
* **Scenario:**
    1.  Pre-load a lattice with 1,000 capsules.
    2.  Capsule A activates Capsule B.
    3.  Measure time from `ACTIVATE(B)` signal to `B.Execute()` start.
* **Constraint:** Must be measured in a **single-threaded** context to prove baseline efficiency.

### B2. Scheduler Throughput (The "Rapid Fire")
* **Metric:** Capsules/second (Caps/sec).
* **Scenario:** "Flood Test" — 100 capsules all active in the same cycle.
* **Measurement:** Time to sort (Priority > Coord) and execute the full queue.
* **Goal:** > 1,000,000 capsules/sec on standard hardware.

---

## 4. Benchmark Suite C: Information Density

These tests measure the **"Holographic" Efficiency** of the protocol.

### C1. Intent Compression Ratio
* **Metric:** Ratio (Source Bytes / Encoded Bytes).
* **Scenario:** "Standard System Prompt" (approx. 500 words / 3KB).
    * **Source:** A standard English system prompt for an AI agent.
    * **Target:** Equivalent logic encoded into a BLE Policy Core (Q0–Q2).
* **Goal:** > 75% reduction in size.

---

## 5. Anti-Patterns (Forbidden)

* ❌ **No "Estimated" Metrics:** Do not commit READMEs saying "We expect X speed." Only commit results from `cargo bench`.
* ❌ **No "Hello World" Cheats:** Benchmarks must use filled capsules (SS_64 or larger) to simulate real workload mass.

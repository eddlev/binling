# BinLing Encoding v0.1 (BLE)

**Status:** Draft — targeting BinLing v0.1 specification freeze.

This document defines the **normative wire-level encoding** for BinLing capsules.
To maximize machine efficiency and eliminate tokenizer overhead, **BinLing v0.1 uses strictly ASCII encoding.**

---

## 1. Encoding Standard: ASCII (Industrial)

### 1.1 Rational
Visual symbols (Unicode/Emoji) incur a "Tokenizer Tax," often costing 3-4 tokens per character.
Standard ASCII characters (0x20–0x7E) are optimized in all major LLM tokenizers (BPE) to represent **1 Token per Byte**.

### 1.2 Normative Character Set
BinLing instructions **MUST** be constructed using only standard ASCII characters.
Use of multi-byte Unicode characters in the instruction stream is **FORBIDDEN** in v0.1.

### 1.3 Opcode Mapping (The "Industrial" Set)
Implementations MUST support the following single-byte mappings:

| Concept | ASCII | Hex | Semantics |
| :--- | :---: | :---: | :--- |
| **INTENT / ROOT** | `#` | `0x23` | The immutable core goal (Q0). |
| **EXECUTE** | `!` | `0x21` | Trigger activation or execution. |
| **VARIABLE** | `$` | `0x24` | Reference to a data container. |
| **QUERY / CHECK** | `?` | `0x3F` | Verification or conditional check. |
| **LOOP / CYCLE** | `@` | `0x40` | Iteration marker. |
| **BRANCH / IF** | `^` | `0x5E` | Conditional fork. |
| **TERMINATE** | `.` | `0x2E` | End of sequence / Void boundary. |
| **SEPARATOR** | `|` | `0x7C` | Field or stratum delimiter. |

---

## 2. Capsule Sizing & Capacity

### 2.1 Byte-Voxel Alignment
Because encoding is strictly 1-Byte ASCII:
* **1 Voxel = 1 Byte = 1 Character = 1 Token.**
* A Cube of size `SS_4` (4x4x4) holds exactly **64 Instructions/Bytes**.

### 2.2 Header Layout (Fixed)
The header remains fixed length (122 bytes minimum) but is now strictly ASCII-compatible where applicable.

| Field | Size | Description |
| :--- | :--- | :--- |
| `MAGIC` | 4 bytes | ASCII `"BLE1"` (0x42 0x4C 0x45 0x31) |
| `PAD_LEN` | 4 bytes | Number of zero-padding bytes (`0x00`) at end. |
| `POLICY_LEN`| 2 bytes | Length of the canonical Policy Core bytes. |

---

## 3. Padding & Integrity

### 3.1 Zero Padding
To maintain byte-alignment without tokenizer confusion:
* Padding bytes **MUST** be `0x00` (Null Byte).
* Padding length is defined by `PAD_LEN` in the header.

### 3.2 Integrity
* `CAPSULE_HASH` covers the entire byte array (Header + ASCII Payload + Zero Padding).
* Any deviation (e.g., using a Unicode quote `”` instead of ASCII `"`) causes a hash mismatch → **FAIL CLOSED**.

---

## 4. Future Roadmap (Non-Normative)

* **v2.0 (Visual Tokens):** Future versions may map functions to single pixels (Visual Tokenization) for multi-modal models, achieving theoretical maximum density (1 pixel = 1 opcode).
* **v0.1 Scope:** Strictly text-based ASCII for maximum compatibility with current LLM infrastructure.

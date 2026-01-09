# BinLing Encoding v0.1 (BLE)

This document defines the **normative wire-level encoding** for BinLing capsules (BLE: BinLing Encoding).
It specifies how a capsule is serialized into bytes, how it is validated, and what constitutes failure.

BinLing capsules are designed to be **machine-first**, **verifiable**, and **fail-closed**.
If any required check cannot be satisfied, execution **must halt**.

---

## 0. Scope and Goals

### Goals
- Provide a **deterministic**, **unambiguous** binary encoding for capsules.
- Support **policy pinning** (Q0–Q2) and **dictionary pinning** (opcode table).
- Support **strict sizing** using 3D Square Space `SS_N` (N×N×N byte-voxels).
- Enforce **fail-closed** behavior under corruption or tampering.

### Non-goals (v0.1)
- Segmentation across multiple capsules (explicitly **not supported** in v0.1).
- Tag-based activation routing (deferred to v0.2+).
- Multiple competing encoding dialects (single canonical encoding only).

---

## 1. Terminology

- **BLIR**: BinLing Intermediate Representation (semantic structure and execution rules).
- **BLE**: BinLing Encoding (wire format encoding of BLIR).
- **Capsule**: An atomic executable unit placed at a 3D coordinate (x,y,z).
- **Policy Core**: Q0 (CORE_INTENT), Q1 (SCOPE_FILTER), Q2 (HARD_CONSTRAINTS).
- **SS_N**: Square Space size parameter; `SS_N = N` implies a 3D volume of `N×N×N` byte-voxels.

---

## 2. Normative Requirements Keywords

The keywords **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** are to be interpreted as described in RFC 2119.

---

## 3. Byte Order and Primitive Types

- All multi-byte integer fields **MUST** use **little-endian** encoding.
- Integer types used in this spec:
  - `u8`  : 1 byte unsigned
  - `u16` : 2 byte unsigned
  - `u32` : 4 byte unsigned
  - `i16` : 2 byte signed (two’s complement)

---

## 4. Capsule Sizing (3D Square Space)

### 4.1 SS_N Capacity
A capsule declares `SS_N` where capacity in bytes is:

- `CAPACITY_BYTES = N³`

This capacity includes:
- header bytes
- payload bytes
- padding bytes
- integrity bytes (hashes/checksums)

### 4.2 Allowed SS_N Values (v0.1)
`SS_N` is restricted to the fixed enum set:

- `{8, 16, 32, 64, 128}`

Any other SS_N value **MUST** cause decode failure.

### 4.3 Strict Fit Rule (v0.1)
Segmentation is **not supported** in v0.1.
Therefore:

- If `(HEADER_LEN + PAYLOAD_LEN + PAD_LEN) != CAPACITY_BYTES`, decode **MUST FAIL**.
- If `PAYLOAD_LEN` cannot fit in the chosen SS_N cube after accounting for header and integrity fields, decode **MUST FAIL**.

---

## 5. Capsule Layout Overview

A capsule is a fixed-size byte array of length `CAPACITY_BYTES = N³`:

[ Header | Payload | Padding ]

- The header includes integrity fields and declares lengths.
- The payload contains the BLIR capsule content (including Q0–Q5 in encoded form).
- Remaining bytes are filled with canonical PKCS#7 padding.

---

## 6. Capsule Header (Normative)

### 6.1 Fixed Header Fields
The capsule header is **fixed-layout** for v0.1.
The header begins at offset 0.

| Field | Type | Size | Description |
|------|------|------|-------------|
| MAGIC | 4 bytes | 4 | ASCII `"BLE1"` |
| VERSION_MAJOR | u8 | 1 | Must be `0` for v0.1 |
| VERSION_MINOR | u8 | 1 | Must be `1` for v0.1 |
| FLAGS | u16 | 2 | Execution/validation flags |
| SS_N | u8 | 1 | Cube size enum: {8,16,32,64,128} |
| PRIORITY | u8 | 1 | 0–255 (0 is highest priority) |
| HEADER_LEN | u16 | 2 | Total header length in bytes |
| PAYLOAD_LEN | u32 | 4 | Payload length in bytes |
| COORD_X | i16 | 2 | Capsule X coordinate |
| COORD_Y | i16 | 2 | Capsule Y coordinate |
| COORD_Z | i16 | 2 | Capsule Z coordinate |
| CAPSULE_ID | u32 | 4 | Unique ID within program |
| DICT_HASH | 32 bytes | 32 | SHA-256 over canonical dictionary |
| POLICY_CORE_HASH | 32 bytes | 32 | SHA-256 over canonical Q0–Q2 |
| CAPSULE_HASH | 32 bytes | 32 | SHA-256 over full capsule bytes (see §8) |

**Minimum header length (v0.1):** 4+1+1+2+1+1+2+4+2+2+2+4+32+32+32 = **122 bytes**  
`HEADER_LEN` **MUST** be at least 122 and **MUST** match the actual header length.

### 6.2 Flags (v0.1)
`FLAGS` is a bitfield. In v0.1, the following bits are defined:

- Bit 0: `FAIL_CLOSED` (MUST be 1)
- Bit 1: `VERIFY_REQUIRED` (MUST be 1)
- Bit 2: `AUDIT_REQUIRED` (MAY be 1; if 1, Q4 must be present and executed)
- Bits 3–15: Reserved (MUST be 0 in v0.1)

If reserved bits are non-zero, decode **MUST FAIL**.

---

## 7. Payload Encoding (BLIR-in-BLE)

### 7.1 Payload Content
The payload encodes the BLIR capsule content, which includes at minimum:

- Q0 CORE_INTENT (sealed)
- Q1 SCOPE_FILTER (sealed)
- Q2 HARD_CONSTRAINTS (sealed)
- Q3 OPS
- Q4 VERIFY/AUDIT
- Q5 EMIT

v0.1 does not mandate a human-readable syntax for payload.
The payload is considered an opaque byte string to the BLE layer, except for hashing and length enforcement.

### 7.2 Canonicalization Requirement for Hash Inputs
While BLE treats payload opaquely, the *hash values* stored in the header depend on canonical forms:

- `DICT_HASH` MUST be computed from a canonical dictionary representation (§8.2).
- `POLICY_CORE_HASH` MUST be computed from canonicalized Q0–Q2 (§8.3).

---

## 8. Hashing and Integrity (Normative)

### 8.1 Hash Algorithm
All hashes in v0.1 **MUST** use:

- **SHA-256** (32 bytes)

### 8.2 Dictionary Hash (DICT_HASH)
`DICT_HASH` is the SHA-256 hash of the canonical dictionary (opcode table / capability classes / reserved tokens) used to interpret this capsule’s payload semantics.

- The canonical dictionary representation format is defined by the BinLing dictionary spec (external to this file).
- Dictionary canonicalization MUST be deterministic (stable ordering, stable encodings).

If `DICT_HASH` does not match the runtime’s pinned dictionary hash, execution **MUST HALT**.

### 8.3 Policy Core Hash (POLICY_CORE_HASH)
`POLICY_CORE_HASH` is the SHA-256 hash of the canonicalized Policy Core (Q0–Q2).

Canonicalization MUST include:
- stable field ordering
- UTF-8 encoding for textual fields
- normalized whitespace rules (where applicable)
- normalized pattern syntax for SCOPE_FILTER rules

If `POLICY_CORE_HASH` cannot be reproduced exactly by the verifier, execution **MUST HALT**.

### 8.4 Capsule Hash (CAPSULE_HASH)
`CAPSULE_HASH` provides tamper evidence for the capsule bytes.

**Hash coverage rule (v0.1):**
- `CAPSULE_HASH` MUST be the SHA-256 of the entire capsule byte array of length `CAPACITY_BYTES`,
  **with the CAPSULE_HASH field itself treated as 32 zero bytes during hashing**.

Rationale:
- This allows a stable “hash-of-whole-capsule” without recursive dependency.

If `CAPSULE_HASH` does not validate, execution **MUST HALT**.

---

## 9. Padding (Normative)

### 9.1 PKCS#7-Style Padding
After the payload, all remaining bytes up to `CAPACITY_BYTES` MUST be filled with PKCS#7-style padding:

- Let `PAD_LEN = CAPACITY_BYTES - HEADER_LEN - PAYLOAD_LEN`.
- If `PAD_LEN <= 0`, decoding MUST FAIL.
- Every padding byte MUST equal `PAD_LEN` (as a single byte, `u8`).

Example:
- If 10 bytes remain, padding bytes are all `0x0A`.

### 9.2 Padding Validation
If any padding byte is not equal to `PAD_LEN`, decode **MUST FAIL**.

Padding is considered part of integrity coverage (CAPSULE_HASH).

---

## 10. Decode + Verify Procedure (Normative)

A compliant implementation MUST perform validation in the following order:

1. Compute `CAPACITY_BYTES = SS_N³` and confirm the input buffer length equals `CAPACITY_BYTES`.
2. Validate `MAGIC == "BLE1"`.
3. Validate `VERSION_MAJOR == 0` and `VERSION_MINOR == 1`.
4. Validate `FLAGS`:
   - FAIL_CLOSED bit set
   - VERIFY_REQUIRED bit set
   - Reserved bits are 0
5. Validate `SS_N` is one of `{8,16,32,64,128}`.
6. Validate `HEADER_LEN` is ≥ 122 and within bounds.
7. Validate `PAYLOAD_LEN` within bounds such that `PAD_LEN = CAPACITY_BYTES - HEADER_LEN - PAYLOAD_LEN` is > 0.
8. Validate PKCS#7 padding bytes.
9. Validate `DICT_HASH` matches runtime pinned dictionary hash.
10. Recompute and validate `POLICY_CORE_HASH` from canonicalized Q0–Q2.
11. Recompute and validate `CAPSULE_HASH` using §8.4 coverage rule.
12. If any step fails: **HALT / FAIL CLOSED**.

---

## 11. Failure Semantics (Normative)

All decode/verify failures are fatal in v0.1.

A compliant runtime MUST:
- refuse execution
- return/emit a failure state
- NOT attempt “best effort” decoding or fallback execution

---

## 12. Compatibility and Evolution

- BLE versioning uses `(VERSION_MAJOR, VERSION_MINOR)`.
- v0.1 decoders MUST reject unknown major versions.
- v0.1 decoders MUST reject minor versions greater than 1 unless explicitly configured for forward-compatibility.

---

## 13. Notes for Implementers (Non-normative)

- v0.1 prioritizes correctness and auditability over performance.
- Future versions may add:
  - segmentation
  - tag-based activation
  - richer integrity structures (e.g., signatures)
  - more granular SS_N values

These are explicitly out of scope for v0.1.


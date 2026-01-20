# Levin Method v0.1

**Status:** Draft — targeting BinLing v0.1 specification freeze.

## 1. Levin Policy Core v0.1

### 1.1 Definition
Each Capsule contains a Policy Core consisting of:
* **Q0: CORE_INTENT** — the goal this capsule must achieve (per capsule; value varies by capsule origin).
    * *Note:* CORE_INTENT is treated as an opaque value by the execution layer; its internal structure is not interpreted by the Levin VM and is only subject to canonicalization and hashing.
* **Q1: SCOPE_FILTER** — the sandbox/border defining what classes of operations, targets, and data are allowed or denied.
* **Q2: HARD_CONSTRAINTS** — rules governing behavior within the sandbox.

### 1.2 Invariants
* **Policy Core schema is invariant:** Q0, Q1, Q2 must exist and preserve their meanings across all topologies.
* **Policy Core is sealed:** any mutation of Q0–Q2 content constitutes tampering.
* **Policy Core is enforced globally:** no later stratum (Q3+) may override or weaken Q0–Q2 semantics or enforcement.

### 1.3 SCOPE_FILTER (Q1) Semantics
* **Default stance:** DENY_BY_DEFAULT.
* **Rule resolution:** LAST_MATCH_WINS.
* **Rule domains:** operations, targets, data classes.
* **Monotonicity:** later strata may add stricter restrictions but may not relax Q1.

### 1.4 Canonicalization (Required)
Before hashing/verifying, Q0–Q2 must be canonicalized deterministically (stable field ordering, normalized encoding).

### 1.5 Verification Posture
**Fail-closed:** if verification of Policy Core fails, execution halts. No “best effort,” no fallback execution.

---

## 2. Levin Lattice VM v0.1

### 2.1 Program Model
A program consists of a finite set of Capsules placed at 3D coordinates (x,y,z) within a bounded 3D topology space. Each capsule is an atomic executable unit (“node/dot”).

### 2.2 Q-Axis Strata
Capsules are structured by Q-axis strata. A **"tightening"** is defined as any restriction that reduces the allowed operational surface relative to Q1; any expansion of scope constitutes a violation.

* **Q0** CORE_INTENT (sealed)
* **Q1** SCOPE_FILTER (sealed)
* **Q2** HARD_CONSTRAINTS (sealed)
* **Q3** OPS (topology-specific)
* **Q4** VERIFY/AUDIT (topology-specific, fatal on Policy Core violation)
* **Q5** EMIT (topology-specific)

### 2.3 Execution Model
Execution is defined as **node activation** rather than sequential instruction-pointer stepping. A capsule “fires” when activated, executes its internal OPS under Policy Core, and may activate other capsules.

### 2.4 Activation Primitives (v0.1)
* `ACTIVATE(x,y,z)` — activate the capsule at an explicit coordinate.
* `ACTIVATE_NEIGHBOR(dx,dy,dz)` — activate a capsule at relative offset.

### 2.5 Scheduling and Determinism (Queue Model)
Execution proceeds in discrete **Cycles**. The runtime maintains two queues:
1.  **Active Queue**: Capsules scheduled for the current cycle.
2.  **Next Queue**: Capsules scheduled for the next cycle.

**Cycle Rule:**
1.  Execute all capsules in the Active Queue.
2.  Activations emitted during execution are appended to the **Next Queue**.
3.  When Active Queue is empty, swap **Next Queue** → **Active Queue** and increment cycle count.

**Ordering Rule:**
Within a cycle, the Active Queue is de-duplicated by `CAPSULE_ID` and sorted deterministically:
1.  **PRIORITY** ascending (0 is highest).
2.  **COORD** lexicographic tie-break (x, then y, then z).

### 2.6 Execution Ceremony
Each capsule firing follows the mandatory ceremony:
1.  **DECODE** capsule header + payload.
2.  **VERIFY** (Policy Core + integrity checks).
3.  **EXECUTE OPS** (within Policy Core).
4.  **AUDIT** (Q4).
5.  **EMIT** outputs and activation commands.

### 2.7 Termination and VOID
Execution enters the **VOID_BOUNDARY** when both the Active Queue and Next Queue are empty.

* **VOID_BOUNDARY Definition:** A terminal execution boundary indicating no capsules remain scheduled and no continuation is defined within the current program context.
* Entering VOID does not imply data loss or state mutation.
* Only explicitly configured runtime actions (HALT, LOOP, DISPATCH_JOB) may occur at the VOID boundary.

Immediate termination (HALT) also occurs if verification fails (fail-closed) or a capsule emits an explicit HALT command.

### 2.8 Conflict Rules
If capsule execution causes conflicting effects (e.g., two active capsules attempt to write the same protected state), the VM must **FAIL CLOSED**.
* **Protected State:** Any state or resource designated as non-mergeable by the runtime or Policy Core.

---

## 3. Encoding and Header Contract
**Refer to `spec/binling_encoding_v0_1.md` for the normative wire-format specification.**
The Encoding Spec defines the authoritative rules for:
* Capsule Sizing (SS_N)
* Header Layout & Hashing
* Zero Padding Rules
* ASCII Character Restrictions

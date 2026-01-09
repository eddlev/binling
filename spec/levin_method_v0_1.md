> Status: Draft — targeting BinLing v0.1 specification freeze.

Levin Method v0.1
1) Levin Policy Core v0.1
1.1 Definition

Each Capsule contains a Policy Core consisting of:

Q0: CORE_INTENT — the goal this capsule must achieve (per capsule; value varies by capsule origin: human operator, governing AI, cron job, automation pipeline step).

Q1: SCOPE_FILTER — the sandbox/border defining what classes of operations, targets, and data are allowed or denied.

Q2: HARD_CONSTRAINTS — rules governing behavior within the sandbox.

1.2 Invariants

Policy Core schema is invariant: Q0, Q1, Q2 must exist and preserve their meanings across all topologies.

Policy Core is sealed: any mutation of Q0–Q2 content constitutes tampering.

Policy Core is enforced globally: no later stratum (Q3+) may override or weaken Q0–Q2 semantics or enforcement.

1.3 SCOPE_FILTER (Q1) semantics

Default stance: DENY_BY_DEFAULT.

Rule resolution: LAST_MATCH_WINS.

Rule domains: at minimum, rules may match:

operation classes (opcode families / capability classes)

targets (namespaces, paths, hostnames, module IDs)

data classes (e.g., PII, credentials) — if present in the system

Monotonicity: later strata may add stricter restrictions but may not relax Q1.

1.4 Canonicalization (required)

Before hashing/verifying, Q0–Q2 must be canonicalized deterministically:

stable field ordering

normalized encoding (UTF-8 for textual fields)

normalized whitespace rules (where applicable)

normalized pattern syntax for SCOPE_FILTER rules

1.5 Verification posture

Fail-closed: if verification of Policy Core fails, execution halts. No “best effort,” no fallback execution.

2) Levin Lattice VM v0.1
2.1 Program model

A program consists of a finite set of Capsules placed at 3D coordinates (x,y,z) within a bounded 3D topology space.

Each capsule is an atomic executable unit (“node/dot”).

2.2 Q-axis strata per capsule

Capsules are structured by Q-axis strata:

Q0 CORE_INTENT (sealed)

Q1 SCOPE_FILTER (sealed)

Q2 HARD_CONSTRAINTS (sealed)

Q3 OPS (topology-specific)

Q4 VERIFY/AUDIT (topology-specific, but must treat Policy Core violations as fatal)

Q5 EMIT (topology-specific)

Topologies may vary in Q3+ configuration per capsule/topology block, but:

Q0–Q2 semantics and enforcement are invariant.

Q3+ may tighten constraints but may not loosen the border defined by Q1.

2.3 Execution model: activation over instruction pointer

Execution is defined as node activation rather than sequential instruction-pointer stepping.

A capsule “fires” when activated, executes its internal OPS under Policy Core, and may activate other capsules.

2.4 Activation primitives (v0.1)

Capsules may emit activation commands:

ACTIVATE(x,y,z) — activate the capsule at an explicit coordinate.

ACTIVATE_NEIGHBOR(dx,dy,dz) — activate a capsule at relative offset from the current capsule’s coordinate.

(Other routing methods, such as tag-based activation, are deferred to v0.2+.)

2.5 Scheduling and determinism

Active capsules are scheduled deterministically using:

PRIORITY ascending (0 is highest priority)

tie-break by coordinate lexicographic order: (x, then y, then z)

This ordering applies whenever multiple capsules are simultaneously active.

2.6 Execution ceremony (mandatory)

Each capsule firing follows the ceremony:

DECODE capsule header + payload

VERIFY (Policy Core + integrity checks)

EXECUTE OPS (within Policy Core)

AUDIT (Q4, if applicable; Policy Core violations are fatal)

EMIT outputs and activation commands

2.7 Termination

Execution terminates when any of the following occurs:

no capsules remain active

a capsule emits explicit HALT

verification fails (fail-closed)

2.8 Conflict rules (v0.1 strictness)

If capsule execution causes conflicting effects (e.g., two active capsules attempt to write the same protected state/key within the same scheduling epoch), the VM must FAIL CLOSED unless an explicit, deterministic conflict rule exists in Policy Core or a globally pinned runtime rule-set.

Default for v0.1: FAIL on conflict.

3) BinLing Encoding and Header Contract v0.1
3.1 Terminology

BLIR: BinLing Intermediate Representation (capsule structure + semantics).

BLE: BinLing Encoding (wire format encoding of BLIR).

3.2 Capsule sizing (3D Square Space)

Each capsule declares an SS_N size parameter, where:

SS_N = N implies a 3D volume of N×N×N byte-voxels

total capsule capacity in bytes is N³ (including header, payload, and padding)

3.3 Fixed allowed SS_N values (v0.1)

SS_N is restricted to a fixed enum set (v0.1): {8, 16, 32, 64, 128}.

Any other SS_N value causes decode failure.

3.4 Priority field (v0.1)

PRIORITY is an unsigned 8-bit integer: 0–255.

Lower numeric value indicates higher scheduling priority.

3.5 Payload length and padding

The capsule header must include PAYLOAD_LEN (bytes).

Payload bytes occupy the first PAYLOAD_LEN bytes of the payload region.

Remaining capacity is filled with canonical PKCS#7-style padding:

each padding byte value equals the number of padding bytes

Padding is not optional; it must be correct and verifiable.

3.6 Integrity and pinning (required)

Each capsule must include fields sufficient to support:

Protocol version pinning

Opcode/dictionary pinning via DICT_HASH

Policy Core pinning via POLICY_CORE_HASH (hash over canonicalized Q0–Q2)

Integrity checking over the capsule content (header + payload + padding), via a checksum/hash mechanism

3.7 Fail-closed decode/verify

Any mismatch in:

protocol version requirements

dictionary hash

policy core hash

padding validation

integrity check
results in HALT / FAIL CLOSED.

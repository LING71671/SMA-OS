# Formal Verifier Module Guide

**Location**: `control-plane/formal-verifier/`
**Domain**: TLA+ formal verification of state machine correctness
**Language**: TLA+
**Score**: 10/25 (mathematical verification, critical for correctness)

## Overview

Mathematical proof of SMA-OS event sourcing correctness using TLA+. Verifies absolute determinism - state transitions produce identical results given same event sequence.

## Structure

```
formal-verifier/
├── SMA_OS.tla        # Main TLA+ specification
├── SMA_OS.cfg        # Model checker configuration
└── README.md         # Theory documentation
```

## Where to Look

| Task | Location | Notes |
|------|----------|-------|
| State variables | `SMA_OS.tla:4-10` | events, hot_tier, cold_tier, snapshots |
| Initial state | `SMA_OS.tla:16-22` | Init predicate definition |
| Event append | `SMA_OS.tla:24-29` | AppendEvent action |
| Snapshot logic | `SMA_OS.tla:31-37` | GenerateSnapshot action |
| Invariants | `SMA_OS.tla:45-51` | TypeOK, AbsoluteDeterminism |

## Conventions (This Module)

### Variable Naming
```tla
VARIABLES
  events,     \* Append-only event log
  hot_tier,   \* Hot storage (Redis)
  cold_tier,  \* Cold storage (ClickHouse/S3)
  cursor      \* Replay cursor
```

### Action Definition
```tla
AppendEvent ==
  /\ Len(events) < MaxEvents
  /\ events' = Append(events, 1)
  /\ hot_tier' = Append(hot_tier, 1)
  /\ state' = state + 1
  /\ UNCHANGED <<cold_tier, snapshots, cursor>>
```

### Invariant Pattern
```tla
AbsoluteDeterminism ==
  state = Len(events)
```

## Anti-Patterns (This Module)

### Forbidden
```tla
\* NEVER: Use temporal operators in invariants
TemporalOk == <>[](state >= 0)  \* WRONG

\* ALWAYS: Use state predicates only
TypeOk == state >= 0  \* CORRECT
```

### State Updates
```tla
\* WRONG: Partial state updates
cursor' = cursor + 1  \* Missing UNCHANGED

\* CORRECT: Explicit UNCHANGED
/\ cursor' = cursor + 1
/\ UNCHANGED <<events, hot_tier, cold_tier>>
```

## Commands

```bash
# Install TLC model checker
wget https://github.com/tlaplus/tlaplus/releases/download/v1.4.5/...

# Run model checker
tlc SMA_OS.tla -config SMA_OS.cfg

# Generate PDF
tlatex SMA_OS.tla
```

## Dependencies

| Tool | Purpose |
|------|---------|
| TLC | TLA+ model checker |
| TLATeX | LaTeX formatting |
| PlusCal | Algorithm language (optional) |

## Notes

- **Model bounds**: MaxEvents constant limits state space
- **Snapshot interval**: Configurable threshold
- **Invariants**: TypeOK, AbsoluteDeterminism
- **Proof**: Replay yields identical state
- **Specification**: Event sourcing correctness

## Model Checking

```tla
\* SMA_OS.cfg
CONSTANTS
  SnapshotInterval = 10
  MaxEvents = 100

INVARIANTS
  TypeOK
  AbsoluteDeterminism

SPECIFICATION
  Spec
```

## Invariants

| Name | Property | Purpose |
|------|----------|---------|
| TypeOK | Type safety | state ∈ Nat, cursor ∈ Nat |
| AbsoluteDeterminism | state = Len(events) | Deterministic replay |

## State Machine

```
Init → [AppendEvent | GenerateSnapshot]* → (state = MaxEvents)
```

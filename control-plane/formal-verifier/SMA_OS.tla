------------------------------- MODULE SMA_OS -------------------------------
EXTENDS Naturals, Sequences, Integers

VARIABLES
    events,       \* Append-only event log
    hot_tier,     \* Events in the hot storage tier (e.g. Redis)
    cold_tier,    \* Events in the cold storage tier (e.g. ClickHouse/S3)
    snapshots,    \* Generated state snapshots
    cursor,       \* Current replay cursor for the state engine
    state         \* The logical accumulated state of the system

CONSTANTS
    SnapshotInterval, \* Number of events before taking a snapshot
    MaxEvents         \* Maximum events for model checking

Init ==
    /\ events = <<>>
    /\ hot_tier = <<>>
    /\ cold_tier = <<>>
    /\ snapshots = [v \in Nat |-> 0] \* snapshot maps version to state
    /\ cursor = 0
    /\ state = 0

AppendEvent ==
    /\ Len(events) < MaxEvents
    /\ events' = Append(events, 1) \* Simulating an event that adds 1 to state
    /\ hot_tier' = Append(hot_tier, 1)
    /\ state' = state + 1
    /\ UNCHANGED <<cold_tier, snapshots, cursor>>

GenerateSnapshot ==
    /\ Len(events) > 0
    /\ Len(events) % SnapshotInterval = 0
    /\ snapshots' = [snapshots EXCEPT ![Len(events)] = state]
    \* In a real system we would also move hot_tier to cold_tier here
    /\ UNCHANGED <<events, hot_tier, cold_tier, cursor, state>>

Next ==
    \/ AppendEvent
    \/ GenerateSnapshot

Spec == Init /\ [][Next]_<<events, hot_tier, cold_tier, snapshots, cursor, state>>

\* Invariants
TypeOK ==
    /\ state \in Nat
    /\ cursor \in Nat

AbsoluteDeterminism ==
    \* The accumulated state must always match the sum of events, ensuring deterministic replay
    state = Len(events)

=============================================================================

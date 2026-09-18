# Profiling Reference

Argon is profile-first.

## Java Flight Recorder / JDK Mission Control

Primary documentation:
https://docs.oracle.com/en/java/java-components/jdk-mission-control/

Use JFR/JMC for sampled CPU hotspots, allocation pressure, garbage collection, thread scheduling, monitor/lock contention, and longer low-overhead captures.

## spark

Project:
https://github.com/lucko/spark

Use spark for quick Minecraft-oriented CPU/tick investigation and reproducible profiling where appropriate.

## Minimum benchmark record

Record Minecraft version, Argon commit, Java/JVM version, Fabric Loader/API versions, installed optimization mods and versions, reproducible scenario, relevant game settings, duration/warm-up, measurement tool, baseline result, and patched result.

## Questions before changing code

1. Is this path hot in a real workload?
2. Is the cost CPU time, allocation, synchronization, I/O, cache misses, or rendering work?
3. Does the cost remain after the common optimization stack is installed?
4. What vanilla behavior must remain identical?
5. What metric should improve?
6. What regression would make the patch unacceptable?

## Allocation work

Distinguish allocation rate, live set, retained memory, GC frequency, and GC pause time. Reducing retained heap and reducing allocation churn are different problems.

## Frame-time work

Prefer frame-time distributions and low-percentile behavior over average FPS alone. A patch that barely changes average FPS but removes repeatable frame-time spikes can still be valuable.

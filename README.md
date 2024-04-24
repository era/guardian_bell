# Guardian Bell

Guardian Bell is a WIP Alarm Engine written in Rust, 
developed during [live streams](https://twitch.tv/code_elias_code).

It should be compatible with [OpenTelemetry](https://opentelemetry.io/) standards.

# Design

## Ingestion

The metrics are kept for a pre-defined `ttl`. The metrics are kept in-memory
and no disk pagination is supported. The metrics are also written in a WAL to make sure
in case of crashes the software can recover to its last valid state.

The metrics are saved on buckets based on their name and tags. In other words,
each timeseries is kept on their own*.

Any node can receive ingestion requests for any metrics. The metrics are 
sharded and we use [DHT](https://en.wikipedia.org/wiki/Distributed_hash_table)
for lookups.

Nodes can only receive requests if their health checks passed for a majority
of nodes, avoiding in case of brain-split the two halves decide opposite
actions regarding the same alarm configuration. 

* note: the alarms later will subscribe to those buckets


<h1 align="center">gray-cacheopt</h1>
<p align="center">Prompt-cache watcher for gray — port of pi-cache-optimizer.</p>

Watches the assembled outbound prompt each request and warns when the
stable prefix changes in cache-breaking ways (mid-history edit, insertion,
or reorder — anything that isn't a pure append), and accumulates `turn_end`
usage into per-turn cache stats.

## What it does

- `context/build` (observe-only, protocol 2.0): fingerprints each request's
  message list. When the shared leading prefix shrinks vs. the previous
  request, the provider prompt cache is invalidated — a warning is queued.
- `prompt/context`: injects that warning into the next turn so the
  operator (and the model) sees the cause: keep system/context/history
  ordering stable.
- `event/notify` `turn_end`: records `{input, output, cache_read,
  cache_write}` into `~/.gray/cacheopt/state.json` (last 24 turns kept).
- `/cachestats` command: totals, cache hit-rate, and recent per-turn
  sizes. `/cachestats reset` clears.

## Install

```sh
cargo build --release
gray plugin install ~/grayplugins/gray-cacheopt/target/release/gray-cacheopt
```

## CLI

`gray-cacheopt stats` prints the same table outside the agent;
`gray-cacheopt reset` clears state. State honors `$GRAY_HOME`
(default `~/.gray/cacheopt/`).

## Limits

Prefix fingerprints are per-sidecar-process: a sidecar restart re-baselines
silently (no false warning). Warnings are heuristic — a compaction is a
legit prefix rewrite and will warn once.

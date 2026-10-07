# Wasm contract

How the frontend talks to Rust for the calculator. Scanner operations share the same worker file and are covered under `Screen scanner/`.

## The boundary

- The frontend imports the wasm-pack output in `crates/wasm/pkg`. Rust changes need `pnpm run wasm`.
- There are no generated types. `PayloadBuilder.ts` hand-mirrors Rust's `Payload`, and result types are hand-written or untyped. A field rename on either side is a silent break.
- Results are the serialised Rust structs. Rust hash maps arrive as JS `Map`s, not plain objects.
- Extra frontend-only fields on payload objects are tolerated because Rust ignores unknown fields.

## Building the payload

`build_payload` assembles, from the store:

- the upgrade list, from keyed upgrades, including each upgrade's state from the previous optimizer result so the next run starts from it;
- the material table: for each material row, the four `(owned, price)` breakpoints (see `Calculator core/Domain model.md`), with prices converted to per-unit;
- the treatment plan, tier, event flags and free-tap budget;
- the previous free-tap order and the advanced-honing cache, fed back in to avoid recomputation.

## What must stay aligned with Rust

- **Material row order.** `ALL_LABELS` in `Constants.ts` defines row order per tier: seven base materials (`NUM_BASE_MATS`), then juices. It must match the tier's constants JSON on the Rust side. Per-material attributes are not positional: see `MATERIAL_TABLE`.
- **Piece order and counts**, including which index is the weapon and which the vambrace.
- **The advanced-honing strategy tables**, duplicated in `Constants.ts` and in Rust.
- **Treatment plan order**, which indexes arrays in the histogram result.
- **Juice ids**: the frontend also assumes the first two ids are juices and the rest books or scrolls.

## Workers

Each character owns two worker bundles (`WorkerBundle.ts`), held in `useRuntimeStore` and keyed by character name, one per operation:

- **Optimizer**: debounced. Starting a run terminates any run in progress, and the worker is terminated again when the result arrives, so every run uses a fresh worker.
- **Histogram**: throttled. The latest payload wins, runs are never cancelled midway, and the worker is reused.

Messages: the page sends an operation and payload; the worker replies with a result. During optimization, Rust also posts intermediate results with a progress estimate directly, bypassing the TypeScript worker code.

## Flow on an input change

1. An input callback starts both workers (the optimizer only if auto-start is on).
2. The histogram runs immediately against the current state so the graphs update.
3. Each optimizer message, intermediate or final, stores the new state and triggers another histogram run.
4. When the user overrides part of the plan ("compare with simple strategies"), only the histogram reruns, and the instructions display the histogram's state instead of the optimizer's.

Leaving the calculator page or switching character cancels both workers.

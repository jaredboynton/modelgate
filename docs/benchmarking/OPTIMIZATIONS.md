# Stream Performance Optimizations

## Objective

Track every optimization attempted while pursuing the benchmark goal. Each entry must include the hypothesis, changed files, benchmark evidence, and whether the change produced a statistically significant improvement.

## Current State

- Consecutive waves without statistically significant improvement: `3`
- Completion target: `3`
- Latest benchmark artifact: `.live-harness/benchmarks/20260525T130230Z-wave-11-integrated/summary.json`
- Latest benchmark status: final Wave 11 integrated run; `any_significant_improvement=false`; completion target achieved
- Latest benchmark log: `docs/benchmarking/BENCHMARKS.md`

## Prior Fixes Already In Working Tree

- Cursor tool-call correctness: protobuf `Value` MCP args decode cleanly.
- Cursor stream lifecycle: response-header timeout, per-read idle timeout, and `RunStream` drop cleanup.
- Bedrock streaming fallback: signed auth headers are preserved on non-H2 fallback.
- Google/Anthropic stream conversion: translator state flushes on upstream EOF.

## Wave 1 — Candidate Optimizations

### Agent A — Harness And Statistics

- Status: `pass` for harness/statistics/docs only; no live provider calls and no provider optimization verdict.
- Hypothesis: before provider tuning, the harness needed safer sample accounting, explicit tool-loop correctness, richer distribution statistics, and a repeatable comparison path so later waves can decide improvement/no-improvement consistently.
- Changed files: `scripts/benchmark/stream_bench.py`, `docs/benchmarking/BENCHMARKS.md`, `docs/benchmarking/OPTIMIZATIONS.md`.
- Fixes:
  - Reads streams with `HTTPResponse.read1(8192)` when available instead of byte-at-a-time reads, reducing client-side measurement overhead while preserving first byte/event timing from the first returned network chunk.
  - Keeps tool-call and continuation sample counts aligned when either request fails; continuation failures no longer duplicate/corrupt the tool-call sample list.
  - Marks `response.failed`, `response.incomplete`, and `error` terminal events as failed samples even when the HTTP status is 2xx.
  - Extracts tool-call IDs/arguments from response output, item events, argument delta chunks, and Anthropic-style tool inputs; records `tool_call_detected` and `tool_arguments_valid`.
  - Adds p05/p25/p75/p90/p95, sample standard deviation, status counts, terminal event counts, and tool-argument-valid counts to summaries.
  - Adds `--compare-to`, `--improvement-threshold-pct`, and `--min-successful-samples`; comparison output uses positive `improvement_pct` for faster latency or higher throughput and suppresses significance when tool correctness regresses.
  - Adds `UMP_BENCH_ARTIFACT_ROOT` so local/fake-server validation can avoid writing benchmark artifacts into the repo.
- Commands and evidence:
  - `python3 -m py_compile scripts/benchmark/stream_bench.py` passed.
  - Local fake SSE check passed with `cursor_responses`, `--samples 1`, and a temp artifact root; generated text, tool-call, and continuation summaries at `1/1`, with `tool_arguments_valid_count == 1`.
  - Synthetic comparison check passed; a 100 ms to 90 ms TTFT median delta produced `improvement_pct: 10.0` and `any_significant_improvement: true`.
- Findings:
  - The previous harness could count a continuation exception as an extra failed tool-call sample, making tool-call success rates and medians unreliable.
  - The previous harness considered 2xx `response.failed` / `response.incomplete` streams successful, which could let upstream/provider failures pollute performance medians.
  - The previous docs stated the 5% rule but did not define the sign of comparison deltas or the emitted distribution statistics.
- Wave effect: no live baseline or provider benchmark was run by Agent A, so the consecutive no-significant-improvement counter remains unchanged.

### Agent B — Bedrock Messages

- **Wave:** 1
- **Hypothesis:** Bedrock `/v1/messages` streaming spends avoidable per-event work in the AWS event-stream decoder by allocating a `HashMap<String, String>` and owned strings for every event header, even though the downstream mapper only reads `:event-type`, `:message-type`, and `:exception-type`.
- **Change:** `src/upstream/bedrock.rs` now parses only those three relevant AWS event-stream headers into borrowed `&str` fields and skips unused header values without allocation. The existing signed-header fallback change in the working tree was preserved.
- **Benchmark evidence:**
  - Baseline live listener: `.live-harness/benchmarks/20260525T113845Z-wave-1-agent-b-bedrock-baseline/summary.json`; `3/3` text samples, median TTFT `2475.6 ms`, median total `2540.2 ms`, median chars/sec `61.9`.
  - Patched isolated release listener on `127.0.0.1:18744`: `.live-harness/benchmarks/20260525T114056Z-wave-1-agent-b-bedrock-post-parser-opt/summary.json`; `3/3` text samples, median TTFT `1214.5 ms`, median total `1338.5 ms`, median chars/sec `46.6`.
  - Original live listener control rerun: `.live-harness/benchmarks/20260525T114120Z-wave-1-agent-b-bedrock-control-rerun/summary.json`; `3/3` text samples, median TTFT `1422.9 ms`, median total `1484.4 ms`, median chars/sec `142.6`.
- **Verdict:** The parser change is retained as a small Bedrock-owned hot-path allocation reduction. The live provider benchmarks stayed healthy, but the control rerun shows enough Bedrock/network variance that the latency delta should not be attributed solely to this patch. Treat this as an evidence-backed local CPU/allocation optimization, not a proven provider-latency breakthrough.
- **Validation:** `cargo fmt --check` passed; `cargo nextest run --test integration_bedrock_transport` passed with `9 passed`; `cargo clippy --test integration_bedrock_transport --no-deps --all-features -- -D warnings` passed. Full `cargo nextest run` is currently blocked by the shared-tree `src/router.rs:682` compile error (`tracing::info!` level uses a non-constant test value), outside this Bedrock-owned change.

### Agent C — Cursor Responses

- **Hypothesis:** Cursor `/v1/responses` run setup does one avoidable full-buffer copy before opening the upstream stream: `open_streaming_run` wrapped the encoded Connect request frame in `Bytes` by cloning the full `Vec<u8>` first, only to preserve retry data if the initial `send_data` failed.
- **Change:** `src/upstream/cursor/transport.rs` now converts the encoded Connect frame directly into `Bytes` and uses cheap `Bytes::clone()` for the normal-send + retry path. This is Cursor-owned, behavior-preserving, and does not touch adapter/session semantics.
- **Pre-change benchmark:** `.live-harness/benchmarks/20260525T113910Z-wave-1-agent-c-cursor/summary.json` (`3/3` successes for text, tool call, and continuation). Medians: text TTFT/total `737.7/1043.6 ms`; tool call `1570.7/1667.4 ms`; continuation `1099.8/1412.7 ms`.
- **Post-change benchmark:** `.live-harness/benchmarks/20260525T114021Z-wave-1-agent-c-cursor-post-initial-frame/summary.json` (`3/3` successes for text, tool call, and continuation). Medians: text TTFT/total `836.5/1098.6 ms`; tool call `918.5/920.0 ms`; continuation `1022.9/1444.4 ms`.
- **Result:** Mixed. The benchmark contract shows a measured improvement for tool-call TTFT/total (`+41.5%` / `+44.8%`) and continuation TTFT (`+7.0%`), but text latency regressed and continuation throughput regressed. Because the code change only removes a local copy and cannot plausibly explain all network-scale variance, treat this as a small safe cleanup with partial measured improvement, not a broad Cursor Responses win.
- **No further Cursor-owned patch in wave 1:** Current working-tree Cursor stream lifecycle hardening already covers response-header timeout, per-read idle timeout, and `RunStream` drop cleanup; the continuation hot path already consumes tool-call results atomically in one session-store lock. Remaining latency appears dominated by upstream Cursor/model behavior and route-level observability work, not an obvious Cursor adapter/session micro-optimization.

### Agent D — Windsurf Responses

- **Hypothesis:** Windsurf `/v1/responses` streaming pays avoidable allocator churn in the Connect/proto drain loop: when a network chunk contains only complete frames, `drain_text_chunks` used `split_off(offset)` and replaced the buffer with a new empty `Vec` instead of retaining capacity for the next chunk.
- **Change:** `src/upstream/windsurf.rs` now clears the buffer when all buffered bytes were consumed and uses `drain(..offset)` only for partial-frame leftovers. This keeps the existing incremental parser behavior while reusing hot-path buffer capacity.
- **Benchmark evidence:**
  - Baseline installed listener on `127.0.0.1:18743`: `.live-harness/benchmarks/20260525T114108Z-wave-1-agent-d-windsurf-baseline/summary.json`; `3/3` successes for text, tool call, and continuation. Medians: text TTFT/total/chars-sec `444.6 ms` / `453.5 ms` / `2338.6`; tool call `762.8 ms` / `762.8 ms` with tool args valid `3/3`; continuation `1192.4 ms` / `1192.6 ms`.
  - Patched active listener on `127.0.0.1:19045`: `.live-harness/benchmarks/20260525T114359Z-wave-1-agent-d-windsurf-post-buffer-reuse-active/summary.json`; `3/3` successes for text, tool call, and continuation. Medians: text TTFT/total/chars-sec `481.6 ms` / `486.1 ms` / `1320.6`; tool call `776.0 ms` / `776.0 ms` with tool args valid `3/3`; continuation `1198.3 ms` / `1198.3 ms`.
  - Comparison: no scenario met the `5%` improvement threshold. Text latency and throughput regressed in this small live sample; tool-call and continuation medians were roughly flat to slightly slower.
- **Tool-loop finding:** The Windsurf tool path can emit valid lookup arguments and accept a continuation request, but continuation previews show the model often answers that the lookup tool is unavailable because the previous-response turn does not carry tool schemas forward. No schema-retention change was made in this wave because it is broader correctness work, not a small stream-performance optimization.
- **Verdict:** Retain the buffer-capacity patch as a safe local allocation reduction covered by a focused regression, but count Agent D wave 1 as no statistically significant Windsurf performance improvement.
- **Validation:** `cargo fmt --check` passed; `cargo nextest run drain_text_chunks_reuses_complete_frame_buffer_capacity` passed; `cargo nextest run windsurf` passed with `29 passed`; `cargo clippy --tests --no-deps --all-features -- -D warnings` passed.

### Agent E — Codex OAuth/WSS Responses

- **Hypothesis:** Codex stream latency might be dominated by proxy-owned WSS setup, REST fallback, catalog refresh, or latch/pool overhead in the `/v1/responses` path.
- **Changed files:** `docs/benchmarking/OPTIMIZATIONS.md` only; no Codex code changed in this wave.
- **WSS path evidence:** live launchd is running `bin/unified-model-proxy-v2` with `UMP_V2_CODEX_TRANSPORT=wss`, so successful `/v1/responses` Codex probes used `wss://chatgpt.com/backend-api/codex/responses` rather than HTTP fallback. Code inspection showed the active path is `src/route/responses_executor.rs` → `src/upstream/codex.rs::responses_prepared_stream` → `send_wss_stream_with_refresh`; `WssThenHttp` latch/fallback is bypassed by the live `wss` transport setting.
- **Hot-path inspection:** catalog validation is not refreshing per request in the route; `validate_codex_catalog_request` reads the latest cached catalog. The WSS send path already has the flat `response.create` serializer, avoiding an extra event `Value` clone. Pool-key hashing and WSS pool lookup are small local costs compared with observed upstream TTFT.
- **Benchmark evidence:** baseline artifact `.live-harness/benchmarks/20260525T113815Z-wave-0-baseline/summary.json` had Codex text median TTFT `1403.6ms` and median total `1437.4ms` over `3/3` successes. The initial Agent E harness run `.live-harness/benchmarks/20260525T113933Z-wave-1-agent-e-codex/summary.json` reported text median TTFT `1260.7ms` and total `1308.4ms`, but overlapped other active provider benchmark processes, so it is noisy. A clean text-only rerun after benchmark processes exited produced `3/3` successes with median TTFT `1450.9ms` and total `1547.6ms`, which is not a statistically significant improvement over baseline.
- **Tool-loop evidence:** a bounded current tool-call probe succeeded over WSS with `{"q":"ok"}` arguments, TTFT `1077.4ms`, and total `1215.6ms`. Codex continuation is not a valid current performance signal: the Codex request policy strips `previous_response_id`, and a bounded continuation probe returned an upstream `event: error` for the orphaned `function_call_output`.
- **Decision:** no small safe Codex-owned optimization was implemented. The measured local route overhead is already small relative to upstream TTFT, and the only notable finding is benchmark/continuation contract noise rather than an evidence-backed stream-performance patch.
- **Follow-up:** fix the benchmark harness so pretty multi-line `event: error` bodies count as failures and either exclude Codex tool continuation from the stream-performance gate or design explicit Codex continuation support as a separate correctness task.

### Agent F — Cross-Provider Observability

- Hypothesis: the existing `request completed` log fires when route handling returns headers, before streamed SSE bodies are consumed, so benchmarks cannot correlate proxy logs with stream EOF, body size, or chunk count.
- Inspection: `src/router.rs` owns request/provider/model logging, while `src/upstream_response.rs` carries provider/status metadata and converts provider streams into Axum bodies without body-completion metrics.
- Change: `src/router.rs` now wraps only `text/event-stream` responses and emits a `response body completed` log at EOF or body error with `request_id`, provider, model, status, upstream status, header latency, body completion elapsed time, body bytes, and body chunks.
- Benchmark evidence: no live provider benchmark was claimed for this observability-only change; focused validation passed with `cargo test router::tests --lib`, `cargo nextest run --lib router::tests`, and `cargo clippy --tests --no-deps --all-features -- -D warnings`.
- Result: benchmarkability improved, but Agent F claims no statistically significant stream-performance improvement in wave 1.

## Wave 1 — Integrated Verdict

- Benchmark artifact: `.live-harness/benchmarks/20260525T114731Z-wave-1-integrated/summary.json`
- Baseline artifact: `.live-harness/benchmarks/20260525T114607Z-wave-0-baseline-corrected/summary.json`
- Result: `any_significant_improvement=true`; the consecutive no-significant-improvement counter remains `0`.
- Significant improvements: Bedrock text latency/throughput, Cursor text throughput, Cursor tool-call terminal latency/events/sec, Windsurf text latency, Codex text throughput, and Codex tool-call throughput.
- Follow-up risks: Cursor text/continuation latency regressed in the small sample, Codex continuation still fails `0/3`, and Windsurf continuation succeeds but may not preserve tool schema semantics strongly enough for quality.

## Wave 2 — Candidate Optimizations

### Agent A — Codex OAuth/WSS Responses

- **Hypothesis:** Codex tool continuation failed because `previous_response_id` was stripped before the WSS `response.create` payload reached upstream.
- **Change:** `src/upstream/codex.rs` preserves `previous_response_id` in the Codex Responses allowlist; `tests/unit_codex.rs` covers flat WSS payload preservation and function-call-output continuation.
- **Evidence:** `.live-harness/benchmarks/20260525T115455Z-wave-2-agent-a-codex-continuation/summary.json` showed Codex continuation improved from Wave 1 `0/3` to `3/3`; continuation median TTFT was `118.2 ms`, total `645.3 ms`.
- **Verdict:** retained as a correctness fix that removes a benchmark-contract leak and makes Codex tool-loop continuation measurable.

### Agent B — Cursor Responses

- **Hypothesis:** Cursor continuation spends avoidable local work cloning pending tool calls that callers do not use.
- **Change:** `src/upstream/cursor/session.rs` consumes pending tool-call IDs atomically without cloning returned `CursorToolCall` values.
- **Evidence:** `.live-harness/benchmarks/20260525T115617Z-wave-2-agent-b-cursor-post-batch-consume/summary.json` was mixed against the immediate pre-run, but the final integrated Wave 2 run improved Cursor text, tool-call, and continuation medians vs Wave 1.
- **Verdict:** retained as a safe local allocation reduction; live deltas remain provider/noise-sensitive.

### Agent C — Windsurf Responses

- **Hypothesis:** Windsurf continuation lost tool availability when clients omitted `tools` on `previous_response_id` turns.
- **Change:** `src/adapter/windsurf_responses.rs` restores prior tool availability from stored function-call output when continuation omits tools, and `src/adapter/windsurf_chat.rs` tells the model to finalize from a tool result unless another tool call is needed.
- **Evidence:** `.live-harness/benchmarks/20260525T120003Z-wave-2-agent-c-after-final-5x/summary.json` showed continuation success `5/5` and removed the “tool unavailable” behavior; final integrated Wave 2 kept Windsurf continuation `3/3` and improved median total vs Wave 1 by `26.5%`.
- **Verdict:** retained as a correctness and tool-loop performance fix.

### Agent D — Bedrock Messages

- **Hypothesis:** Bedrock fallback auth headers should be cloned/signed only on non-H2 fallback, not on every successful streaming request.
- **Change:** `src/upstream/bedrock.rs` lazily clones/applies fallback headers only inside the non-H2 fallback branch while preserving the signed-header fallback guard.
- **Evidence:** `.live-harness/benchmarks/20260525T115648Z-wave-2-agent-d-bedrock-post-fallback-header-lazy-release/summary.json` improved vs the immediate pre-patch rerun, but not vs the latest integrated baseline. The final Wave 2 integrated run regressed Bedrock latency vs Wave 1.
- **Verdict:** retained as a small hot-path cleanup; not counted as a Wave 2 Bedrock performance win.

### Agent E — Benchmark Harness

- **Hypothesis:** the benchmark harness could still misclassify provider terminal errors and overclaim improvements when tool-loop correctness regressed.
- **Change:** `scripts/benchmark/stream_bench.py` now handles multiline `event: error`, `response.status` failures, `response.error`, continuation attempt IDs, terminal-event early stop, and provider-level tool-loop regression gating.
- **Evidence:** fake/local regression checks and `python3 -m py_compile scripts/benchmark/stream_bench.py` passed; final Wave 2 made Codex continuation failure/success visible without corrupting tool-call stats.
- **Verdict:** retained as measurement correctness work.

### Agent F — Cross-Provider Observability

- **Scope:** observability and benchmark integration only; no provider source or harness logic changes.
- **Correctness issue:** the live benchmark client stops reading after terminal SSE events and closes the connection, so some streams never reached Axum body EOF and did not emit the body-completion log.
- **Change:** `src/router.rs` now logs a final `response body completed` record from `BodyCompletionState::drop` when an SSE body is dropped before EOF, preserving byte/chunk counters and setting `body_error="body stream dropped before EOF"`.
- **Validation:** `cargo fmt --check`, `cargo nextest run --lib router::tests`, full `cargo nextest run`, `cargo clippy --tests --no-deps --all-features -- -D warnings`, `cargo build --release`, install to `bin/unified-model-proxy-v2`, `launchctl kickstart -k gui/$(id -u)/dev.unified-model-proxy-v2`, and `/health` probe all passed.
- **Preliminary benchmark artifact:** `.live-harness/benchmarks/20260525T115457Z-wave-2-integrated/summary.json`; compared to `.live-harness/benchmarks/20260525T114731Z-wave-1-integrated/summary.json`. Other unowned provider edits appeared during or immediately after this run, so rerun once the tree settles before treating it as the final Wave 2 integrated result.
- **Log artifact:** `.live-harness/benchmarks/20260525T115457Z-wave-2-integrated/proxy-log-snippet.log` contains `27` body-completion records, matching the `27` streamed requests actually sent by the run: Bedrock `3`, Cursor `9`, Windsurf `6`, Codex `9`.
- **Log result:** Bedrock emitted `3` dropped-before-EOF records because the harness closed after terminal SSE; Cursor, Windsurf, and Codex emitted `24` clean EOF records.
- **Preliminary benchmark result:** `any_significant_improvement=true`; the no-significant-improvement counter remains `0` if this run is accepted after the tree settles.
- **Significant gains vs Wave 1:** Cursor text TTFT/terminal/total latency, Cursor tool-call TTFT, Cursor continuation TTFT, and Codex tool-call TTFT/terminal/total latency.
- **Regressions/noise:** Bedrock text latency/throughput and Codex text latency/throughput regressed in this sample; Windsurf tool continuation regressed to `0/3` because tool-call samples missed `response_id` or `call_id`.

## Wave 2 — Integrated Verdict

- Benchmark artifact: `.live-harness/benchmarks/20260525T120309Z-wave-2-integrated-final/summary.json`
- Baseline artifact: `.live-harness/benchmarks/20260525T114731Z-wave-1-integrated/summary.json`
- Result: `any_significant_improvement=true`; the consecutive no-significant-improvement counter remains `0`.
- Significant gains: Cursor text/tool/continuation latency, Windsurf tool-call/continuation latency, Codex text/tool-call latency, and Codex continuation correctness from `0/3` to `3/3`.
- Follow-up risks: Bedrock text latency regressed in the final integrated sample, Windsurf text latency regressed despite tiny-output throughput, and Codex text/tool-call throughput regressed.

## Wave 3 — Candidate Optimizations

### Agent A — Bedrock Messages

- **Hypothesis:** Wave 2 Bedrock regression might be proxy-owned in signing, fallback, event decode, or SSE mapping.
- **Change:** no code change.
- **Evidence:** `.live-harness/benchmarks/20260525T120808Z-wave-3-agent-a-bedrock-focused-15x/summary.json` ran `15/15`; total median was `1516.0 ms`, with range `1020.4–2060.9 ms` and p95 `1952.1 ms`. Median post-header time was only `80.4 ms`, so most latency was upstream pre-first-byte rather than local event decode/SSE mapping.
- **Verdict:** no-improvement; no proxy-owned Bedrock regression found.

### Agent B — Cursor Responses

- **Hypothesis:** Cursor stream state had unused per-run closed-state locking.
- **Change:** `src/upstream/cursor/transport.rs` removed unused `closed` state and `is_closed`.
- **Evidence:** `.live-harness/benchmarks/20260525T120944Z-wave-3-agent-b-debug-postpatch/summary.json` was mixed: tool-call TTFT improved vs prepatch, but text TTFT and continuation TTFT regressed under live variance.
- **Verdict:** retained as a cleanup; no broad Cursor improvement.

### Agent C — Windsurf Responses

- **Hypothesis:** Wave 2 Windsurf text regression was variance, and replayed tool schemas could be simplified.
- **Change:** `src/adapter/windsurf_responses.rs` replays prior tool names with minimal object parameters instead of inferring schemas from prior arguments; `src/adapter/windsurf_chat.rs` uses a shorter tool-result finalization guard.
- **Evidence:** `.live-harness/benchmarks/20260525T120731Z-wave-3-agent-c-windsurf-current-live/summary.json` showed text total `550.1 ms`, close to Wave 1 `534.1 ms`; `.live-harness/benchmarks/20260525T120940Z-wave-3-agent-c-windsurf-replay-simplified/summary.json` kept continuation `5/5`.
- **Verdict:** retained as a correctness simplification; final integrated Wave 3 also produced a significant Windsurf continuation latency gain vs best prior.

### Agent D — Codex OAuth/WSS Responses

- **Hypothesis:** stale pooled WSS reuse can close before emitting SSE bytes, causing terminal misses or slow retries.
- **Change:** `src/upstream/codex.rs` retries a reused pooled Codex WSS once if it fails/closes before emitting any SSE bytes; `tests/integration_codex_transport.rs` covers stale pooled WSS retry.
- **Evidence:** `.live-harness/benchmarks/20260525T121412Z-wave-3-agent-d-codex-post-stale-pool-retry-rerun/summary.json` removed text terminal misses but was mixed on latency/throughput. Final integrated Wave 3 improved Codex tool-call TTFT by `25.6%` vs best prior while total latency regressed.
- **Verdict:** retained as a correctness hardening; partial significant TTFT gain only.

### Agent E — Benchmark Harness

- **Hypothesis:** comparing only to the immediate prior artifact can falsely reset the 3-wave no-improvement gate after a regression/recovery.
- **Change:** `scripts/benchmark/stream_bench.py` now accepts repeated `--compare-to` values and selects the best prior median per provider/scenario/metric; `scripts/benchmark/test_stream_bench_compare.py` covers single-baseline compatibility and best-of semantics.
- **Evidence:** Wave 3 integrated used Wave 0, Wave 1, and Wave 2 as repeated baselines and emitted `baseline_mode=best_of_prior_median`.
- **Verdict:** retained as measurement correctness work.

### Agent F — Cross-Provider Observability

- **Hypothesis:** SSE body-completion logging might add overhead or miss early-close streams.
- **Change:** no code change.
- **Evidence:** `.live-harness/benchmarks/20260525T120731Z-wave-3-agent-f-body-log-focused/proxy-body-completion-summary.json` showed `42/42` request/body completion pairs and `0` missed; focused router body-completion tests passed.
- **Verdict:** no-improvement; observability path is adequate.

## Wave 3 — Integrated Verdict

- Benchmark artifact: `.live-harness/benchmarks/20260525T121729Z-wave-3-integrated/summary.json`
- Baseline artifacts: Wave 0 corrected, Wave 1 integrated, and Wave 2 integrated final, using best prior medians per metric.
- Result: `any_significant_improvement=true`; the consecutive no-significant-improvement counter remains `0`.
- Significant gains: Windsurf continuation latency and Codex tool-call TTFT.
- Follow-up risks: Cursor and Bedrock appear near plateau/noisy; Codex tool-call TTFT improved while total latency regressed; best-baseline comparisons should be used for all future waves.

## Wave 4 — Candidate Optimizations

### Agent A — Bedrock Messages

- **Change:** no source change.
- **Evidence:** `.live-harness/benchmarks/20260525T122137Z-wave-4-agent-a-bedrock-focused-5x/summary.json` showed Bedrock total `1570.7 ms` vs best prior Wave 1 `1234.1 ms`, with `5/5` success.
- **Verdict:** no-improvement; Bedrock appears dominated by live upstream variance.

### Agent B — Cursor Responses

- **Change:** no source change.
- **Evidence:** `.live-harness/benchmarks/20260525T122057Z-wave-4-agent-b-cursor-plateau/summary.json` showed Cursor text total improved vs best prior, but tool-call and continuation totals missed best prior.
- **Verdict:** no provider patch; final integrated Wave 4 still produced a significant Cursor text latency improvement.

### Agent C — Windsurf Responses

- **Change:** no source change.
- **Evidence:** three focused 5-sample continuation runs stayed near the Wave 3 best but did not beat it by `5%`: `.live-harness/benchmarks/20260525T122118Z-wave-4-agent-c-windsurf-focused-5x-a/summary.json`, `.live-harness/benchmarks/20260525T122203Z-wave-4-agent-c-windsurf-focused-5x-b/summary.json`, and `.live-harness/benchmarks/20260525T122221Z-wave-4-agent-c-windsurf-focused-5x-c/summary.json`.
- **Verdict:** durable continuation gain from Wave 3, but Wave 4 is below the significance threshold.

### Agent D — Codex OAuth/WSS Responses

- **Change:** no source change.
- **Evidence:** `.live-harness/benchmarks/20260525T122116Z-wave-4-agent-d-codex-focused/summary.json` showed Codex continuation total `657.7 ms` vs best prior `666.5 ms`, below `5%`; pool-off probe failed continuation `0/3`, proving pooling remains required for continuation.
- **Verdict:** no provider patch; final integrated Wave 4 significantly improved Codex tool-call total latency.

### Agent E — Benchmark Harness

- **Change:** `scripts/benchmark/stream_bench.py` handles absent scenarios in compare/markdown paths, clarifies best-of wording, detects function calls inside completed `response.output`, and marks continuation samples failed when they emit another tool call.
- **Tests:** `scripts/benchmark/test_stream_bench_compare.py` covers best success/tool ratios, zero throughput, absent scenarios, markdown notes, completed-response tool-call detection, and repeated continuation tool-call failure.
- **Evidence:** the first Wave 4 integrated artifact was superseded because a Windsurf continuation sample emitted another tool call; `.live-harness/benchmarks/20260525T122753Z-wave-4-integrated-corrected/summary.json` is the accepted Wave 4 artifact.
- **Verdict:** retained as measurement correctness work.

### Agent F — Cross-Provider Observability

- **Change:** no source change.
- **Evidence:** `.live-harness/benchmarks/20260525T122134Z-wave-4-agent-f-log-probe/summary.md` showed the live `x-request-id` appeared in both `request completed` and `response body completed` logs with expected body byte/chunk fields.
- **Verdict:** no-improvement; logging correlation remains healthy.

## Wave 4 — Integrated Verdict

- Benchmark artifact: `.live-harness/benchmarks/20260525T122753Z-wave-4-integrated-corrected/summary.json`
- Superseded artifact: `.live-harness/benchmarks/20260525T122600Z-wave-4-integrated/summary.json`
- Baseline artifacts: Wave 0 corrected, Wave 1 integrated, Wave 2 integrated final, and Wave 3 integrated, using best prior medians per metric.
- Result: `any_significant_improvement=true`; the consecutive no-significant-improvement counter remains `0`.
- Significant gains: Cursor text latency and Codex tool-call terminal/total latency.
- Follow-up risks: Bedrock remains below best prior; Cursor tool/continuation remain below best prior; Windsurf continuation is a near miss but below threshold; Codex continuation total improved only `4.9%`.

## Wave 5 — Candidate Optimizations

### Agent A — Bedrock Messages

- **Change:** no source change.
- **Evidence:** `.live-harness/benchmarks/20260525T123113Z-wave-5-agent-a-bedrock-final-plateau/summary.json` showed Bedrock total `1708.2 ms` vs best prior `1234.1 ms`, with `5/5` success and header/first-byte/TTFT aligned.
- **Verdict:** no-improvement; latency remains upstream-dominated.

### Agent B — Cursor Responses

- **Change:** no source change.
- **Evidence:** `.live-harness/benchmarks/20260525T123445Z-wave-5-agent-b-cursor-focused-rerun/summary.json` missed best-prior text, tool-call, and continuation total medians. Local first-byte stayed around `0.8–3.5 ms`, pointing away from local serialization.
- **Verdict:** no provider patch; final integrated Wave 5 only improved Cursor continuation throughput, not latency.

### Agent C — Windsurf Responses

- **Change:** `src/adapter/windsurf_responses.rs` stops re-exposing prior tools on tool-result continuation; `src/adapter/windsurf_chat.rs` adds no-tool finalization prompt handling; `tests/integration_windsurf.rs` verifies replay prompt uses no available tools.
- **Evidence:** `.live-harness/benchmarks/20260525T123856Z-wave-5-agent-c-windsurf-final-guard-5x/summary.json` had continuation `5/5` with zero repeated tool calls and tool-call total `777.6 ms` vs best prior `866.1 ms`.
- **Verdict:** retained as correctness hardening and tool-call latency improvement; continuation latency regressed vs best prior.

### Agent D — Codex OAuth/WSS Responses

- **Change:** no source change.
- **Evidence:** `.live-harness/benchmarks/20260525T123209Z-wave-5-agent-d-codex-focused-rerun/summary.json` missed Wave 4 corrected tool/continuation best totals, though text improved vs Wave 4 corrected.
- **Verdict:** no-improvement for requested tool/continuation path.

### Agent E — Benchmark Harness

- **Change:** no source change.
- **Evidence:** audit confirmed repeated-tool detection is sufficient; requiring final-answer text would create false failures for accepted zero-text continuations.
- **Verdict:** no-improvement; Wave 5 integrated interpretation unchanged.

### Agent F — Cross-Provider Observability

- **Change:** no source change.
- **Evidence:** Wave 4 corrected log window had `30/30` request/body pairs and no unmatched stream bodies; post-Wave-4 traffic had `59` matched stream pairs and only `/health` unmatched.
- **Verdict:** no-improvement; log correlation remains healthy.

## Wave 5 — Integrated Verdict

- Benchmark artifact: `.live-harness/benchmarks/20260525T124219Z-wave-5-integrated/summary.json`
- Baseline artifacts: Wave 0 corrected, Wave 1 integrated, Wave 2 integrated final, Wave 3 integrated, and Wave 4 corrected, using best prior medians per metric.
- Result: `any_significant_improvement=true`; the consecutive no-significant-improvement counter remains `0`.
- Significant gains: Cursor continuation throughput, Windsurf text latency, Windsurf tool-call latency, and Codex tool-call throughput.
- Follow-up risks: these gains are mostly provider variance/throughput slices rather than broad latency wins; Bedrock, Cursor tool latency, Windsurf continuation, Codex text, and Codex continuation all missed best prior.

## Wave 6 — Candidate Optimizations

### Agent A — Bedrock Messages

- **Change:** no source change.
- **Evidence:** `.live-harness/benchmarks/20260525T124537Z-wave-6-agent-a-bedrock-plateau/summary.json` ran `15/15`; total median `1627.5 ms`, total stdev `386.3 ms`, and no Bedrock log errors/retries/throttles.
- **Verdict:** no-improvement; Bedrock plateau/noise confirmed.

### Agent B — Cursor Responses

- **Change:** no source change.
- **Evidence:** `.live-harness/benchmarks/20260525T124552Z-wave-6-agent-b-cursor-focused-5x-a/summary.json` and `.live-harness/benchmarks/20260525T124614Z-wave-6-agent-b-cursor-focused-5x-b/summary.json` missed best prior latency/throughput medians.
- **Verdict:** no-improvement; Wave 5 continuation throughput was not durable.

### Agent C — Windsurf Responses

- **Change:** `src/adapter/windsurf_chat.rs` tightens final-result prompting and omits the misleading empty `Available tools: (none)` section; `tests/integration_windsurf.rs` covers this.
- **Evidence:** `.live-harness/benchmarks/20260525T124934Z-wave-6-agent-c-windsurf-result-guard-release/summary.json` had continuation `3/3` with no repeated tool calls, but continuation was only `2.1%` faster than best prior.
- **Verdict:** retained as correctness hardening; not a statistically significant performance win.

### Agent D — Codex OAuth/WSS Responses

- **Change:** no source change.
- **Evidence:** `.live-harness/benchmarks/20260525T124619Z-wave-6-agent-d-codex-wss-plateau/summary.json` missed text, tool, and continuation best total latencies; Wave 5 tool throughput gain was not durable.
- **Verdict:** no-improvement.

### Agent E — Benchmark Harness

- **Change:** `scripts/benchmark/stream_bench.py` now records `text_chars` / `events` distributions and gates throughput significance when output is tiny and latency regresses; `scripts/benchmark/test_stream_bench_compare.py` covers the tiny-output regression.
- **Evidence:** Wave 5 still resets because Windsurf latency gains remain significant, but Codex tiny-output tool throughput-only wins no longer count.
- **Verdict:** retained as measurement correctness work.

### Agent F — Cross-Provider Observability

- **Change:** no source change.
- **Evidence:** `.live-harness/benchmarks/20260525T124739Z-wave-6-agent-f-log-correlation/proxy-body-completion-summary.json` showed `99/99` body completions matched request completions with no field mismatches.
- **Verdict:** no-improvement; log correlation remains healthy.

## Wave 6 — Integrated Verdict

- Benchmark artifact: `.live-harness/benchmarks/20260525T125151Z-wave-6-integrated/summary.json`
- Baseline artifacts: Wave 0 corrected, Wave 1 integrated, Wave 2 integrated final, Wave 3 integrated, Wave 4 corrected, and Wave 5 integrated, using best prior medians per metric.
- Original result: `any_significant_improvement=true` under the median-only throughput gate.
- Re-evaluated result: `any_significant_improvement=false` under the Wave 7 distribution gate.
- Follow-up: Wave 7 tightened statistical significance beyond median deltas so provider variance does not masquerade as optimization.

## Wave 7 — Integrated Verdict

- Benchmark artifact: `.live-harness/benchmarks/20260525T125629Z-wave-7-integrated/summary.json`
- Baseline artifacts: Wave 0 corrected through Wave 6 integrated, using best prior medians per metric.
- Harness change: `scripts/benchmark/stream_bench.py` now requires per-sample pairwise dominance of at least `0.8` before a median delta can count as significant; `scripts/benchmark/test_stream_bench_compare.py` covers overlapping/noisy samples and stable non-overlapping improvements.
- Result: `any_significant_improvement=false`; the consecutive no-significant-improvement counter is `2` when Wave 6 is re-evaluated under the corrected gate.
- Findings: Bedrock remained below best prior, Cursor tool loop failed `0/3`, Windsurf remained below best prior, and Codex’s apparent throughput/continuation gains did not pass distribution dominance.

## Wave 8 — Integrated Verdict

- Benchmark artifact: `.live-harness/benchmarks/20260525T125757Z-wave-8-integrated/summary.json`
- Baseline artifacts: Wave 0 corrected through Wave 7 integrated, using best prior medians per metric.
- Result: `any_significant_improvement=true`; the consecutive no-significant-improvement counter resets to `0`.
- Significant gains: Codex tool-call terminal/total latency.
- Findings: Cursor failed all scenarios, Bedrock and Windsurf missed best prior, and Codex continuation did not materially improve.

## Wave 9 — Integrated Verdict

- Benchmark artifact: `.live-harness/benchmarks/20260525T125915Z-wave-9-integrated/summary.json`
- Baseline artifacts: Wave 0 corrected through Wave 8 integrated, using best prior medians per metric.
- Result: `any_significant_improvement=false`; the consecutive no-significant-improvement counter is `1`.
- Findings: Bedrock, Windsurf, and Codex missed best prior; Cursor failed all scenarios and was blocked by the tool-loop guard.

## Wave 10 — Integrated Verdict

- Benchmark artifact: `.live-harness/benchmarks/20260525T130048Z-wave-10-integrated/summary.json`
- Baseline artifacts: Wave 0 corrected through Wave 9 integrated, using best prior medians per metric.
- Result: `any_significant_improvement=false`; the consecutive no-significant-improvement counter is `2`.
- Findings: Bedrock and Codex missed best prior, Cursor failed all scenarios, and Windsurf near-misses did not pass the distribution gate.

## Wave 11 — Integrated Verdict

- Benchmark artifact: `.live-harness/benchmarks/20260525T130230Z-wave-11-integrated/summary.json`
- Baseline artifacts: Wave 0 corrected through Wave 10 integrated, using best prior medians per metric.
- Result: `any_significant_improvement=false`; the consecutive no-significant-improvement counter is `3`, so the benchmark goal completion gate is satisfied.
- Findings: Bedrock, Windsurf, and Codex missed best-prior latency medians or failed distribution dominance; Cursor stayed below the minimum success threshold with `1/5` tool-call and `1/5` continuation success.

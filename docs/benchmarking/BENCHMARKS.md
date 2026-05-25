# Stream Performance Benchmarks

## Objective

Benchmark and optimize streaming performance for:

1. Bedrock through the Messages endpoint.
2. Cursor through Responses.
3. Windsurf through Responses.
4. OpenAI Codex OAuth/WebSocket through Responses.

The goal is complete only after 3 consecutive optimization waves produce no statistically significant improvement in any measured behavior.

## Measurement Contract

- **TTFT:** time from request send start to first meaningful streamed model event.
- **Total latency:** time from request send start to terminal response event or response EOF.
- **Throughput:** response characters per second and event count per second after TTFT.
- **Tool loop:** time to tool-call emission, tool arguments correctness, continuation request TTFT, continuation total latency, and terminal status.
- **Significance rule:** compare same-provider wave samples against the prior best baseline using median and mean. A wave counts as improved when median TTFT or median total latency improves by at least 5%, or median throughput improves by at least 5%, with at least 3 successful samples on both sides and pairwise distribution dominance of at least `0.8`. Tool-loop correctness regressions invalidate performance gains.
- **Harness statistics:** every run records success/failure counts, HTTP status counts, terminal event counts, median/mean/min/max, p05/p25/p75/p90/p95, and sample standard deviation when at least 2 successful samples exist.
- **Comparison sign:** positive `improvement_pct` means lower latency or higher throughput versus the compared baseline; negative values are regressions.

## Benchmark Environment

- Proxy URL: `http://127.0.0.1:18743`
- Artifacts root: `.live-harness/benchmarks/`
- Harness: `scripts/benchmark/stream_bench.py`
- Overrides: `UMP_BENCH_BASE_URL` changes the proxy URL, and `UMP_BENCH_ARTIFACT_ROOT` changes the artifact root for local/fake-server checks.
- Baseline comparison: pass `--compare-to path/to/summary.json` to embed same-provider/scenario median deltas and significant-improvement flags in the artifact.
- Current consecutive no-significant-improvement waves: `3`

## Provider Matrix

| ID | Provider | Route | Model | Scenario |
|---|---|---|---|---|
| `bedrock_messages` | Bedrock | `/v1/messages` | `claude-sonnet-4-6` | Streaming text |
| `cursor_responses` | Cursor | `/v1/responses` | `composer-2-fast` | Streaming text + forced tool loop |
| `windsurf_responses` | Windsurf | `/v1/responses` | `swe-1.6` | Streaming text + forced tool loop |
| `codex_responses_ws` | Codex OAuth/WSS | `/v1/responses` | `gpt-5.5` | Streaming text + forced tool loop |

## Wave Log

### Wave 0 — Baseline

#### `wave-0-baseline-corrected`

- Timestamp: `2026-05-25T11:46:07Z`
- Samples per scenario: `3`
- Artifact: `.live-harness/benchmarks/20260525T114607Z-wave-0-baseline-corrected/summary.json`
- Verdict: corrected baseline after terminal-event handling and markdown rendering fixes; this is the comparison baseline for Wave 1.

| Provider | Scenario | Success | Median TTFT ms | P95 TTFT ms | Median total ms | Median chars/sec | Tool args valid | Notes |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| bedrock_messages | text | 3/3 | 1306.7 | 1470.5 | 1340.3 | 51.6 |  |  |
| cursor_responses | text | 3/3 | 870.5 | 1188.9 | 1124.6 | 31.4 |  |  |
| cursor_responses | tool_call | 3/3 | 962.2 | 1435.0 | 1486.7 | 10000.0 | 3/3 |  |
| cursor_responses | tool_continuation | 3/3 | 1050.3 | 1087.6 | 1515.6 | 1229.0 |  |  |
| windsurf_responses | text | 3/3 | 649.2 | 896.6 | 649.4 | 1305.9 |  |  |
| windsurf_responses | tool_call | 3/3 | 866.0 | 1327.8 | 866.1 | 0.0 | 3/3 |  |
| windsurf_responses | tool_continuation | 3/3 | 1731.5 | 1905.1 | 1731.6 | 0.0 |  |  |
| codex_responses_ws | text | 3/3 | 1299.8 | 1563.4 | 1343.9 | 191.2 |  |  |
| codex_responses_ws | tool_call | 3/3 | 1052.0 | 1149.6 | 1197.9 | 68.5 | 3/3 |  |
| codex_responses_ws | tool_continuation | 0/3 |  |  |  |  |  |  |

### Wave 1 — Integrated

#### `wave-1-integrated`

- Timestamp: `2026-05-25T11:47:31Z`
- Samples per scenario: `3`
- Artifact: `.live-harness/benchmarks/20260525T114731Z-wave-1-integrated/summary.json`
- Baseline artifact: `.live-harness/benchmarks/20260525T114607Z-wave-0-baseline-corrected/summary.json`
- Verdict: statistically significant improvement found, so the consecutive no-improvement counter remains `0`.

| Provider | Scenario | Success | Median TTFT ms | P95 TTFT ms | Median total ms | Median chars/sec | Tool args valid | Notes |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| bedrock_messages | text | 3/3 | 1169.8 | 1903.7 | 1234.1 | 57.2 |  |  |
| cursor_responses | text | 3/3 | 1296.2 | 1368.1 | 1484.4 | 42.5 |  |  |
| cursor_responses | tool_call | 3/3 | 1214.5 | 1303.1 | 1214.5 | 10000.0 | 3/3 |  |
| cursor_responses | tool_continuation | 3/3 | 1327.0 | 1581.0 | 2012.5 | 1068.6 |  |  |
| windsurf_responses | text | 3/3 | 522.1 | 899.8 | 534.1 | 987.1 |  |  |
| windsurf_responses | tool_call | 3/3 | 949.6 | 1535.0 | 949.7 | 0.0 | 3/3 |  |
| windsurf_responses | tool_continuation | 3/3 | 1810.6 | 2163.9 | 1810.7 | 0.0 |  |  |
| codex_responses_ws | text | 3/3 | 1327.2 | 1514.0 | 1354.9 | 235.1 |  |  |
| codex_responses_ws | tool_call | 3/3 | 1286.1 | 1365.9 | 1424.4 | 72.3 | 3/3 |  |
| codex_responses_ws | tool_continuation | 0/3 |  |  |  |  |  |  |

#### Wave 1 Baseline Comparison

- Any significant improvement: `true`
- Significant gains: Bedrock text latency/throughput, Cursor text throughput, Cursor tool-call terminal latency/events/sec, Windsurf text latency, Codex text throughput, and Codex tool-call throughput.
- Regressions/noise to investigate next: Cursor text latency, Cursor continuation latency/throughput, Windsurf tool-loop latency, Codex tool-call latency, and Codex continuation `0/3`.

| Provider | Scenario | Metric | Baseline median | Current median | Improvement % | Significant |
|---|---|---|---:|---:|---:|---:|
| bedrock_messages | text | ttft_ms | 1306.7 | 1169.8 | 10.5 | true |
| bedrock_messages | text | total_ms | 1340.3 | 1234.1 | 7.9 | true |
| bedrock_messages | text | chars_per_sec | 51.6 | 57.2 | 10.8 | true |
| cursor_responses | text | total_ms | 1124.6 | 1484.4 | -32.0 | false |
| cursor_responses | text | chars_per_sec | 31.4 | 42.5 | 35.6 | true |
| cursor_responses | tool_call | total_ms | 1486.7 | 1214.5 | 18.3 | true |
| cursor_responses | tool_continuation | total_ms | 1515.6 | 2012.5 | -32.8 | false |
| windsurf_responses | text | ttft_ms | 649.2 | 522.1 | 19.6 | true |
| windsurf_responses | text | total_ms | 649.4 | 534.1 | 17.8 | true |
| windsurf_responses | tool_call | total_ms | 866.1 | 949.7 | -9.7 | false |
| windsurf_responses | tool_continuation | total_ms | 1731.6 | 1810.7 | -4.6 | false |
| codex_responses_ws | text | total_ms | 1343.9 | 1354.9 | -0.8 | false |
| codex_responses_ws | text | chars_per_sec | 191.2 | 235.1 | 22.9 | true |
| codex_responses_ws | tool_call | total_ms | 1197.9 | 1424.4 | -18.9 | false |
| codex_responses_ws | tool_call | chars_per_sec | 68.5 | 72.3 | 5.5 | true |
| codex_responses_ws | tool_continuation | success | 0/3 | 0/3 |  | false |

### Wave 2 — Integrated Final

#### `wave-2-integrated-final`

- Timestamp: `2026-05-25T12:03:09Z`
- Samples per scenario: `3`
- Artifact: `.live-harness/benchmarks/20260525T120309Z-wave-2-integrated-final/summary.json`
- Baseline artifact: `.live-harness/benchmarks/20260525T114731Z-wave-1-integrated/summary.json`
- Verdict: statistically significant improvement found, so the consecutive no-improvement counter remains `0`.

| Provider | Scenario | Success | Median TTFT ms | P95 TTFT ms | Median total ms | Median chars/sec | Tool args valid | Notes |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| bedrock_messages | text | 3/3 | 1734.8 | 1954.1 | 1814.2 | 50.3 |  |  |
| cursor_responses | text | 3/3 | 947.3 | 1650.1 | 1201.0 | 48.7 |  |  |
| cursor_responses | tool_call | 3/3 | 889.4 | 1021.4 | 889.4 | 10000.0 | 3/3 |  |
| cursor_responses | tool_continuation | 3/3 | 1152.6 | 1260.8 | 1319.6 | 1315.7 |  |  |
| windsurf_responses | text | 3/3 | 778.8 | 818.8 | 823.6 | 8000.0 |  |  |
| windsurf_responses | tool_call | 3/3 | 883.4 | 1991.1 | 883.5 | 0.0 | 3/3 |  |
| windsurf_responses | tool_continuation | 3/3 | 1330.1 | 1428.7 | 1330.1 | 0.0 |  |  |
| codex_responses_ws | text | 3/3 | 1214.9 | 4195.4 | 1249.2 | 124.3 |  |  |
| codex_responses_ws | tool_call | 3/3 | 954.6 | 1047.0 | 1101.3 | 68.1 | 3/3 |  |
| codex_responses_ws | tool_continuation | 3/3 | 115.5 | 194.8 | 666.5 | 0.0 |  |  |

#### Wave 2 Baseline Comparison

- Any significant improvement: `true`
- Correctness gains: Codex continuation improved from `0/3` to `3/3`; Windsurf continuation remained `3/3` and now retains enough tool context to finalize with the tool result.
- Significant latency gains: Cursor text/tool/continuation, Windsurf tool-call/continuation, Codex text/tool-call.
- Regressions/noise to investigate next: Bedrock text latency regressed, Windsurf text latency regressed despite high tiny-output chars/sec, and Codex text/tool-call throughput regressed.

| Provider | Scenario | Metric | Baseline median | Current median | Improvement % | Significant |
|---|---|---|---:|---:|---:|---:|
| bedrock_messages | text | total_ms | 1234.1 | 1814.2 | -47.0 | false |
| cursor_responses | text | total_ms | 1484.4 | 1201.0 | 19.1 | true |
| cursor_responses | tool_call | total_ms | 1214.5 | 889.4 | 26.8 | true |
| cursor_responses | tool_continuation | total_ms | 2012.5 | 1319.6 | 34.4 | true |
| windsurf_responses | text | total_ms | 534.1 | 823.6 | -54.2 | false |
| windsurf_responses | tool_call | total_ms | 949.7 | 883.5 | 7.0 | true |
| windsurf_responses | tool_continuation | total_ms | 1810.7 | 1330.1 | 26.5 | true |
| codex_responses_ws | text | total_ms | 1354.9 | 1249.2 | 7.8 | true |
| codex_responses_ws | tool_call | total_ms | 1424.4 | 1101.3 | 22.7 | true |
| codex_responses_ws | tool_continuation | success | 0/3 | 3/3 |  | false |

### Wave 3 — Integrated

#### `wave-3-integrated`

- Timestamp: `2026-05-25T12:17:29Z`
- Samples per scenario: `3`
- Artifact: `.live-harness/benchmarks/20260525T121729Z-wave-3-integrated/summary.json`
- Baseline artifacts: Wave 0 corrected, Wave 1 integrated, and Wave 2 integrated final.
- Baseline selection: best prior median per provider/scenario/metric.
- Verdict: statistically significant improvement found, so the consecutive no-improvement counter remains `0`.

| Provider | Scenario | Success | Median TTFT ms | P95 TTFT ms | Median total ms | Median chars/sec | Tool args valid | Notes |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| bedrock_messages | text | 3/3 | 1223.5 | 1823.8 | 1279.9 | 35.0 |  |  |
| cursor_responses | text | 3/3 | 1045.9 | 1247.2 | 1281.7 | 31.7 |  |  |
| cursor_responses | tool_call | 3/3 | 874.6 | 1270.9 | 942.6 | 10000.0 | 3/3 |  |
| cursor_responses | tool_continuation | 3/3 | 1148.0 | 2562.6 | 1951.2 | 895.1 |  |  |
| windsurf_responses | text | 3/3 | 543.8 | 852.4 | 546.6 | 1062.8 |  |  |
| windsurf_responses | tool_call | 3/3 | 1005.4 | 1153.2 | 1005.5 | 0.0 | 3/3 |  |
| windsurf_responses | tool_continuation | 3/3 | 755.4 | 768.8 | 755.5 | 0.0 |  |  |
| codex_responses_ws | text | 3/3 | 1674.8 | 3098.6 | 1715.1 | 78.4 |  |  |
| codex_responses_ws | tool_call | 3/3 | 710.6 | 1181.1 | 1417.9 | 54.2 | 3/3 |  |
| codex_responses_ws | tool_continuation | 3/3 | 180.8 | 200.3 | 680.0 | 0.0 |  |  |

#### Wave 3 Best-Baseline Comparison

- Any significant improvement: `true`
- Significant gains vs best prior: Windsurf continuation latency and Codex tool-call TTFT.
- No-improvement/variance: Bedrock was within high live variance; Cursor did not beat best prior; Windsurf text/tool-call did not beat best prior; Codex text/continuation did not beat best prior.

| Provider | Scenario | Metric | Best prior median | Current median | Improvement % | Significant |
|---|---|---:|---:|---:|---:|---:|
| bedrock_messages | text | total_ms | 1234.1 | 1279.9 | -3.7 | false |
| cursor_responses | text | total_ms | 1124.6 | 1281.7 | -14.0 | false |
| cursor_responses | tool_call | total_ms | 889.4 | 942.6 | -6.0 | false |
| cursor_responses | tool_continuation | total_ms | 1319.6 | 1951.2 | -47.9 | false |
| windsurf_responses | text | total_ms | 534.1 | 546.6 | -2.3 | false |
| windsurf_responses | tool_call | total_ms | 866.1 | 1005.5 | -16.1 | false |
| windsurf_responses | tool_continuation | total_ms | 1330.1 | 755.5 | 43.2 | true |
| codex_responses_ws | text | total_ms | 1249.2 | 1715.1 | -37.3 | false |
| codex_responses_ws | tool_call | ttft_ms | 954.6 | 710.6 | 25.6 | true |
| codex_responses_ws | tool_call | total_ms | 1101.3 | 1417.9 | -28.7 | false |
| codex_responses_ws | tool_continuation | total_ms | 666.5 | 680.0 | -2.0 | false |

### Wave 4 — Integrated Corrected

#### `wave-4-integrated-corrected`

- Timestamp: `2026-05-25T12:27:53Z`
- Samples per scenario: `3`
- Artifact: `.live-harness/benchmarks/20260525T122753Z-wave-4-integrated-corrected/summary.json`
- Baseline artifacts: Wave 0 corrected, Wave 1 integrated, Wave 2 integrated final, and Wave 3 integrated.
- Baseline selection: best prior median per provider/scenario/metric.
- Supersedes: `.live-harness/benchmarks/20260525T122600Z-wave-4-integrated/summary.json`, because the harness now fails continuation samples that emit another tool call.
- Verdict: statistically significant improvement found, so the consecutive no-improvement counter remains `0`.

| Provider | Scenario | Success | Median TTFT ms | P95 TTFT ms | Median total ms | Median chars/sec | Tool args valid | Notes |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| bedrock_messages | text | 3/3 | 1716.7 | 2015.2 | 1885.9 | 56.5 |  |  |
| cursor_responses | text | 3/3 | 755.8 | 777.5 | 926.4 | 46.9 |  |  |
| cursor_responses | tool_call | 3/3 | 936.9 | 1098.3 | 1276.0 | 294.9 | 3/3 |  |
| cursor_responses | tool_continuation | 3/3 | 1292.9 | 1528.5 | 1890.5 | 1016.5 |  |  |
| windsurf_responses | text | 3/3 | 567.0 | 599.5 | 573.0 | 1333.2 |  |  |
| windsurf_responses | tool_call | 3/3 | 888.4 | 1066.9 | 888.4 | 0.0 | 3/3 |  |
| windsurf_responses | tool_continuation | 3/3 | 732.6 | 839.6 | 732.7 | 0.0 |  |  |
| codex_responses_ws | text | 3/3 | 1846.4 | 2657.5 | 1846.5 | 246.4 |  |  |
| codex_responses_ws | tool_call | 3/3 | 712.4 | 884.6 | 860.8 | 69.1 | 3/3 |  |
| codex_responses_ws | tool_continuation | 3/3 | 126.1 | 134.2 | 634.1 | 0.0 |  |  |

#### Wave 4 Best-Baseline Comparison

- Any significant improvement: `true`
- Significant gains vs best prior: Cursor text latency and Codex tool-call terminal/total latency.
- Near misses: Windsurf continuation total improved from best prior `755.5 ms` to `732.7 ms`, but only by `3.0%`.
- No-improvement/variance: Bedrock stayed below best prior, Cursor tool/continuation did not beat best prior, Windsurf text/tool-call did not beat best prior, and Codex continuation was below the `5%` threshold.

| Provider | Scenario | Metric | Best prior median | Current median | Improvement % | Significant |
|---|---|---:|---:|---:|---:|---:|
| bedrock_messages | text | total_ms | 1234.1 | 1885.9 | -52.8 | false |
| cursor_responses | text | total_ms | 1124.6 | 926.4 | 17.6 | true |
| cursor_responses | tool_call | total_ms | 889.4 | 1276.0 | -43.5 | false |
| cursor_responses | tool_continuation | total_ms | 1319.6 | 1890.5 | -43.3 | false |
| windsurf_responses | text | total_ms | 534.1 | 573.0 | -7.3 | false |
| windsurf_responses | tool_call | total_ms | 866.1 | 888.4 | -2.6 | false |
| windsurf_responses | tool_continuation | total_ms | 755.5 | 732.7 | 3.0 | false |
| codex_responses_ws | text | total_ms | 1249.2 | 1846.5 | -47.8 | false |
| codex_responses_ws | tool_call | total_ms | 1101.3 | 860.8 | 21.8 | true |
| codex_responses_ws | tool_continuation | total_ms | 666.5 | 634.1 | 4.9 | false |

### Wave 5 — Integrated

#### `wave-5-integrated`

- Timestamp: `2026-05-25T12:42:19Z`
- Samples per scenario: `3`
- Artifact: `.live-harness/benchmarks/20260525T124219Z-wave-5-integrated/summary.json`
- Baseline artifacts: Wave 0 corrected, Wave 1 integrated, Wave 2 integrated final, Wave 3 integrated, and Wave 4 corrected.
- Baseline selection: best prior median per provider/scenario/metric.
- Verdict: statistically significant improvement found, so the consecutive no-improvement counter remains `0`.

| Provider | Scenario | Success | Median TTFT ms | P95 TTFT ms | Median total ms | Median chars/sec | Tool args valid | Notes |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| bedrock_messages | text | 3/3 | 1573.9 | 1850.0 | 1661.4 | 51.5 |  |  |
| cursor_responses | text | 3/3 | 801.1 | 1127.5 | 964.9 | 48.4 |  |  |
| cursor_responses | tool_call | 3/3 | 947.6 | 981.8 | 1052.4 | 1078.2 | 3/3 |  |
| cursor_responses | tool_continuation | 3/3 | 1286.6 | 1428.6 | 1609.5 | 1861.5 |  |  |
| windsurf_responses | text | 3/3 | 473.4 | 637.7 | 473.9 | 963.8 |  |  |
| windsurf_responses | tool_call | 3/3 | 717.6 | 738.2 | 717.6 | 0.0 | 3/3 |  |
| windsurf_responses | tool_continuation | 3/3 | 852.3 | 1015.9 | 852.4 | 0.0 |  |  |
| codex_responses_ws | text | 3/3 | 1427.5 | 1605.2 | 1468.2 | 196.2 |  |  |
| codex_responses_ws | tool_call | 3/3 | 857.0 | 1028.7 | 958.9 | 91.7 | 3/3 |  |
| codex_responses_ws | tool_continuation | 3/3 | 111.6 | 122.9 | 720.8 | 0.0 |  |  |

#### Wave 5 Best-Baseline Comparison

- Any significant improvement: `true`
- Significant gains vs best prior: Cursor continuation throughput, Windsurf text latency, Windsurf tool-call latency, and Codex tool-call throughput.
- No-improvement/variance: Bedrock remained below best prior, Cursor text/tool latency missed best prior, Windsurf continuation regressed vs Wave 4 corrected, and Codex text/continuation missed best prior.

| Provider | Scenario | Metric | Best prior median | Current median | Improvement % | Significant |
|---|---|---:|---:|---:|---:|---:|
| bedrock_messages | text | total_ms | 1234.1 | 1661.4 | -34.6 | false |
| cursor_responses | text | total_ms | 926.4 | 964.9 | -4.2 | false |
| cursor_responses | tool_call | total_ms | 889.4 | 1052.4 | -18.3 | false |
| cursor_responses | tool_continuation | total_ms | 1319.6 | 1609.5 | -22.0 | false |
| cursor_responses | tool_continuation | chars_per_sec | 1315.7 | 1861.5 | 41.5 | true |
| windsurf_responses | text | total_ms | 534.1 | 473.9 | 11.3 | true |
| windsurf_responses | tool_call | total_ms | 866.1 | 717.6 | 17.1 | true |
| windsurf_responses | tool_continuation | total_ms | 732.7 | 852.4 | -16.3 | false |
| codex_responses_ws | text | total_ms | 1249.2 | 1468.2 | -17.5 | false |
| codex_responses_ws | tool_call | total_ms | 860.8 | 958.9 | -11.4 | false |
| codex_responses_ws | tool_call | chars_per_sec | 72.3 | 91.7 | 26.8 | true |
| codex_responses_ws | tool_continuation | total_ms | 634.1 | 720.8 | -13.7 | false |

### Wave 6 — Integrated

#### `wave-6-integrated`

- Timestamp: `2026-05-25T12:51:51Z`
- Samples per scenario: `3`
- Artifact: `.live-harness/benchmarks/20260525T125151Z-wave-6-integrated/summary.json`
- Baseline artifacts: Wave 0 corrected, Wave 1 integrated, Wave 2 integrated final, Wave 3 integrated, Wave 4 corrected, and Wave 5 integrated.
- Baseline selection: best prior median per provider/scenario/metric with throughput output gates.
- Verdict: statistically significant improvement found, so the consecutive no-improvement counter remains `0`.

| Provider | Scenario | Success | Median TTFT ms | P95 TTFT ms | Median total ms | Median chars/sec | Tool args valid | Notes |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| bedrock_messages | text | 3/3 | 1326.8 | 2389.8 | 1384.9 | 41.5 |  |  |
| cursor_responses | text | 3/3 | 3838.0 | 3884.0 | 4005.1 | 47.9 |  |  |
| cursor_responses | tool_call | 2/3 | 12291.2 | 20746.3 | 12291.3 | 10000.0 | 2/2 |  |
| cursor_responses | tool_continuation | 2/3 | 3663.5 | 5084.3 | 5115.1 | 513.9 |  |  |
| windsurf_responses | text | 3/3 | 782.6 | 4304.3 | 809.4 | 986.1 |  |  |
| windsurf_responses | tool_call | 3/3 | 1056.1 | 1482.6 | 1056.1 | 0.0 | 3/3 |  |
| windsurf_responses | tool_continuation | 3/3 | 821.1 | 4069.9 | 821.1 | 0.0 |  |  |
| codex_responses_ws | text | 3/3 | 1210.0 | 1778.2 | 1234.4 | 328.5 |  |  |
| codex_responses_ws | tool_call | 3/3 | 822.6 | 857.6 | 890.8 | 84.5 | 3/3 |  |
| codex_responses_ws | tool_continuation | 3/3 | 116.8 | 146.5 | 661.9 | 0.0 |  |  |

#### Wave 6 Best-Baseline Comparison

- Any significant improvement: `true`
- Significant gains vs best prior: Codex text throughput only.
- Important regression: Cursor tool-loop success dropped to `2/3`, so Cursor metrics were blocked by the tool-loop guard.
- No-improvement/variance: Bedrock, Cursor latency, Windsurf, Codex tool-call latency, and Codex continuation missed best prior.

| Provider | Scenario | Metric | Best prior median | Current median | Improvement % | Significant |
|---|---|---:|---:|---:|---:|---:|
| bedrock_messages | text | total_ms | 1234.1 | 1384.9 | -12.2 | false |
| cursor_responses | text | total_ms | 926.4 | 4005.1 | -332.3 | false |
| cursor_responses | tool_call | success | 3/3 | 2/3 |  | false |
| cursor_responses | tool_continuation | success | 3/3 | 2/3 |  | false |
| windsurf_responses | text | total_ms | 473.9 | 809.4 | -70.8 | false |
| windsurf_responses | tool_call | total_ms | 717.6 | 1056.1 | -47.2 | false |
| windsurf_responses | tool_continuation | total_ms | 732.7 | 821.1 | -12.1 | false |
| codex_responses_ws | text | total_ms | 1249.2 | 1234.4 | 1.2 | false |
| codex_responses_ws | text | chars_per_sec | 246.4 | 328.5 | 33.3 | true |
| codex_responses_ws | tool_call | total_ms | 860.8 | 890.8 | -3.5 | false |
| codex_responses_ws | tool_continuation | total_ms | 634.1 | 661.9 | -4.4 | false |

### Wave 7 — Integrated

#### `wave-7-integrated`

- Timestamp: `2026-05-25T12:56:29Z`
- Samples per scenario: `3`
- Artifact: `.live-harness/benchmarks/20260525T125629Z-wave-7-integrated/summary.json`
- Baseline artifacts: Wave 0 corrected through Wave 6 integrated.
- Baseline selection: best prior median per provider/scenario/metric with output and distribution gates.
- Verdict: no statistically significant improvement.
- Streak note: under the Wave 7 distribution gate, Wave 6 re-evaluates to no significant improvement, so the current no-improvement streak is `2`.

| Provider | Scenario | Success | Median TTFT ms | P95 TTFT ms | Median total ms | Median chars/sec | Tool args valid | Notes |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| bedrock_messages | text | 3/3 | 1385.6 | 1413.1 | 1491.8 | 52.9 |  |  |
| cursor_responses | text | 1/3 | 8137.3 | 8137.3 | 8475.0 | 35.5 |  |  |
| cursor_responses | tool_call | 0/3 |  |  |  |  |  | Error |
| cursor_responses | tool_continuation | 0/3 |  |  |  |  |  | missing response_id or call_id |
| windsurf_responses | text | 3/3 | 784.0 | 1520.9 | 804.8 | 1086.4 |  |  |
| windsurf_responses | tool_call | 3/3 | 1011.3 | 4320.6 | 1011.3 | 0.0 | 3/3 |  |
| windsurf_responses | tool_continuation | 3/3 | 771.2 | 955.6 | 771.3 | 0.0 |  |  |
| codex_responses_ws | text | 3/3 | 1344.0 | 1828.3 | 1374.7 | 8000.0 |  |  |
| codex_responses_ws | tool_call | 3/3 | 831.6 | 1454.1 | 967.7 | 74.9 | 3/3 |  |
| codex_responses_ws | tool_continuation | 3/3 | 114.6 | 120.7 | 561.6 | 0.0 |  |  |

#### Wave 7 Best-Baseline Comparison

- Any significant improvement: `false`
- Distribution gate effect: Codex text throughput and Codex continuation total deltas did not count because pairwise dominance was below threshold.
- Correctness issue: Cursor had provider/tool-loop failures and was blocked by the tool-loop guard.

| Provider | Scenario | Metric | Best prior median | Current median | Improvement % | Significant |
|---|---|---:|---:|---:|---:|---:|
| bedrock_messages | text | total_ms | 1234.1 | 1491.8 | -20.9 | false |
| cursor_responses | tool_call | success | 3/3 | 0/3 |  | false |
| cursor_responses | tool_continuation | success | 3/3 | 0/3 |  | false |
| windsurf_responses | text | total_ms | 473.9 | 804.8 | -69.8 | false |
| windsurf_responses | tool_call | total_ms | 717.6 | 1011.3 | -40.9 | false |
| windsurf_responses | tool_continuation | total_ms | 732.7 | 771.3 | -5.3 | false |
| codex_responses_ws | text | chars_per_sec | 328.5 | 8000.0 | 2335.1 | false |
| codex_responses_ws | tool_call | total_ms | 860.8 | 967.7 | -12.4 | false |
| codex_responses_ws | tool_continuation | total_ms | 634.1 | 561.6 | 11.4 | false |

### Wave 8 — Integrated

#### `wave-8-integrated`

- Timestamp: `2026-05-25T12:57:57Z`
- Samples per scenario: `3`
- Artifact: `.live-harness/benchmarks/20260525T125757Z-wave-8-integrated/summary.json`
- Baseline artifacts: Wave 0 corrected through Wave 7 integrated.
- Baseline selection: best prior median per provider/scenario/metric with output and distribution gates.
- Verdict: statistically significant improvement found, so the consecutive no-improvement counter resets to `0`.

| Provider | Scenario | Success | Median TTFT ms | P95 TTFT ms | Median total ms | Median chars/sec | Tool args valid | Notes |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| bedrock_messages | text | 3/3 | 1441.1 | 1661.4 | 1625.4 | 54.2 |  |  |
| cursor_responses | text | 0/3 |  |  |  |  |  | Error |
| cursor_responses | tool_call | 0/3 |  |  |  |  |  | Error |
| cursor_responses | tool_continuation | 0/3 |  |  |  |  |  | missing response_id or call_id |
| windsurf_responses | text | 3/3 | 582.6 | 796.1 | 587.6 | 8000.0 |  |  |
| windsurf_responses | tool_call | 3/3 | 1050.9 | 1164.1 | 1050.9 | 0.0 | 3/3 |  |
| windsurf_responses | tool_continuation | 3/3 | 808.0 | 908.7 | 808.0 | 0.0 |  |  |
| codex_responses_ws | text | 3/3 | 1234.3 | 1348.7 | 1296.0 | 201.3 |  |  |
| codex_responses_ws | tool_call | 3/3 | 687.5 | 708.6 | 793.5 | 76.2 | 3/3 |  |
| codex_responses_ws | tool_continuation | 3/3 | 170.5 | 1413.9 | 560.8 | 0.0 |  |  |

#### Wave 8 Best-Baseline Comparison

- Any significant improvement: `true`
- Significant gains vs best prior: Codex tool-call terminal/total latency.
- Regressions/noise: Cursor failed all scenarios; Bedrock and Windsurf stayed below best prior; Codex continuation total improved only `0.1%`.

| Provider | Scenario | Metric | Best prior median | Current median | Improvement % | Significant |
|---|---|---:|---:|---:|---:|---:|
| bedrock_messages | text | total_ms | 1234.1 | 1625.4 | -31.7 | false |
| cursor_responses | tool_call | success | 3/3 | 0/3 |  | false |
| windsurf_responses | text | total_ms | 473.9 | 587.6 | -24.0 | false |
| windsurf_responses | tool_call | total_ms | 717.6 | 1050.9 | -46.4 | false |
| windsurf_responses | tool_continuation | total_ms | 732.7 | 808.0 | -10.3 | false |
| codex_responses_ws | tool_call | total_ms | 860.8 | 793.5 | 7.8 | true |
| codex_responses_ws | tool_continuation | total_ms | 561.6 | 560.8 | 0.1 | false |

### Wave 9 — Integrated

#### `wave-9-integrated`

- Timestamp: `2026-05-25T12:59:15Z`
- Samples per scenario: `5`
- Artifact: `.live-harness/benchmarks/20260525T125915Z-wave-9-integrated/summary.json`
- Baseline artifacts: Wave 0 corrected through Wave 8 integrated.
- Baseline selection: best prior median per provider/scenario/metric with output and distribution gates.
- Verdict: no statistically significant improvement; current no-improvement streak is `1`.

| Provider | Scenario | Success | Median TTFT ms | P95 TTFT ms | Median total ms | Median chars/sec | Tool args valid | Notes |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| bedrock_messages | text | 5/5 | 1488.7 | 2037.3 | 1537.5 | 23.9 |  |  |
| cursor_responses | text | 0/5 |  |  |  |  |  | Error |
| cursor_responses | tool_call | 0/5 |  |  |  |  |  | Error |
| cursor_responses | tool_continuation | 0/5 |  |  |  |  |  | missing response_id or call_id |
| windsurf_responses | text | 5/5 | 966.5 | 1287.5 | 973.2 | 1186.9 |  |  |
| windsurf_responses | tool_call | 5/5 | 1011.1 | 1468.6 | 1011.1 | 0.0 | 5/5 |  |
| windsurf_responses | tool_continuation | 5/5 | 854.8 | 1200.2 | 854.9 | 0.0 |  |  |
| codex_responses_ws | text | 5/5 | 1921.1 | 2460.6 | 1922.9 | 200.7 |  |  |
| codex_responses_ws | tool_call | 5/5 | 1126.2 | 1775.4 | 1283.6 | 63.9 | 5/5 |  |
| codex_responses_ws | tool_continuation | 5/5 | 126.9 | 160.8 | 729.3 | 0.0 |  |  |

#### Wave 9 Best-Baseline Comparison

- Any significant improvement: `false`
- Notable: Cursor remained `0/5`, and every provider/scenario missed best prior or failed distribution dominance.

| Provider | Scenario | Metric | Best prior median | Current median | Improvement % | Significant |
|---|---|---:|---:|---:|---:|---:|
| bedrock_messages | text | total_ms | 1234.1 | 1537.5 | -24.6 | false |
| cursor_responses | tool_call | success | 3/3 | 0/5 |  | false |
| windsurf_responses | text | total_ms | 473.9 | 973.2 | -105.4 | false |
| windsurf_responses | tool_call | total_ms | 717.6 | 1011.1 | -40.9 | false |
| windsurf_responses | tool_continuation | total_ms | 732.7 | 854.9 | -16.7 | false |
| codex_responses_ws | text | total_ms | 1234.4 | 1922.9 | -55.8 | false |
| codex_responses_ws | tool_call | total_ms | 793.5 | 1283.6 | -61.8 | false |
| codex_responses_ws | tool_continuation | total_ms | 560.8 | 729.3 | -30.0 | false |

### Wave 10 — Integrated

#### `wave-10-integrated`

- Timestamp: `2026-05-25T13:00:48Z`
- Samples per scenario: `5`
- Artifact: `.live-harness/benchmarks/20260525T130048Z-wave-10-integrated/summary.json`
- Baseline artifacts: Wave 0 corrected through Wave 9 integrated.
- Baseline selection: best prior median per provider/scenario/metric with output and distribution gates.
- Verdict: no statistically significant improvement; current no-improvement streak is `2`.

| Provider | Scenario | Success | Median TTFT ms | P95 TTFT ms | Median total ms | Median chars/sec | Tool args valid | Notes |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| bedrock_messages | text | 5/5 | 1626.3 | 2239.7 | 1787.4 | 32.3 |  |  |
| cursor_responses | text | 0/5 |  |  |  |  |  | Error |
| cursor_responses | tool_call | 0/5 |  |  |  |  |  | Error |
| cursor_responses | tool_continuation | 0/5 |  |  |  |  |  | missing response_id or call_id |
| windsurf_responses | text | 5/5 | 482.6 | 621.3 | 483.9 | 1360.6 |  |  |
| windsurf_responses | tool_call | 5/5 | 736.1 | 755.2 | 736.1 | 0.0 | 5/5 |  |
| windsurf_responses | tool_continuation | 5/5 | 876.6 | 946.3 | 876.6 | 0.0 |  |  |
| codex_responses_ws | text | 5/5 | 1917.5 | 5375.2 | 1965.2 | 155.7 |  |  |
| codex_responses_ws | tool_call | 5/5 | 975.9 | 1747.0 | 1457.5 | 68.3 | 5/5 |  |
| codex_responses_ws | tool_continuation | 5/5 | 152.1 | 265.9 | 819.9 | 0.0 |  |  |

#### Wave 10 Best-Baseline Comparison

- Any significant improvement: `false`
- Notable: Cursor remained `0/5`; Windsurf text/tool medians were near but did not beat best prior under distribution dominance.

| Provider | Scenario | Metric | Best prior median | Current median | Improvement % | Significant |
|---|---|---:|---:|---:|---:|---:|
| bedrock_messages | text | total_ms | 1234.1 | 1787.4 | -44.8 | false |
| cursor_responses | tool_call | success | 3/3 | 0/5 |  | false |
| windsurf_responses | text | total_ms | 473.9 | 483.9 | -2.1 | false |
| windsurf_responses | tool_call | total_ms | 717.6 | 736.1 | -2.6 | false |
| windsurf_responses | tool_continuation | total_ms | 732.7 | 876.6 | -19.6 | false |
| codex_responses_ws | text | total_ms | 1234.4 | 1965.2 | -59.2 | false |
| codex_responses_ws | tool_call | total_ms | 793.5 | 1457.5 | -83.7 | false |
| codex_responses_ws | tool_continuation | total_ms | 560.8 | 819.9 | -46.2 | false |

### Wave 11 — Integrated

#### `wave-11-integrated`

- Timestamp: `2026-05-25T13:02:30Z`
- Samples per scenario: `5`
- Artifact: `.live-harness/benchmarks/20260525T130230Z-wave-11-integrated/summary.json`
- Baseline artifacts: Wave 0 corrected through Wave 10 integrated.
- Baseline selection: best prior median per provider/scenario/metric with output and distribution gates.
- Verdict: no statistically significant improvement; current no-improvement streak is `3`, satisfying the completion gate.

| Provider | Scenario | Success | Median TTFT ms | P95 TTFT ms | Median total ms | Median chars/sec | Tool args valid | Notes |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| bedrock_messages | text | 5/5 | 1708.9 | 1804.8 | 1744.6 | 47.4 |  |  |
| cursor_responses | text | 0/5 |  |  |  |  |  | response.failed |
| cursor_responses | tool_call | 1/5 | 954.1 | 954.1 | 954.3 | 10000.0 | 1/1 | response.failed on 4/5 |
| cursor_responses | tool_continuation | 1/5 | 2033.6 | 2033.6 | 2327.4 | 2287.7 |  | missing successful setup on 4/5 |
| windsurf_responses | text | 5/5 | 518.0 | 2269.8 | 525.3 | 1183.8 |  |  |
| windsurf_responses | tool_call | 5/5 | 932.8 | 1014.4 | 932.8 | 0.0 | 5/5 |  |
| windsurf_responses | tool_continuation | 5/5 | 831.3 | 3819.1 | 831.4 | 0.0 |  |  |
| codex_responses_ws | text | 5/5 | 1569.0 | 1860.7 | 1614.1 | 177.4 |  |  |
| codex_responses_ws | tool_call | 5/5 | 1118.3 | 3027.3 | 1329.4 | 41.6 | 5/5 |  |
| codex_responses_ws | tool_continuation | 5/5 | 256.9 | 314.0 | 977.5 | 0.0 |  |  |

#### Wave 11 Best-Baseline Comparison

- Any significant improvement: `false`
- Notable: Wave 9, Wave 10, and Wave 11 are three consecutive integrated runs with no statistically significant improvement under the corrected distribution gate.
- Cursor remained below the minimum success threshold; all successful Bedrock, Windsurf, and Codex scenarios missed best prior medians or failed distribution dominance.

| Provider | Scenario | Metric | Best prior median | Current median | Improvement % | Significant |
|---|---|---:|---:|---:|---:|---:|
| bedrock_messages | text | total_ms | 1234.1 | 1744.6 | -41.4 | false |
| cursor_responses | text | success | tool_call 3/3; continuation 3/3 | tool_call 1/5; continuation 1/5 |  | false |
| cursor_responses | tool_call | total_ms | 889.4 | 954.3 | -7.3 | false |
| cursor_responses | tool_continuation | total_ms | 1319.6 | 2327.4 | -76.4 | false |
| windsurf_responses | text | total_ms | 473.9 | 525.3 | -10.8 | false |
| windsurf_responses | tool_call | total_ms | 717.6 | 932.8 | -30.0 | false |
| windsurf_responses | tool_continuation | total_ms | 732.7 | 831.4 | -13.5 | false |
| codex_responses_ws | text | total_ms | 1234.4 | 1614.1 | -30.8 | false |
| codex_responses_ws | tool_call | total_ms | 793.5 | 1329.4 | -67.5 | false |
| codex_responses_ws | tool_continuation | total_ms | 560.8 | 977.5 | -74.3 | false |

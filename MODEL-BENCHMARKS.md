# Bedrock Model Availability and TPS Benchmarks

Generated: 2026-05-26

## Scope

These notes summarize the Bedrock/Mantle model checks and throughput benchmarks run from this workspace using the AWS credentials in `.env`.

The target workload was simple transcript chunk classification:

- Input: customer call transcript chunks.
- Output: compact structured JSON / JSONL with theme labels, confidence, and evidence.
- Useful metrics: end-to-end latency for small outputs, strict JSON reliability, and sustained output tokens/sec for long batched outputs.

## Bedrock Qwen Availability

Live `us-east-1` Bedrock catalog showed these Qwen models:

- `qwen.qwen3-32b-v1:0`
- `qwen.qwen3-coder-30b-a3b-v1:0`
- `qwen.qwen3-coder-next`
- `qwen.qwen3-next-80b-a3b`
- `qwen.qwen3-vl-235b-a22b`

`Qwen3-0.6B` was not available in the Bedrock catalog.

## Z.AI Pricing

US Bedrock pricing per 1M tokens:

| Model | Context | Max output | Input / 1M | Output / 1M |
|---|---:|---:|---:|---:|
| `zai.glm-4.7-flash` | 203K | 4K | $0.07 | $0.40 |
| `zai.glm-4.7` | 203K | 4K | $0.60 | $2.20 |
| `zai.glm-5` | 200K | 128K | $1.00 | $3.20 |

## Cerebras Pricing and Benchmark

Cerebras `gpt-oss-120b` was tested using `CEREBRAS_API_KEY` from `.env`. The account is billed through AWS.

Pricing:

| Model | Context | Max output | Input / 1M | Output / 1M |
|---|---:|---:|---:|---:|
| `gpt-oss-120b` via Cerebras | 131K | >10K verified | $0.35 | $0.75 |

Important runtime note: use `reasoning_effort = "low"` for classification and JSONL workloads. Without this setting, `gpt-oss-120b` can spend the full completion budget on reasoning tokens and emit no visible content.

10k-output JSONL classification benchmark:

| Model | Prompt Tokens | Completion Tokens | Reasoning Tokens | Stream TPS | End-to-End TPS | TTFB | Total Time | Valid Records | Finish |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---|
| `gpt-oss-120b` via Cerebras | 6,701 | 10,099 | 69 | 3,342.5 | 2,985.6 | 0.361s | 3.383s | 420/420 | `stop` |

## Mantle Grok Test

`xai.grok-4.20` was exposed by Mantle `/v1/models` in `us-east-1`, but inference did not produce usable output:

- `/v1/responses` returned `HTTP 200` but emitted no SSE events.
- Non-streaming `/v1/responses` timed out after 90 seconds on a one-sentence prompt.
- `/v1/chat/completions` returned `401` with `Berm is not enabled for this account`.
- The same streaming harness worked against `openai.gpt-oss-120b`, so the failure appears specific to the Grok/Mantle path.

No TPS result was available for `xai.grok-4.20`.

## Mantle Model Pricing

Pricing below is US Standard on-demand pricing per 1M tokens where it was published on the AWS Bedrock pricing page or verified from provider docs/search during this investigation. Mantle model availability was checked live through `bedrock-mantle.{region}.api.aws/v1/models`.

Limit columns are provider-surface limits, not necessarily the base model's native architecture limit. `TBD` means the Mantle listing/pricing work found the model, but this pass did not find a clean official limit row.

| Mantle model | Context | Max output | Input / 1M | Output / 1M | Notes |
|---|---:|---:|---:|---:|---|
| `anthropic.claude-haiku-4-5` | 200K | 64K | $1.00 | $5.00 | Available in `us-east-1`, `us-west-2` |
| `anthropic.claude-opus-4-7` | 1M | 128K | $5.00 | $25.00 | Available in `us-east-1` |
| `deepseek.v3.1` | 128K | 8K | N/A | N/A | Mantle lists it, but no US Standard row was found in the AWS pricing page fetch |
| `deepseek.v3.2` | 164K | 8K | $0.62 | $1.85 | US regions |
| `google.gemma-3-4b-it` | 128K | 8K | $0.04 | $0.08 | US regions |
| `google.gemma-3-12b-it` | 128K | 8K | $0.09 | $0.29 | US regions |
| `google.gemma-3-27b-it` | TBD | TBD | $0.23 | $0.38 | US regions |
| `minimax.minimax-m2` | 1M | 8K | $0.30 | $1.20 | US regions |
| `minimax.minimax-m2.1` | TBD | TBD | $0.30 | $1.20 | US regions |
| `minimax.minimax-m2.5` | 196K | 8K | $0.30 | $1.20 | US regions |
| `mistral.devstral-2-123b` | TBD | TBD | $0.40 | $2.00 | US regions |
| `mistral.magistral-small-2509` | TBD | TBD | $0.50 | $1.50 | AWS pricing label: Magistral Small 1.2 |
| `mistral.ministral-3-3b-instruct` | 128K | 8K | $0.10 | $0.10 | US regions |
| `mistral.ministral-3-8b-instruct` | TBD | TBD | $0.15 | $0.15 | US regions |
| `mistral.ministral-3-14b-instruct` | 128K | 8K | $0.20 | $0.20 | US regions |
| `mistral.mistral-large-3-675b-instruct` | TBD | TBD | $0.50 | $1.50 | US regions |
| `mistral.voxtral-mini-3b-2507` | TBD | TBD | $0.04 | $0.04 | AWS pricing label: Voxtral Mini 1.0 |
| `mistral.voxtral-small-24b-2507` | TBD | TBD | $0.10 | $0.30 | AWS pricing label: Voxtral Small 1.0 |
| `moonshotai.kimi-k2-thinking` | TBD | TBD | $0.60 | $2.50 | US regions |
| `moonshotai.kimi-k2.5` | 256K | 16K | $0.60 | $3.00 | US regions |
| `nvidia.nemotron-nano-9b-v2` | TBD | TBD | $0.06 | $0.23 | AWS pricing label: NVIDIA Nemotron Nano 2 |
| `nvidia.nemotron-nano-12b-v2` | TBD | TBD | $0.20 | $0.60 | AWS pricing label: NVIDIA Nemotron Nano 2 VL |
| `nvidia.nemotron-nano-3-30b` | 256K | 8K | $0.06 | $0.24 | US regions |
| `nvidia.nemotron-super-3-120b` | 256K | 32K | $0.15 | $0.65 | US regions |
| `openai.gpt-oss-20b` | 128K | 16K documented; 8K observed | $0.07 | $0.20 | US Standard |
| `openai.gpt-oss-120b` | 128K | 16K documented; 8K observed | $0.15 | $0.60 | US Standard |
| `openai.gpt-oss-safeguard-20b` | 128K | 16K | $0.07 | $0.20 | US regions |
| `openai.gpt-oss-safeguard-120b` | 128K | 16K | $0.15 | $0.60 | US regions |
| `qwen.qwen3-32b` | 32K | 8K | $0.1545 | $0.6180 | Published row found for Sydney Standard; no US row found in fetched pricing text |
| `qwen.qwen3-235b-a22b-2507` | TBD | TBD | $0.2266 | $0.9064 | Published row found for Sydney Standard; no US row found in fetched pricing text |
| `qwen.qwen3-coder-30b-a3b-instruct` | 256K | 16K | $0.1545 | $0.6180 | Published row found for Sydney Standard; no US row found in fetched pricing text |
| `qwen.qwen3-coder-480b-a35b-instruct` | 128K | 16K | N/A | N/A | Mantle lists it, but no matching pricing row was found in the AWS pricing page fetch |
| `qwen.qwen3-coder-next` | 256K | TBD | $0.50 | $1.20 | US regions |
| `qwen.qwen3-next-80b-a3b-instruct` | TBD | TBD | $0.15 | $1.20 | US regions |
| `qwen.qwen3-vl-235b-a22b-instruct` | TBD | TBD | $0.53 | $2.66 | US regions |
| `writer.palmyra-vision-7b` | TBD | TBD | $0.15 | $0.60 | US regions |
| `xai.grok-4.20` | TBD | TBD | $1.25 | $2.50 | xAI published pricing; Mantle inference hung in tests |
| `xai.grok-4.3` | 1M | 30K | $1.25 | $2.50 | xAI published pricing |
| `zai.glm-4.6` | TBD | TBD | N/A | N/A | Mantle lists it, but no matching pricing row was found in the AWS pricing page fetch |
| `zai.glm-4.7` | 203K | 4K | $0.60 | $2.20 | US regions |
| `zai.glm-4.7-flash` | 203K | 4K | $0.07 | $0.40 | US regions |
| `zai.glm-5` | 200K | 128K | $1.00 | $3.20 | US regions |

## Small Classification Benchmark

Benchmark shape:

- API: `bedrock-runtime.converse`
- Region: `us-east-1`
- Task: classify one transcript chunk into compact JSON.
- Focused run: 20 trials per candidate.
- Output size: roughly 29-51 output tokens.

For this tiny-output case, output stream TPS is not a useful model-speed measure because many providers flush one chunk. The useful metric is end-to-end latency plus strict JSON validity.

| Model | Context | Max output | Median Latency | P95 Latency | Direct JSON | Notes |
|---|---:|---:|---:|---:|---:|---|
| `mistral.ministral-3-14b-instruct` | 128K | 8K | 0.422s | 0.542s | 0/20 | Fastest, but always fenced JSON |
| `amazon.nova-micro-v1:0` | TBD | <10K observed | 0.514s | 0.628s | 20/20 | Best small-classification default |
| `google.gemma-3-4b-it` | 128K | 8K | 0.546s | 0.617s | 0/20 | Fenced JSON |
| `meta.llama3-1-8b-instruct-v1:0` | 128K | <8K observed | 0.564s | 1.217s | 20/20 | Good speed, higher p95 |
| `zai.glm-4.7-flash` | 203K | 4K current docs | 0.575s | 0.666s | 20/20 | Good backup |
| `qwen.qwen3-coder-30b-a3b-v1:0` | 256K | 16K | 0.583s | 14.734s | 20/20 | Fast median, bad tail latency |
| `meta.llama4-scout-17b-instruct-v1:0` | TBD | <8K observed | 0.606s | 0.685s | 20/20 | Reliable, slightly slower |
| `nvidia.nemotron-nano-3-30b` | 256K | 8K | 0.634s | 0.837s | 20/20 | Reliable |
| `amazon.nova-lite-v1:0` | TBD | <10K observed | 0.651s | 0.775s | 0/20 | Violated schema in this run |

Small-output recommendation:

- Use `amazon.nova-micro-v1:0` for low-cost, reliable single-chunk transcript theme classification.
- Use `zai.glm-4.7-flash` as the backup.
- Avoid `qwen.qwen3-coder-30b-a3b-v1:0` for latency-sensitive single calls because it had long tail outliers.

## Corrected 10k+ Output TPS Benchmark

Benchmark shape:

- API: `bedrock-runtime.converse_stream`
- Region: `us-east-1`
- Workload: 420 transcript chunks in, JSONL classifications out.
- Target: at least 10,000 output tokens.
- `maxTokens`: 12,000.
- Measured from streamed text events and Bedrock usage metadata.

Results:

Note: this benchmark is a historical run from 2026-05-26. Current AWS model cards list `zai.glm-4.7-flash` at 4K max output, so rerun before relying on it for 10K+ completion workloads.

| Model | Context | Max output | Output Tokens | Stream TPS | End-to-End TPS | TTFB | Total Time | Valid Records | Stop Reason |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---|
| `qwen.qwen3-coder-30b-a3b-v1:0` | 256K | 16K | 12,000 | 327.1 | 317.2 | 1.14s | 37.84s | 331/420 | `max_tokens` |
| `nvidia.nemotron-nano-3-30b` | 256K | 8K | 12,000 | 280.8 | 273.1 | 1.18s | 43.93s | 294/420 | `max_tokens` |
| `zai.glm-4.7-flash` | 203K | 4K current docs | 12,000 | 268.4 | 258.3 | 1.73s | 46.46s | 414/420 | `max_tokens` |
| `mistral.ministral-3-14b-instruct` | 128K | 8K | 12,000 | 237.7 | 232.8 | 1.05s | 51.54s | 319/420 | `max_tokens` |
| `google.gemma-3-4b-it` | 128K | 8K | 12,000 | 172.1 | 169.7 | 0.96s | 70.70s | 327/420 | `max_tokens` |

Disqualified for the 10k+ output requirement:

| Model | Context | Max output | Reason |
|---|---:|---:|---|
| `meta.llama4-scout-17b-instruct-v1:0` | TBD | <8K observed | Bedrock rejected `maxTokens=12000`; model limit is below 8192 |
| `meta.llama3-1-8b-instruct-v1:0` | 128K | <8K observed | Bedrock rejected `maxTokens=12000`; model limit is below 8192 |
| `amazon.nova-micro-v1:0` | TBD | <10K observed | Bedrock rejected `maxTokens=12000`; model requires less than 10,000 |
| `amazon.nova-lite-v1:0` | TBD | <10K observed | Bedrock rejected `maxTokens=12000`; model requires less than 10,000 |

Long-output recommendation:

- Use `zai.glm-4.7-flash` for batched transcript classification where completion quality matters. It was not the highest TPS, but it produced 414 valid records out of 420 before hitting the token cap.
- Use `qwen.qwen3-coder-30b-a3b-v1:0` if raw long-output TPS is the priority. It reached 327 stream TPS, but only completed 331 valid records before hitting the token cap.

## Final Routing Recommendation

For this workload:

- Single/small chunk classification: `amazon.nova-micro-v1:0`.
- Large batched classification / long structured output: `zai.glm-4.7-flash`.
- Raw sustained long-output TPS: `qwen.qwen3-coder-30b-a3b-v1:0`.

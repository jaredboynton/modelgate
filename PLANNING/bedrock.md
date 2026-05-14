# Bedrock - Claude in Amazon Bedrock (mantle endpoint)

Source doc: https://platform.claude.com/docs/en/build-with-claude/claude-in-amazon-bedrock

Verified live against AWS account `780401591112` (profile `kepler-admin`, region `us-east-1`) on 2026-05-12.

## TL;DR

Two distinct Bedrock surfaces are usable from this account, both serving the same Claude 4.x family:

| Surface | Endpoint | Auth | Body shape |
|---|---|---|---|
| Mantle (new) | `https://bedrock-mantle.{region}.api.aws/anthropic/v1/messages` | SigV4 OR Bedrock API key (Bearer / `x-api-key`) | Anthropic Messages-compatible API |
| Legacy | `https://bedrock-runtime.{region}.amazonaws.com/model/{modelId}/invoke` | SigV4 OR Bedrock API key (Bearer) | `InvokeModel` / `Converse`, `anthropic_version: bedrock-2023-05-31`, requires inference profile id (`us.anthropic.claude-...`) for on-demand |

For UMP v2: prefer mantle. It eliminates the body-shape divergence that the v1 `bedrock` provider had to translate, so the resolver can pass the request through almost untouched and only swap the auth layer.

## Authentication paths (in order of preference per AWS docs)

1. **Bedrock service role** - admin provisions a role, developer gets `iam:PassRole` on it, Bedrock assumes it. Most secure, longest-lived.
2. **IAM assumed roles** - federated IdP -> `sts:AssumeRole` -> 12h temp creds -> SigV4. Standard pattern.
3. **Bedrock bearer tokens** - 12h short-term tokens minted via `aws-bedrock-token-generator`, sent as `x-api-key` (or `Authorization: Bearer`). Least preferred. Long-term Bedrock API keys also exist and use the same wire format; admins should block them with a deny on `bedrock:CallWithBearerToken` gated by `bedrock:BearerTokenType`.

## This account's IAM posture

Identity: `arn:aws:iam::780401591112:user/jared`

Attached / inherited policies:
- Group `AWSPowerUser` -> `arn:aws:iam::aws:policy/PowerUserAccess`
- User `IAMFullAccess`
- User `cse-bootcamp-user-management`, `jared-iam-v2`
- Inline: `AllowIamSimulation`, `api-catalog-backstage-passrole`, `Assume-jared-admin` (`sts:AssumeRole` -> `arn:aws:iam::780401591112:role/jared-admin`), `ECSPassRole`

`iam simulate-principal-policy` returns `allowed` (no SCP block) for:
- `bedrock-mantle:CreateInference`
- `bedrock:CallWithBearerToken`
- `bedrock:InvokeModel`
- `bedrock:GetFoundationModel`

So this account can both call the new mantle Messages API directly with SigV4 and mint / use Bedrock API keys.

## Bedrock model-access gate (us-east-1)

`aws bedrock list-foundation-models --by-provider anthropic`:

```
anthropic.claude-haiku-4-5-20251001-v1:0   ACTIVE
anthropic.claude-opus-4-7                  ACTIVE
anthropic.claude-opus-4-6-v1               ACTIVE
anthropic.claude-opus-4-5-20251101-v1:0    ACTIVE
anthropic.claude-opus-4-1-20250805-v1:0    ACTIVE
anthropic.claude-opus-4-20250514-v1:0      LEGACY
```

Mantle accepts unversioned ids for the Mantle-enabled Claude rows
(`anthropic.claude-haiku-4-5`, `anthropic.claude-opus-4-7`). Bedrock Runtime is
required for inference-profile rows such as `us.anthropic.claude-sonnet-4-6`
and `us.anthropic.claude-opus-4-6-v1`. UMP routes every Claude alias through
Bedrock only; direct Anthropic credentials, auth stores, and endpoints are
blocked by the source guard test.

`claude-mythos-preview` is NOT enabled - it requires a separate Glasswing-allowlisted account.

## Live verification

### Mantle via SigV4 (PowerUser creds)

```bash
curl https://bedrock-mantle.us-east-1.api.aws/anthropic/v1/messages \
  --aws-sigv4 "aws:amz:us-east-1:bedrock-mantle" \
  --user "$AWS_ACCESS_KEY_ID:$AWS_SECRET_ACCESS_KEY" \
  -H "content-type: application/json" \
  -H "anthropic-version: 2023-06-01" \
  -d '{"model":"anthropic.claude-haiku-4-5","max_tokens":32,
       "messages":[{"role":"user","content":"ping"}]}'
```

-> HTTP 200, `model: claude-haiku-4-5-20251001`. Standard Anthropic Messages response shape (id `msg_bdrk_...`, `usage` includes `cache_creation` / `cache_read` blocks).

### Mantle via Bedrock API key

Local live checks used a long-term key on this account; do not commit the key
material itself.

```
<redacted Bedrock API key>
```

Decoded prefix: `BedrockAPIKey-<redacted>` - account-bound.

Both header forms accepted (HTTP 200 each):

```bash
# Authorization: Bearer
curl https://bedrock-mantle.us-east-1.api.aws/anthropic/v1/messages \
  -H "Authorization: Bearer $BEDROCK_KEY" \
  -H "content-type: application/json" -H "anthropic-version: 2023-06-01" \
  -d '{"model":"anthropic.claude-haiku-4-5","max_tokens":32,
       "messages":[{"role":"user","content":"ping"}]}'

# x-api-key (the form the Anthropic doc shows)
curl https://bedrock-mantle.us-east-1.api.aws/anthropic/v1/messages \
  -H "x-api-key: $BEDROCK_KEY" \
  -H "content-type: application/json" -H "anthropic-version: 2023-06-01" \
  -d '{"model":"anthropic.claude-haiku-4-5","max_tokens":32,
       "messages":[{"role":"user","content":"ping"}]}'
```

### Legacy `bedrock-runtime` with same key

`POST https://bedrock-runtime.us-east-1.amazonaws.com/model/anthropic.claude-haiku-4-5-20251001-v1:0/invoke` with `Authorization: Bearer` -> HTTP 400 routing error ("on-demand throughput isn't supported, use an inference profile"), NOT auth failure. Auth layer is shared between mantle and legacy; only the body shape and model-id form differ.

## Feature support

Supported on mantle:
- Messages API
- Prompt caching (response confirms `cache_creation` / `cache_read_input_tokens`)
- Extended thinking
- Tool use (client-defined tools)
- Citations
- Structured outputs

Not supported on Bedrock at all:
- Anthropic-defined tools (Web Search, Web Fetch, Remote MCP, Memory, Files API, Computer Use, Skills, Code Execution)
- Claude Managed Agents
- Message Batches API
- `/v1/users`

## Quotas / regions

- Default 2M input TPM, raisable to 4M without Anthropic approval. RPM is AWS-side, contact AWS support.
- ZDR available via AWS support ticket.
- us-east-1 supports Global, US, and in-region routing for both Opus 4.7 and Haiku 4.5.

## Implications for UMP v2

1. **Replace the `bedrock` provider with `bedrock-mantle`.** Drop the `InvokeModel` body translator. Let the resolver pass `messages.create` payloads straight through; only swap auth.
2. **Auth layering.** Resolver should accept (in priority order):
   - `AWS_BEARER_TOKEN_BEDROCK` env (long-term Bedrock API key) -> `x-api-key` header on outbound. This is the simplest path and matches what UMP v1 already documents in the auth flow.
   - SigV4 from the AWS SDK chain (`AWS_REGION`, profile, role) when no bearer is set. Use the dedicated `AnthropicBedrockMantle` client for this in TS, or sign manually with `aws:amz:{region}:bedrock-mantle`.
3. **Model ID normalisation.** Strip dated suffixes when routing to mantle; preserve them only for the legacy fallback. Map `claude-haiku-4-5-20251001` -> `anthropic.claude-haiku-4-5` on the wire.
4. **Existing key hygiene.** Rotate any long-term key used for live checks before sharing this doc outside a trusted local workspace. Admins should pair this with an SCP/`bedrock:BearerTokenType` deny where possible.
5. **Legacy fallback worth keeping?** Probably not for UMP v2. Mantle returns prompt-caching usage data the legacy invoke path historically lacked, and removing it eliminates a whole class of body-shape bugs. Only retain legacy if a downstream consumer specifically needs `Converse` semantics.

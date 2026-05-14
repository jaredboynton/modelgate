# Error Handling

Golden rule: every route-facing failure crosses the boundary as `AppError` through `AppResult<T>`.

## Contracts

- Use `AppResult<T>` for handlers, auth loaders, adapters, and upstream calls that can fail into the HTTP surface.
- Return `AppError::ModelNotSupported` for unknown or disallowed model IDs. Do not fall back to another provider.
- Return `AppError::MissingCredential` only after model/provider validation has selected the provider that needs the credential.
- Keep OpenAI-shaped JSON errors centralized through `AppError::into_response` and `openai_error_body`.
- Preserve typed codes that clients and tests assert: `model_not_supported`, `invalid_api_key`, `unsupported_feature`, and `upstream_forbidden`.

## Route Behavior

- Route code should validate request shape and model/provider fit before calling provider auth.
- Missing Codex auth is a `401` authentication error, not a model-routing failure.
- Provider `403` responses map to permission errors; other upstream, I/O, and JSON failures are proxy/upstream failures.
- Unsupported public OpenAI facade routes should return explicit unsupported errors, not proxy to Codex by accident.

## Review Checklist

- Does the new failure path return `AppError` instead of ad hoc tuples or raw JSON?
- Is the status/code mapping covered by a nearby unit or route test?
- Could this error reveal local credentials, tokens, filesystem paths, or live provider response bodies?
- Does the order still prefer `model_not_supported` before missing credentials when the route can decide locally?

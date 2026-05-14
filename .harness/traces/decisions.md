# Decision Trace

Record harness decisions that future agents should not rediscover.

## 2026-05-14: Harness templates are additive

Decision: `.harness/**` coordination templates are added without runtime proxy edits.

Constraint: Other agents may be editing source, docs, CI, or tests in disjoint scopes.

Rejected: Broad repo restructuring inside this lane | outside approved ownership and high conflict risk.

Confidence: high

Scope-risk: narrow

Directive: Use these files for coordination, not as permission to edit paths outside an assigned scope.

Tested: `find .harness -type f | sort`

Not-tested: Full Cargo validation; this lane changes templates only.

---
name: Runtime/API validation issue
about: Report an API, smoke-test, conformance or generated-artifact problem
title: "validation: "
labels: validation
assignees: ""
---

## Failing Surface

- [ ] `/api/help`
- [ ] `/api/validation/status`
- [ ] `/api/actions/dry-run`
- [ ] Runtime state API
- [ ] Artifact API
- [ ] Operational API
- [ ] SDK/conformance
- [ ] Readiness package
- [ ] Other:

## Command Used

```bash
make smoke-api
# or
make eden-api-conformance
# or paste the exact curl command
```

## Expected Signal

What response, artifact or status should have been produced?

## Actual Signal

Paste the smallest relevant output. Do not include `.env` values, credentials,
tokens, private keys or generated private runtime state.

## Environment

- OS:
- Rust version:
- Commit:
- API port:
- State dir:

## Safety And Mutation Boundary

- [ ] The report does not require exposing the API beyond localhost.
- [ ] Any dry-run endpoint remained non-mutating.
- [ ] No secrets or private runtime data are included.

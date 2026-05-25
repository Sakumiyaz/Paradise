---
name: Bug report
about: Report a reproducible defect in Paradise
title: "bug: "
labels: bug
assignees: ""
---

## Summary

Describe the defect in one or two sentences.

## Impact

What workflow is blocked or producing incorrect evidence?

## Minimal Reproduction

```bash
# commands run
```

## Expected Behavior

What should have happened?

## Actual Behavior

What happened instead?

## Logs Or Output

Paste the smallest relevant output. Remove secrets, tokens, local private paths
and generated runtime state that should not be public.

## Environment

- OS:
- Rust version:
- Commit:
- State dir used:

## Validation

Which checks did you run?

```bash
make fmt
make check
make eden-api-conformance
```

## Suspected Area

- [ ] README or docs
- [ ] GARM runtime command
- [ ] Runtime API endpoint
- [ ] GEWC routing or safety gate
- [ ] Validation artifact or readiness package
- [ ] SDK/conformance
- [ ] Unsure

## Security Or Privacy Impact

State whether this involves credentials, `.env`, private data, command
execution, network exposure or runtime state.

Do not paste secrets into this issue.

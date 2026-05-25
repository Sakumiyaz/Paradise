# History Rewrite Playbook

This playbook exists because generated artifacts and backups were removed from
the current tree, but may still exist in Git history. Rewriting history is a
destructive coordination event and was not executed automatically.

## When To Use

Use this only before making the repository public and only after the owner
confirms that force-pushing rewritten history is acceptable.

## Paths Already Removed From Current Tree

```text
eden_core/Cargo.toml.v7
eden_core/__pycache__/evolution.cpython-310.pyc
eden_core/__pycache__/scraper.cpython-310.pyc
eden_core/__pycache__/self_evaluation.cpython-310.pyc
eden_core/src/garm/legacy_sources/eden_repl.rs.bak
eden_core/src/garm/legacy_sources/eden_repl.rs.debug
eden_core/language_reports/tokens_iniciales.json
eden_core/reports/metacog_report_1775856338.json
eden_core/src/physics/optimized_laplacian.rs.bak
```

## Recommended Tooling

Prefer `git filter-repo`:

```bash
python3 -m pip install --user git-filter-repo
```

Or use BFG Repo-Cleaner if that is already the team's standard tool.

## Safe Preparation

Create a fresh mirror backup first:

```bash
git clone --mirror git@github.com:Sakumiyaz/Paradise.git paradise.git.backup
```

Inspect history for sensitive paths:

```bash
git log --all --name-only --pretty=format: |
  rg '(__pycache__|\.pyc$|\.bak$|\.debug$|Cargo\.toml\.v|/reports/|language_reports)'
```

Run a real history-aware secret scanner:

```bash
gitleaks detect --source . --redact
trufflehog git file://. --no-update --fail
```

## Rewrite Command

Run only after explicit owner approval:

```bash
git filter-repo \
  --invert-paths \
  --path eden_core/Cargo.toml.v7 \
  --path eden_core/__pycache__/evolution.cpython-310.pyc \
  --path eden_core/__pycache__/scraper.cpython-310.pyc \
  --path eden_core/__pycache__/self_evaluation.cpython-310.pyc \
  --path eden_core/src/garm/legacy_sources/eden_repl.rs.bak \
  --path eden_core/src/garm/legacy_sources/eden_repl.rs.debug \
  --path eden_core/language_reports/tokens_iniciales.json \
  --path eden_core/reports/metacog_report_1775856338.json \
  --path eden_core/src/physics/optimized_laplacian.rs.bak
```

Then force-push only after notifying collaborators:

```bash
git push --force-with-lease origin master
git push --force-with-lease origin --tags
```

## Post-Rewrite Checks

```bash
make public-audit
make fmt
make check
make eden-api-conformance
```

Confirm GitHub Actions passes after the force-push.

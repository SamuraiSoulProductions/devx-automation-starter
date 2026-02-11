# Copilot Instructions

- Prefer minimal diffs and small PRs.
- Keep CLI output deterministic (stable formatting).
- If CLI help changes, regenerate docs: `python python/tools/tools/gen_docs.py`
- Add/adjust tests for behavior changes.
- Avoid adding dependencies unless clearly justified.

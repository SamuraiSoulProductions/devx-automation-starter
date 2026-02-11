from __future__ import annotations

import subprocess
from pathlib import Path


def find_repo_root(start: Path) -> Path:
    p = start.resolve()
    for parent in (p, *p.parents):
        if (parent / ".git").exists():
            return parent
    raise RuntimeError("Repo root not found (no .git).")


ROOT = find_repo_root(Path(__file__).resolve())
DOCS = ROOT / "docs" / "COMMANDS.md"
GEN = ROOT / "python" / "tools" / "tools" / "gen_docs.py"
BIN = ROOT / "rust" / "devx_cli" / "target" / "debug" / "devx_cli"


def test_docs_are_synced():
    assert BIN.exists(), "Build rust/devx_cli first: (cd rust/devx_cli && cargo build)"
    before = DOCS.read_text(encoding="utf-8") if DOCS.exists() else ""
    subprocess.check_call(["python", str(GEN)])
    after = DOCS.read_text(encoding="utf-8")
    assert after != ""
    assert before == after, "Docs changed — run gen_docs.py and commit the result."

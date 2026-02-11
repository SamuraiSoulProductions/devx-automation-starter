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
BIN = ROOT / "rust" / "devx_cli" / "target" / "debug" / "devx_cli"


def main() -> int:
    if not BIN.exists():
        raise SystemExit("Binary not found. Build it first: (cd rust/devx_cli && cargo build)")

    help_text = subprocess.check_output([str(BIN), "emit-help"], text=True)
    content = (
        "# Commands\\n\\n"
        "This file is auto-generated from `devx_cli --help`.\\n\\n"
        "```text\\n"
        f"{help_text}"
        "```\\n"
    )

    DOCS.parent.mkdir(parents=True, exist_ok=True)
    DOCS.write_text(content, encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

"""Shared utility functions for the codex_package builder."""

from __future__ import annotations

import shutil
import stat
import tempfile
from pathlib import Path
from urllib.request import urlopen


def is_executable(path: Path) -> bool:
    """Check if path has any executable bit set (owner, group, or other)."""
    return bool(path.stat().st_mode & (stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH))


def default_cache_root() -> Path:
    """Return the default cache directory for downloaded artifacts."""
    return Path(tempfile.gettempdir()) / "codex-package"


def download_file(url: str, dest: Path, *, timeout: int = 120) -> None:
    """Download a URL to a destination file with atomic write and cleanup."""
    dest.parent.mkdir(parents=True, exist_ok=True)
    temp_path = dest.with_suffix(f"{dest.suffix}.tmp")
    temp_path.unlink(missing_ok=True)
    try:
        with urlopen(url, timeout=timeout) as response:
            with temp_path.open("wb") as output:
                shutil.copyfileobj(response, output)
        temp_path.replace(dest)
    finally:
        temp_path.unlink(missing_ok=True)


def resolve_output_path(explicit_path: Path | None, default_path: Path | None) -> Path | None:
    """Resolve an explicit path or fall back to a default."""
    if explicit_path is not None:
        return explicit_path.resolve()
    return default_path

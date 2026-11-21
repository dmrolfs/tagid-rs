#!/usr/bin/env python3
# ABOUTME: Checks validator 0.21 is Snowflake-only and the old 0.18/IDNA 0.5 chain is gone across tagid feature sets.
# ABOUTME: Fails on Cargo errors rather than treating a failed inverse query as absence.

from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]
CASES = [
    ("default", [], False),
    ("minimal", ["--no-default-features"], False),
    ("non-snowflake", ["--no-default-features", "--features", "derive,cuid,ulid,uuid,envelope,functional"], False),
    ("lyra", ["--features", "disintegrate,envelope,sqlx,cuid,ulid"], False),
    ("snowflake", ["--features", "snowflake"], True),
    ("minimal-snowflake", ["--no-default-features", "--features", "snowflake"], True),
]


def main():
    for name, features, expected in CASES:
        result = subprocess.run(
            ["cargo", "tree", "-p", "tagid", "--target", "all",
             "-e", "normal,build", "--prefix", "none", "--format", "{p}", *features],
            cwd=ROOT, capture_output=True, text=True, check=True, timeout=120,
        )
        packages = set(re.findall(r"^([\w-]+) v(\S+)", result.stdout, re.MULTILINE))
        if not any(package == "tagid" for package, _ in packages):
            raise RuntimeError(f"{name}: Cargo did not report tagid")
        for dependency, version_prefix, want in [
            ("validator", "0.21.", expected),
            ("proc-macro-error3", "3.", expected),
            ("validator", "0.18.", False),
            ("idna", "0.5.", False),
            ("proc-macro-error", "1.", False),
        ]:
            present = any(package == dependency and version.startswith(version_prefix)
                          for package, version in packages)
            if present != want:
                raise RuntimeError(f"{name}: expected {dependency} {version_prefix} present={want}")
        print(f"PASS {name}: validator present={expected}, old chain absent", flush=True)


if __name__ == "__main__":
    main()

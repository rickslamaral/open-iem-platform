#!/usr/bin/env python3
"""Validate one canonical SemVer against project manifests and optional tag."""
from __future__ import annotations

import argparse
import json
import pathlib
import re
import sys
import tomllib

SEMVER = re.compile(r"^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$")
TAG = re.compile(r"^v(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$")


def read_version(root: pathlib.Path) -> str:
    value = (root / "VERSION").read_text(encoding="utf-8").strip()
    if not SEMVER.fullmatch(value):
        raise ValueError("VERSION must contain exact SemVer X.Y.Z")
    return value


def cargo_version(root: pathlib.Path) -> str:
    data = tomllib.loads((root / "server" / "Cargo.toml").read_text(encoding="utf-8"))
    workspace = data.get("workspace")
    if not isinstance(workspace, dict):
        raise ValueError("workspace table missing")
    package = workspace.get("package")
    if not isinstance(package, dict) or not isinstance(package.get("version"), str):
        raise ValueError("workspace package version missing")
    return package["version"]


def package_version(path: pathlib.Path) -> str:
    data = json.loads(path.read_text(encoding="utf-8"))
    value = data.get("version")
    if not isinstance(value, str):
        raise ValueError(f"package version missing: {path}")
    return value


def validate(root: pathlib.Path, tag: str | None = None) -> str:
    version = read_version(root)
    manifests = [cargo_version(root)] + [
        package_version(root / name)
        for name in ("web/musician/package.json", "web/engineer/package.json")
    ]
    if any(candidate != version for candidate in manifests):
        raise ValueError("VERSION does not match every project manifest")
    if tag is not None and TAG.fullmatch(tag) is None:
        raise ValueError("tag must be exact vX.Y.Z")
    if tag is not None and tag[1:] != version:
        raise ValueError("tag does not match VERSION")
    return version


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=pathlib.Path, default=pathlib.Path(__file__).resolve().parents[1])
    parser.add_argument("--tag")
    args = parser.parse_args()
    try:
        print(validate(args.root.resolve(), args.tag))
    except (OSError, ValueError, json.JSONDecodeError) as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

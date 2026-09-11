#!/usr/bin/env python3
"""Validate Open IEM server release archives before publication."""
from __future__ import annotations

import argparse
import pathlib
import posixpath
import tarfile


ALLOWED_FILES = {"api-server", "open-iem-admin", "README.md", "LICENSE", "CHANGELOG.md"}


def validate(archive: pathlib.Path, required: set[str]) -> None:
    with tarfile.open(archive, mode="r:gz") as bundle:
        members = bundle.getmembers()
        if not members:
            raise ValueError("archive is empty")

        for member in members:
            name = member.name
            if name.startswith("/") or ".." in pathlib.PurePosixPath(name).parts:
                raise ValueError(f"unsafe archive path: {name}")
            if posixpath.normpath(name) != name:
                raise ValueError(f"non-canonical archive path: {name}")

        roots = {member.name.split("/", 1)[0] for member in members}
        if len(roots) != 1 or "" in roots:
            raise ValueError("archive must contain one top-level directory")
        root = roots.pop()
        expected = {f"{root}/{name}" for name in ALLOWED_FILES}
        names = set()
        root_directory_seen = False
        for member in members:
            name = member.name
            if name != root and not name.startswith(f"{root}/"):
                raise ValueError(f"archive member outside top-level directory: {name}")
            if posixpath.normpath(name) != name:
                raise ValueError(f"non-canonical archive path: {name}")
            if member.isdir():
                if name.rstrip("/") != root:
                    raise ValueError(f"unexpected directory: {name}")
                if root_directory_seen:
                    raise ValueError(f"duplicate archive directory: {name}")
                root_directory_seen = True
                continue
            if not member.isfile():
                raise ValueError(f"archive links or special files are forbidden: {name}")
            if name not in expected:
                raise ValueError(f"unexpected archive member: {name}")
            if name in names:
                raise ValueError(f"duplicate archive member: {name}")
            names.add(name)

        if not root_directory_seen:
            raise ValueError(f"archive root directory missing: {root}")
        missing = {f"{root}/{name}" for name in required} - names
        if missing:
            raise ValueError(f"required archive members missing: {', '.join(sorted(missing))}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("archive", type=pathlib.Path)
    parser.add_argument("required", nargs="+", help="required regular file basenames")
    args = parser.parse_args()
    try:
        validate(args.archive, set(args.required))
    except (OSError, tarfile.TarError, ValueError) as exc:
        print(f"archive validation failed: {exc}")
        return 1
    print(f"archive validation passed: {args.archive}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

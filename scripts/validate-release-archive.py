#!/usr/bin/env python3
"""Validate Open IEM server release archives before publication."""
from __future__ import annotations

import argparse
import os
import pathlib
import posixpath
import stat
import tarfile


ALLOWED_FILES = {"api-server", "open-iem-admin", "README.md", "LICENSE", "CHANGELOG.md"}
MAX_ARCHIVE_BYTES = 512 * 1024 * 1024
MAX_MEMBERS = 32
MAX_MEMBER_BYTES = 256 * 1024 * 1024
MAX_UNCOMPRESSED_BYTES = 512 * 1024 * 1024
_WINDOWS_RESERVED_NAMES = {
    "CON",
    "PRN",
    "AUX",
    "NUL",
    *(f"COM{number}" for number in range(1, 10)),
    *(f"LPT{number}" for number in range(1, 10)),
}
_WINDOWS_INVALID_CHARS = set('<>:"|?*')


def _validate_member_name(name: str) -> None:
    if "\\" in name or any(ord(character) < 0x20 or ord(character) == 0x7F for character in name):
        raise ValueError(f"unsafe archive path: {name!r}")
    if name.startswith("/") or ".." in pathlib.PurePosixPath(name).parts:
        raise ValueError(f"unsafe archive path: {name}")
    if posixpath.normpath(name) != name:
        raise ValueError(f"non-canonical archive path: {name}")
    for component in name.split("/"):
        if (
            not component
            or (component not in {".", ".."} and component[-1] in {".", " "})
            or any(character in _WINDOWS_INVALID_CHARS for character in component)
            or (
                component not in {".", ".."}
                and component.split(".", 1)[0].upper() in _WINDOWS_RESERVED_NAMES
            )
        ):
            raise ValueError(f"unsafe cross-platform archive path: {name!r}")


def _consume_member_payload(bundle: tarfile.TarFile, member: tarfile.TarInfo) -> None:
    extracted = bundle.extractfile(member)
    if extracted is None:
        raise ValueError(f"archive member payload is unreadable: {member.name}")
    consumed = 0
    while consumed < member.size:
        chunk = extracted.read(min(1024 * 1024, member.size - consumed))
        if not chunk:
            raise ValueError(f"truncated archive member payload: {member.name}")
        consumed += len(chunk)


def validate(archive: pathlib.Path, required: set[str]) -> None:
    nofollow = getattr(os, "O_NOFOLLOW", None)
    if nofollow is None:
        raise RuntimeError("platform does not support fail-closed archive verification")
    try:
        archive_fd = os.open(
            archive,
            os.O_RDONLY
            | os.O_CLOEXEC
            | os.O_NONBLOCK
            | nofollow,
        )
    except OSError as exc:
        raise ValueError("archive must be a readable regular file") from exc
    try:
        archive_file = os.fdopen(archive_fd, "rb")
    except OSError as exc:
        os.close(archive_fd)
        raise ValueError("archive must be a readable regular file") from exc
    with archive_file:
        metadata = os.fstat(archive_file.fileno())
        if not stat.S_ISREG(metadata.st_mode):
            raise ValueError("archive must be a readable regular file")
        if metadata.st_size > MAX_ARCHIVE_BYTES:
            raise ValueError("archive exceeds compressed size limit")
        with tarfile.open(fileobj=archive_file, mode="r:gz") as bundle:
            members = []
            total_uncompressed_bytes = 0
            for member in bundle:
                if member.size < 0 or member.size > MAX_MEMBER_BYTES:
                    raise ValueError(f"archive member exceeds size limit: {member.name}")
                total_uncompressed_bytes += member.size
                if total_uncompressed_bytes > MAX_UNCOMPRESSED_BYTES:
                    raise ValueError("archive exceeds uncompressed size limit")
                members.append(member)
                if len(members) > MAX_MEMBERS:
                    raise ValueError("archive exceeds member count limit")
            if not members:
                raise ValueError("archive is empty")

            for member in members:
                if member.isfile():
                    _consume_member_payload(bundle, member)

        for member in members:
            name = member.name
            _validate_member_name(name)

        roots = {member.name.split("/", 1)[0] for member in members}
        if len(roots) != 1 or "" in roots:
            raise ValueError("archive must contain one top-level directory")
        root = roots.pop()
        if root in {".", ".."} or posixpath.normpath(root) != root:
            raise ValueError(f"non-canonical archive root: {root}")
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
    required = set(args.required)
    unknown_required = required - ALLOWED_FILES
    if unknown_required:
        parser.error(
            "required basenames are not allowed: "
            + ", ".join(sorted(unknown_required))
        )
    try:
        validate(args.archive, required)
    except (OSError, RuntimeError, tarfile.TarError, ValueError) as exc:
        print(f"archive validation failed: {exc}")
        return 1
    print(f"archive validation passed: {args.archive}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

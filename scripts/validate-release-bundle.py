#!/usr/bin/env python3
"""Validate the release bundle assembled before GitHub Release publication."""
from __future__ import annotations

import argparse
import hashlib
import os
import pathlib
import re
import stat

_ARCHIVE_RE = re.compile(r"^open-iem-server-(?P<version>[^/]+)-(?P<arch>x86_64-linux|aarch64-linux)\.tar\.gz$")
_WEB_RE = re.compile(r"^open-iem-(?:musician-pwa|engineer-ui)-(?P<version>[^/]+)\.tar\.gz$")


def _files(bundle: pathlib.Path) -> list[pathlib.Path]:
    entries = list(bundle.iterdir())
    for entry in entries:
        metadata = entry.lstat()
        if stat.S_ISLNK(metadata.st_mode) or not stat.S_ISREG(metadata.st_mode):
            raise ValueError("bundle contains non-regular entries")
    return entries


def _read_bytes(path: pathlib.Path, label: str) -> bytes:
    nofollow = getattr(os, "O_NOFOLLOW", None)
    if nofollow is None:
        raise RuntimeError("platform does not support fail-closed bundle verification")
    try:
        fd = os.open(path, os.O_RDONLY | os.O_CLOEXEC | nofollow)
    except OSError as exc:
        raise ValueError(f"{label} must be a regular file: {path.name}") from exc
    try:
        metadata = os.fstat(fd)
        if not stat.S_ISREG(metadata.st_mode):
            raise ValueError(f"{label} must be a regular file: {path.name}")
        with os.fdopen(fd, "rb") as stream:
            fd = -1
            return stream.read()
    finally:
        if fd >= 0:
            os.close(fd)


def _read_nonempty(path: pathlib.Path, label: str) -> bytes:
    content = _read_bytes(path, label)
    if not content:
        raise ValueError(f"{label} is empty: {path.name}")
    return content


def _exists(path: pathlib.Path) -> bool:
    try:
        path.lstat()
    except FileNotFoundError:
        return False
    return True


def _read_text(path: pathlib.Path, label: str) -> str:
    try:
        return _read_bytes(path, label).decode("utf-8")
    except UnicodeDecodeError as exc:
        raise ValueError(f"{label} is not valid UTF-8: {path.name}") from exc


def _regular_file(path: pathlib.Path, label: str) -> None:
    _read_bytes(path, label)


def _check_checksum(archive: pathlib.Path, checksum: pathlib.Path) -> None:
    lines = _read_text(checksum, "checksum").splitlines()
    digest = hashlib.sha256(_read_bytes(archive, "archive")).hexdigest()
    expected = f"{digest}  {archive.name}"
    if lines != [expected]:
        raise ValueError(f"checksum mismatch: {checksum.name}")


def validate(bundle: pathlib.Path, version: str) -> None:
    try:
        bundle_metadata = bundle.lstat()
    except OSError as exc:
        raise ValueError("bundle must be a directory") from exc
    if stat.S_ISLNK(bundle_metadata.st_mode) or not stat.S_ISDIR(bundle_metadata.st_mode):
        raise ValueError("bundle must be a directory")
    entries = _files(bundle)
    names = {entry.name for entry in entries}
    server_archives: dict[str, pathlib.Path] = {}
    web_archives: list[pathlib.Path] = []
    for entry in entries:
        match = _ARCHIVE_RE.fullmatch(entry.name)
        if match:
            if match.group("version") != version:
                raise ValueError(f"artifact version mismatch: {entry.name}")
            arch = match.group("arch")
            if arch in server_archives:
                raise ValueError(f"duplicate server archive: {arch}")
            server_archives[arch] = entry
            continue
        web_match = _WEB_RE.fullmatch(entry.name)
        if web_match:
            if web_match.group("version") != version:
                raise ValueError(f"artifact version mismatch: {entry.name}")
            web_archives.append(entry)
            continue
    expected_archives = {
        "x86_64-linux", "aarch64-linux"
    }
    if set(server_archives) != expected_archives:
        raise ValueError("bundle must contain exactly x86_64-linux and aarch64-linux server archives")
    web_names = [archive.name.removesuffix(f"-{version}.tar.gz") for archive in web_archives]
    if len(web_names) != 2 or set(web_names) != {
        "open-iem-musician-pwa", "open-iem-engineer-ui"
    }:
        raise ValueError("bundle must contain exactly both web archives")

    expected_names: set[str] = set()
    sbom_pattern = f"open-iem-server-{version}-sbom.json"
    if sbom_pattern in names:
        expected_names.add(sbom_pattern)
    for archive in [*server_archives.values(), *web_archives]:
        checksum = bundle / f"{archive.name}.sha256"
        if not _exists(checksum):
            raise ValueError(f"missing checksum: {checksum.name}")
        _regular_file(checksum, "checksum")
        _check_checksum(archive, checksum)
        expected_names.update({archive.name, checksum.name})
        if archive.name.startswith("open-iem-server-"):
            signature = bundle / f"{archive.name}.sig"
            if not _exists(signature):
                raise ValueError(f"missing or empty signature: {signature.name}")
            _read_nonempty(signature, "signature")
            expected_names.add(signature.name)
    unexpected = names - expected_names
    if unexpected:
        raise ValueError("unexpected bundle files: " + ", ".join(sorted(unexpected)))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("bundle", type=pathlib.Path)
    parser.add_argument("version")
    args = parser.parse_args()
    try:
        validate(args.bundle, args.version)
    except (OSError, ValueError) as exc:
        print(f"release bundle validation failed: {exc}")
        return 1
    print(f"release bundle validation passed: {args.bundle}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

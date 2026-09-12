#!/usr/bin/env python3
"""Validate the release bundle assembled before GitHub Release publication."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import pathlib
import re
import stat
import subprocess
import sys
import tempfile

_MAX_FILE_BYTES = 512 * 1024 * 1024
_MAX_BUNDLE_BYTES = 2 * 1024 * 1024 * 1024
_MAX_MANIFEST_BYTES = 64 * 1024
_MAX_SIGNATURE_BYTES = 64 * 1024
_MAX_PUBLIC_KEY_BYTES = 64 * 1024
_MAX_BUNDLE_ENTRIES = 32
_COPY_CHUNK_BYTES = 1024 * 1024
_OPENSSL = "/usr/bin/openssl"
_ARCHIVE_RE = re.compile(r"^open-iem-server-(?P<version>[^/]+)-(?P<arch>x86_64-linux|aarch64-linux)\.tar\.gz$")
_WEB_RE = re.compile(r"^open-iem-(?:musician-pwa|engineer-ui)-(?P<version>[^/]+)\.tar\.gz$")


def _nofollow() -> int:
    if not hasattr(os, "O_NOFOLLOW") or not hasattr(os, "O_DIRECTORY"):
        raise RuntimeError("platform does not support fail-closed bundle verification")
    return os.O_NOFOLLOW


def _open_dir(path: pathlib.Path) -> int:
    try:
        fd = os.open(path, os.O_RDONLY | os.O_CLOEXEC | os.O_DIRECTORY | _nofollow())
    except OSError as exc:
        raise ValueError("bundle must be a directory") from exc
    if not stat.S_ISDIR(os.fstat(fd).st_mode):
        os.close(fd)
        raise ValueError("bundle must be a directory")
    return fd


def _entry_names(dir_fd: int) -> list[str]:
    names = os.listdir(dir_fd)
    if len(names) > _MAX_BUNDLE_ENTRIES:
        raise ValueError("bundle exceeds entry count limit")
    return names


def _open_entry(dir_fd: int, name: str, label: str, max_bytes: int | None = None) -> int:
    if max_bytes is None:
        max_bytes = _MAX_FILE_BYTES
    try:
        fd = os.open(name, os.O_RDONLY | os.O_CLOEXEC | os.O_NONBLOCK | _nofollow(), dir_fd=dir_fd)
    except OSError as exc:
        raise ValueError(f"{label} is non-regular: {name}") from exc
    try:
        metadata = os.fstat(fd)
        if not stat.S_ISREG(metadata.st_mode):
            raise ValueError(f"{label} is non-regular: {name}")
        if metadata.st_size > max_bytes:
            raise ValueError(f"{label} exceeds size limit: {name}")
        return fd
    except BaseException:
        os.close(fd)
        raise


def _copy_fd(in_fd: int, out_fd: int, label: str, max_bytes: int = _MAX_FILE_BYTES) -> tuple[int, str]:
    size = 0
    digest = hashlib.sha256()
    while True:
        chunk = os.read(in_fd, _COPY_CHUNK_BYTES)
        if not chunk:
            break
        size += len(chunk)
        if size > max_bytes:
            raise ValueError(f"{label} exceeds size limit")
        digest.update(chunk)
        view = memoryview(chunk)
        while view:
            written = os.write(out_fd, view)
            if written <= 0:
                raise OSError("short output write")
            view = view[written:]
    return size, digest.hexdigest()


def _hash_fd(fd: int, label: str, max_bytes: int = _MAX_FILE_BYTES) -> tuple[int, str]:
    sink = os.open(os.devnull, os.O_WRONLY | os.O_CLOEXEC)
    try:
        return _copy_fd(fd, sink, label, max_bytes)
    finally:
        os.close(sink)


def _has_content(fd: int, label: str) -> bool:
    return os.fstat(fd).st_size > 0


def _read_fd(fd: int, label: str, max_bytes: int = _MAX_FILE_BYTES) -> bytes:
    chunks: list[bytes] = []
    size = 0
    while True:
        chunk = os.read(fd, _COPY_CHUNK_BYTES)
        if not chunk:
            return b"".join(chunks)
        size += len(chunk)
        if size > max_bytes:
            raise ValueError(f"{label} exceeds size limit")
        chunks.append(chunk)


def _read_bytes(path: pathlib.Path, label: str, max_bytes: int | None = None) -> bytes:
    if max_bytes is None:
        max_bytes = _MAX_FILE_BYTES
    parent_fd = _open_dir(path.parent)
    try:
        fd = _open_entry(parent_fd, path.name, label, max_bytes)
        try:
            return _read_fd(fd, label, max_bytes)
        finally:
            os.close(fd)
    finally:
        os.close(parent_fd)


def _read_text_fd(fd: int, name: str, label: str) -> str:
    try:
        return _read_fd(fd, label).decode("utf-8")
    except UnicodeDecodeError as exc:
        raise ValueError(f"{label} is not valid UTF-8: {name}") from exc


def _write_manifest(path: pathlib.Path, lines: list[str]) -> None:
    content = ("\n".join(lines) + "\n").encode()
    if len(content) > _MAX_MANIFEST_BYTES:
        raise ValueError("manifest exceeds size limit")
    parent_fd = _open_dir(path.parent)
    try:
        try:
            fd = os.open(path.name, os.O_WRONLY | os.O_CREAT | os.O_TRUNC | os.O_CLOEXEC | _nofollow(), 0o600, dir_fd=parent_fd)
        except OSError as exc:
            raise ValueError("manifest must be a writable regular file") from exc
        try:
            if not stat.S_ISREG(os.fstat(fd).st_mode):
                raise ValueError("manifest must be a regular file")
            os.fchmod(fd, 0o600)
            view = memoryview(content)
            while view:
                written = os.write(fd, view)
                if written <= 0:
                    raise OSError("short manifest write")
                view = view[written:]
        finally:
            os.close(fd)
    finally:
        os.close(parent_fd)


def _validate_open(dir_fd: int, names: list[str], root: pathlib.Path, version: str, public_key: pathlib.Path | None, manifest: pathlib.Path | None) -> None:
    total = 0
    metadata: dict[str, os.stat_result] = {}
    for name in names:
        fd = _open_entry(dir_fd, name, "bundle entry")
        try:
            metadata[name] = os.fstat(fd)
            total += metadata[name].st_size
        finally:
            os.close(fd)
    if total > _MAX_BUNDLE_BYTES:
        raise ValueError("bundle exceeds total size limit")
    server: dict[str, str] = {}
    web: list[str] = []
    for name in names:
        match = _ARCHIVE_RE.fullmatch(name)
        if match:
            if match.group("version") != version:
                raise ValueError(f"artifact version mismatch: {name}")
            if match.group("arch") in server:
                raise ValueError(f"duplicate server archive: {match.group('arch')}")
            server[match.group("arch")] = name
        else:
            match = _WEB_RE.fullmatch(name)
            if match:
                if match.group("version") != version:
                    raise ValueError(f"artifact version mismatch: {name}")
                web.append(name)
    if set(server) != {"x86_64-linux", "aarch64-linux"}:
        raise ValueError("bundle must contain exactly x86_64-linux and aarch64-linux server archives")
    if len(web) != 2 or {n.removesuffix(f"-{version}.tar.gz") for n in web} != {"open-iem-musician-pwa", "open-iem-engineer-ui"}:
        raise ValueError("bundle must contain exactly both web archives")
    expected: set[str] = set()
    sbom = f"open-iem-server-{version}-sbom.json"
    if sbom in names:
        fd = _open_entry(dir_fd, sbom, "SBOM")
        try:
            try:
                document = json.loads(_read_fd(fd, "SBOM"))
            except json.JSONDecodeError as exc:
                raise ValueError(f"SBOM is not valid JSON: {sbom}") from exc
        finally:
            os.close(fd)
        if not isinstance(document, dict):
            raise ValueError(f"SBOM must contain a JSON object: {sbom}")
        expected.add(sbom)
    digests: dict[str, str] = {}
    for archive in [*server.values(), *web]:
        checksum_name = f"{archive}.sha256"
        if checksum_name not in names:
            raise ValueError(f"missing checksum: {checksum_name}")
        cfd = _open_entry(dir_fd, checksum_name, "checksum")
        afd = _open_entry(dir_fd, archive, "archive")
        try:
            checksum = _read_text_fd(cfd, checksum_name, "checksum").splitlines()
            _, digest = _hash_fd(afd, "archive")
        finally:
            os.close(cfd)
            os.close(afd)
        if checksum != [f"{digest}  {archive}"]:
            raise ValueError(f"checksum mismatch: {checksum_name}")
        expected.update({archive, checksum_name})
        digests[archive] = digest
        if archive.startswith("open-iem-server-"):
            sig_name = f"{archive}.sig"
            if sig_name not in names:
                raise ValueError(f"missing or empty signature: {sig_name}")
            sfd = _open_entry(dir_fd, sig_name, "signature", _MAX_SIGNATURE_BYTES)
            verify_fd = _open_entry(dir_fd, archive, "archive") if public_key is not None else -1
            try:
                if not _has_content(sfd, "signature"):
                    raise ValueError(f"missing or empty signature: {sig_name}")
                if public_key is not None:
                    os.set_inheritable(verify_fd, True)
                    os.set_inheritable(sfd, True)
                    verifier = pathlib.Path(__file__).with_name("verify-release-signature.py")
                    result = subprocess.run(
                        [sys.executable, str(verifier), f"/proc/self/fd/{verify_fd}",
                         f"/proc/self/fd/{sfd}", str(public_key)],
                        check=False, capture_output=True, text=True, timeout=30,
                        pass_fds=(verify_fd, sfd),
                    )
                    if result.returncode != 0:
                        detail = (result.stdout or result.stderr).strip()
                        raise ValueError(f"invalid server signature: {sig_name}: {detail}")
            finally:
                os.close(sfd)
                if verify_fd >= 0:
                    os.close(verify_fd)
            expected.add(sig_name)
    unexpected = set(names) - expected
    if unexpected:
        raise ValueError("unexpected bundle files: " + ", ".join(sorted(unexpected)))
    if manifest is not None:
        lines = []
        for name in sorted(expected):
            fd = _open_entry(dir_fd, name, "manifest artifact")
            sink = os.open(os.devnull, os.O_WRONLY | os.O_CLOEXEC)
            try:
                _, digest = _copy_fd(fd, sink, "manifest artifact")
            finally:
                os.close(sink)
                os.close(fd)
            lines.append(f"{digest}  {name}")
        _write_manifest(manifest, lines)


def _validate(bundle: pathlib.Path, version: str, public_key: pathlib.Path | None = None, manifest: pathlib.Path | None = None) -> None:
    fd = _open_dir(bundle)
    try:
        _validate_open(fd, _entry_names(fd), bundle, version, public_key, manifest)
    finally:
        os.close(fd)


def validate(bundle: pathlib.Path, version: str, public_key: pathlib.Path | None = None, manifest: pathlib.Path | None = None) -> None:
    _validate(bundle, version, public_key, manifest)


def _remove_tree_at(parent_fd: int, name: str) -> None:
    try:
        fd = os.open(name, os.O_RDONLY | os.O_CLOEXEC | os.O_DIRECTORY | _nofollow(), dir_fd=parent_fd)
    except FileNotFoundError:
        return
    try:
        for child in os.listdir(fd):
            try:
                os.unlink(child, dir_fd=fd)
            except IsADirectoryError:
                _remove_tree_at(fd, child)
        os.rmdir(name, dir_fd=parent_fd)
    finally:
        os.close(fd)


def _remove_tree_fd(path: pathlib.Path) -> None:
    parent_fd = _open_dir(path.parent)
    try:
        _remove_tree_at(parent_fd, path.name)
    finally:
        os.close(parent_fd)


def validate_to_output(bundle: pathlib.Path, version: str, output: pathlib.Path, public_key: pathlib.Path | None = None, manifest: pathlib.Path | None = None) -> None:
    parent_fd = _open_dir(output.parent)
    source_fd = -1
    staging: pathlib.Path | None = None
    try:
        try:
            os.stat(output.name, dir_fd=parent_fd, follow_symlinks=False)
        except FileNotFoundError:
            pass
        else:
            raise ValueError(f"output directory already exists: {output}")
        source_fd = _open_dir(bundle)
        names = _entry_names(source_fd)
        staging = pathlib.Path(tempfile.mkdtemp(prefix=f".{output.name}.", dir=output.parent))
        staging_fd = _open_dir(staging)
        copied_bytes = 0
        try:
            for name in names:
                in_fd = _open_entry(source_fd, name, "bundle entry")
                try:
                    out_fd = os.open(name, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_CLOEXEC | _nofollow(), 0o600, dir_fd=staging_fd)
                    try:
                        copied, _ = _copy_fd(in_fd, out_fd, f"bundle entry: {name}")
                        copied_bytes += copied
                        if copied_bytes > _MAX_BUNDLE_BYTES:
                            raise ValueError("bundle exceeds total size limit")
                    finally:
                        os.close(out_fd)
                finally:
                    os.close(in_fd)
        finally:
            os.close(staging_fd)
        _validate(staging, version, public_key, manifest)
        os.rename(staging.name, output.name, src_dir_fd=parent_fd, dst_dir_fd=parent_fd)
        staging = None
    finally:
        if source_fd >= 0:
            os.close(source_fd)
        if staging is not None:
            _remove_tree_at(parent_fd, staging.name)
        os.close(parent_fd)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("bundle", type=pathlib.Path)
    parser.add_argument("version")
    parser.add_argument("--public-key", type=pathlib.Path)
    parser.add_argument("--manifest", type=pathlib.Path)
    parser.add_argument("--output", type=pathlib.Path)
    args = parser.parse_args()
    try:
        if args.output is None:
            validate(args.bundle, args.version, args.public_key, args.manifest)
        else:
            validate_to_output(args.bundle, args.version, args.output, args.public_key, args.manifest)
    except (OSError, ValueError, subprocess.TimeoutExpired) as exc:
        print(f"release bundle validation failed: {exc}")
        return 1
    print(f"release bundle validation passed: {args.bundle}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

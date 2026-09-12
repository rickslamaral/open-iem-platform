#!/usr/bin/env python3
"""Verify a detached Ed25519 signature for a release artifact."""
from __future__ import annotations

import argparse
import os
import pathlib
import stat
import subprocess


_OPENSSL = "/usr/bin/openssl"


def _open_regular(path: pathlib.Path, label: str) -> int:
    nofollow = getattr(os, "O_NOFOLLOW", None)
    if nofollow is None:
        raise RuntimeError("platform does not support fail-closed symlink verification")
    try:
        fd = os.open(
            path,
            os.O_RDONLY
            | os.O_CLOEXEC
            | os.O_NONBLOCK
            | nofollow,
        )
    except OSError as exc:
        raise ValueError(f"{label} must be a regular file: {path}") from exc
    try:
        if not stat.S_ISREG(os.fstat(fd).st_mode):
            raise ValueError(f"{label} must be a regular file: {path}")
        return fd
    except BaseException:
        os.close(fd)
        raise


def verify(artifact: pathlib.Path, signature: pathlib.Path, public_key: pathlib.Path) -> None:
    descriptors: list[int] = []
    try:
        for path, label in ((artifact, "artifact"), (signature, "signature"), (public_key, "public key")):
            descriptors.append(_open_regular(path, label))
        artifact_fd, signature_fd, public_key_fd = descriptors
        artifact_ref = f"/proc/self/fd/{artifact_fd}"
        signature_ref = f"/proc/self/fd/{signature_fd}"
        public_key_ref = f"/proc/self/fd/{public_key_fd}"
        try:
            key_type = subprocess.run(
                [_OPENSSL, "pkey", "-pubin", "-in", public_key_ref, "-text", "-noout"],
                check=False,
                capture_output=True,
                text=True,
                timeout=15,
                pass_fds=tuple(descriptors),
            )
            if key_type.returncode != 0 or not (key_type.stdout or "").lstrip().startswith("ED25519 Public-Key"):
                raise ValueError("public key must be an Ed25519 key")
            result = subprocess.run(
                [
                    _OPENSSL,
                    "pkeyutl",
                    "-verify",
                    "-pubin",
                    "-inkey",
                    public_key_ref,
                    "-sigfile",
                    signature_ref,
                    "-in",
                    artifact_ref,
                ],
                check=False,
                capture_output=True,
                text=True,
                timeout=15,
                pass_fds=tuple(descriptors),
            )
        except FileNotFoundError as exc:
            raise RuntimeError("openssl is required to verify release signatures") from exc
        except OSError as exc:
            raise RuntimeError("unable to execute openssl") from exc
        except subprocess.TimeoutExpired as exc:
            raise RuntimeError("signature verification timed out") from exc
    finally:
        for fd in descriptors:
            os.close(fd)
    if result.returncode != 0:
        detail = (result.stderr or result.stdout).strip()
        raise ValueError(f"release signature verification failed{': ' + detail if detail else ''}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("artifact", type=pathlib.Path)
    parser.add_argument("signature", type=pathlib.Path)
    parser.add_argument("public_key", type=pathlib.Path)
    args = parser.parse_args()
    try:
        verify(args.artifact, args.signature, args.public_key)
    except (OSError, RuntimeError, ValueError) as exc:
        print(f"signature verification failed: {exc}")
        return 1
    print(f"signature verification passed: {args.artifact}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

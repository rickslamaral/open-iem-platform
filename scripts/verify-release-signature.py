#!/usr/bin/env python3
"""Verify a detached Ed25519 signature for a release artifact."""
from __future__ import annotations

import argparse
import pathlib
import subprocess


def verify(artifact: pathlib.Path, signature: pathlib.Path, public_key: pathlib.Path) -> None:
    for path, label in ((artifact, "artifact"), (signature, "signature"), (public_key, "public key")):
        if not path.is_file() or path.is_symlink():
            raise ValueError(f"{label} must be a regular file: {path}")
    try:
        result = subprocess.run(
            [
                "openssl",
                "pkeyutl",
                "-verify",
                "-pubin",
                "-inkey",
                str(public_key),
                "-sigfile",
                str(signature),
                "-in",
                str(artifact),
            ],
            check=False,
            capture_output=True,
            text=True,
            timeout=15,
        )
    except FileNotFoundError as exc:
        raise RuntimeError("openssl is required to verify release signatures") from exc
    except subprocess.TimeoutExpired as exc:
        raise RuntimeError("signature verification timed out") from exc
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

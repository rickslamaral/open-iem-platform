import importlib.machinery
import importlib.util
import subprocess
from pathlib import Path
from typing import cast

SPEC = cast(
    importlib.machinery.ModuleSpec,
    importlib.util.spec_from_file_location(
        "verify_release_signature", Path(__file__).parents[1] / "scripts/verify-release-signature.py"
    ),
)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader
SPEC.loader.exec_module(MODULE)


def make_keys(tmp_path):
    private = tmp_path / "private.pem"
    public = tmp_path / "public.pem"
    subprocess.run(
        ["openssl", "genpkey", "-algorithm", "ed25519", "-out", str(private)], check=True, capture_output=True
    )
    subprocess.run(
        ["openssl", "pkey", "-in", str(private), "-pubout", "-out", str(public)], check=True, capture_output=True
    )
    return private, public


def sign(private, artifact, signature):
    subprocess.run(
        ["openssl", "pkeyutl", "-sign", "-inkey", str(private), "-in", str(artifact), "-out", str(signature)],
        check=True,
        capture_output=True,
    )


def test_valid_signature_passes(tmp_path):
    private, public = make_keys(tmp_path)
    artifact = tmp_path / "artifact.tar.gz"
    signature = tmp_path / "artifact.tar.gz.sig"
    artifact.write_bytes(b"release archive")
    sign(private, artifact, signature)
    MODULE.verify(artifact, signature, public)


def test_modified_artifact_fails(tmp_path):
    private, public = make_keys(tmp_path)
    artifact = tmp_path / "artifact.tar.gz"
    signature = tmp_path / "artifact.tar.gz.sig"
    artifact.write_bytes(b"release archive")
    sign(private, artifact, signature)
    artifact.write_bytes(b"tampered archive")
    try:
        MODULE.verify(artifact, signature, public)
    except ValueError as exc:
        assert "verification failed" in str(exc)
    else:
        raise AssertionError("modified artifact accepted")


def test_missing_signature_fails(tmp_path):
    _, public = make_keys(tmp_path)
    artifact = tmp_path / "artifact.tar.gz"
    artifact.write_bytes(b"release archive")
    try:
        MODULE.verify(artifact, tmp_path / "missing.sig", public)
    except ValueError as exc:
        assert "signature must be a regular file" in str(exc)
    else:
        raise AssertionError("missing signature accepted")


def test_invalid_signature_fails(tmp_path):
    private, public = make_keys(tmp_path)
    artifact = tmp_path / "artifact.tar.gz"
    signature = tmp_path / "artifact.tar.gz.sig"
    artifact.write_bytes(b"release archive")
    signature.write_bytes(b"invalid signature")
    try:
        MODULE.verify(artifact, signature, public)
    except ValueError as exc:
        assert "verification failed" in str(exc)
    else:
        raise AssertionError("invalid signature accepted")


def test_non_ed25519_public_key_fails(tmp_path):
    private, public = make_keys(tmp_path)
    rsa_public = tmp_path / "rsa-public.pem"
    rsa_private = tmp_path / "rsa-private.pem"
    subprocess.run(
        ["openssl", "genpkey", "-algorithm", "RSA", "-pkeyopt", "rsa_keygen_bits:2048", "-out", str(rsa_private)],
        check=True,
        capture_output=True,
    )
    subprocess.run(
        ["openssl", "pkey", "-in", str(rsa_private), "-pubout", "-out", str(rsa_public)],
        check=True,
        capture_output=True,
    )
    artifact = tmp_path / "artifact.tar.gz"
    signature = tmp_path / "artifact.tar.gz.sig"
    artifact.write_bytes(b"release archive")
    sign(private, artifact, signature)
    try:
        MODULE.verify(artifact, signature, rsa_public)
    except ValueError as exc:
        assert "Ed25519" in str(exc)
    else:
        raise AssertionError("non-Ed25519 public key accepted")


def test_oversized_signature_fails_closed(tmp_path):
    private, public = make_keys(tmp_path)
    artifact = tmp_path / "artifact.tar.gz"
    signature = tmp_path / "artifact.tar.gz.sig"
    artifact.write_bytes(b"release archive")
    sign(private, artifact, signature)
    signature.write_bytes(b"x" * (MODULE._MAX_SIGNATURE_BYTES + 1))
    try:
        MODULE.verify(artifact, signature, public)
    except ValueError as exc:
        assert "signature exceeds size limit" in str(exc)
    else:
        raise AssertionError("oversized signature accepted")


def test_oversized_public_key_fails_closed(tmp_path):
    private, public = make_keys(tmp_path)
    artifact = tmp_path / "artifact.tar.gz"
    signature = tmp_path / "artifact.tar.gz.sig"
    artifact.write_bytes(b"release archive")
    sign(private, artifact, signature)
    public.write_bytes(b"x" * (MODULE._MAX_PUBLIC_KEY_BYTES + 1))
    try:
        MODULE.verify(artifact, signature, public)
    except ValueError as exc:
        assert "public key exceeds size limit" in str(exc)
    else:
        raise AssertionError("oversized public key accepted")


def test_symlink_artifact_fails_closed(tmp_path):
    private, public = make_keys(tmp_path)
    artifact_target = tmp_path / "artifact-target.tar.gz"
    artifact = tmp_path / "artifact.tar.gz"
    signature = tmp_path / "artifact.tar.gz.sig"
    artifact_target.write_bytes(b"release archive")
    sign(private, artifact_target, signature)
    artifact.symlink_to(artifact_target)
    try:
        MODULE.verify(artifact, signature, public)
    except ValueError as exc:
        assert "artifact must be a regular file" in str(exc)
    else:
        raise AssertionError("symlink artifact accepted")


def test_symlink_signature_fails_closed(tmp_path):
    private, public = make_keys(tmp_path)
    artifact = tmp_path / "artifact.tar.gz"
    signature_target = tmp_path / "signature-target.sig"
    signature = tmp_path / "artifact.tar.gz.sig"
    artifact.write_bytes(b"release archive")
    sign(private, artifact, signature_target)
    signature.symlink_to(signature_target)
    try:
        MODULE.verify(artifact, signature, public)
    except ValueError as exc:
        assert "signature must be a regular file" in str(exc)
    else:
        raise AssertionError("symlink signature accepted")


def test_symlink_public_key_fails(tmp_path):
    private, public = make_keys(tmp_path)
    artifact = tmp_path / "artifact.tar.gz"
    signature = tmp_path / "artifact.tar.gz.sig"
    linked_key = tmp_path / "linked-public.pem"
    artifact.write_bytes(b"release archive")
    sign(private, artifact, signature)
    linked_key.symlink_to(public)
    try:
        MODULE.verify(artifact, signature, linked_key)
    except ValueError as exc:
        assert "public key must be a regular file" in str(exc)
    else:
        raise AssertionError("symlink public key accepted")

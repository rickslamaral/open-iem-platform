import hashlib
import importlib.machinery
import os
import importlib.util
import subprocess
import tarfile
from pathlib import Path
from typing import cast

SPEC = cast(
    importlib.machinery.ModuleSpec,
    importlib.util.spec_from_file_location(
        "validate_release_bundle", Path(__file__).parents[1] / "scripts/validate-release-bundle.py"
    ),
)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader
SPEC.loader.exec_module(MODULE)


VERSION = "0.3.1"


def add_file(bundle, name, content=b"artifact"):
    path = bundle / name
    path.write_bytes(content)
    return path


def add_archive(bundle, name):
    archive = add_file(bundle, name)
    add_file(bundle, f"{name}.sha256", f"{hashlib.sha256(archive.read_bytes()).hexdigest()}  {name}\n".encode())
    return archive


def make_valid_bundle(tmp_path):
    bundle = tmp_path / "dist"
    bundle.mkdir()
    for arch in ("x86_64-linux", "aarch64-linux"):
        archive = add_archive(bundle, f"open-iem-server-{VERSION}-{arch}.tar.gz")
        add_file(bundle, f"{archive.name}.sig", b"signature")
    add_archive(bundle, f"open-iem-musician-pwa-{VERSION}.tar.gz")
    add_archive(bundle, f"open-iem-engineer-ui-{VERSION}.tar.gz")
    return bundle


def make_signed_bundle(tmp_path):
    bundle = make_valid_bundle(tmp_path)
    private, public = tmp_path / "private.pem", tmp_path / "public.pem"
    subprocess.run(["openssl", "genpkey", "-algorithm", "ed25519", "-out", str(private)], check=True, capture_output=True)
    subprocess.run(["openssl", "pkey", "-in", str(private), "-pubout", "-out", str(public)], check=True, capture_output=True)
    for archive in bundle.glob("open-iem-server-*.tar.gz"):
        subprocess.run(["openssl", "pkeyutl", "-sign", "-rawin", "-inkey", str(private), "-in", str(archive), "-out", str(bundle / f"{archive.name}.sig")], check=True, capture_output=True)
    return bundle, public


def test_public_key_verifies_server_signatures(tmp_path):
    bundle, public = make_signed_bundle(tmp_path)
    MODULE.validate(bundle, VERSION, public)


def test_public_key_rejects_forged_server_signature(tmp_path):
    bundle, public = make_signed_bundle(tmp_path)
    (bundle / f"open-iem-server-{VERSION}-x86_64-linux.tar.gz.sig").write_bytes(b"forged")
    try:
        MODULE.validate(bundle, VERSION, public)
    except ValueError as exc:
        assert "invalid server signature" in str(exc)
    else:
        raise AssertionError("forged server signature accepted")


def test_manifest_lists_validated_files_and_digests(tmp_path):
    bundle = make_valid_bundle(tmp_path)
    manifest = tmp_path / "release-manifest.sha256"
    MODULE.validate(bundle, VERSION, manifest=manifest)
    lines = manifest.read_text(encoding="utf-8").splitlines()
    assert len(lines) == 10
    for line in lines:
        digest, name = line.split("  ")
        assert digest == hashlib.sha256((bundle / name).read_bytes()).hexdigest()


def test_valid_bundle_passes(tmp_path):
    MODULE.validate(make_valid_bundle(tmp_path), VERSION)


def test_bundle_entry_count_limit_fails(tmp_path):
    bundle = make_valid_bundle(tmp_path)
    for index in range(MODULE._MAX_BUNDLE_ENTRIES - len(list(bundle.iterdir())) + 1):
        add_file(bundle, f"extra-{index}")
    try:
        MODULE.validate(bundle, VERSION)
    except ValueError as exc:
        assert "entry count limit" in str(exc)
    else:
        raise AssertionError("bundle with too many entries accepted")


def test_validate_to_output_snapshots_validated_files(tmp_path):
    bundle = make_valid_bundle(tmp_path)
    output = tmp_path / "release-upload"
    manifest = tmp_path / "release-manifest.sha256"
    MODULE.validate_to_output(bundle, VERSION, output, manifest=manifest)
    assert {path.name for path in output.iterdir()} == {path.name for path in bundle.iterdir()}
    for path in output.iterdir():
        assert path.read_bytes() == (bundle / path.name).read_bytes()


def test_validate_to_output_removes_invalid_staging(tmp_path):
    bundle = make_valid_bundle(tmp_path)
    add_file(bundle, "unexpected.txt")
    output = tmp_path / "release-upload"
    try:
        MODULE.validate_to_output(bundle, VERSION, output)
    except ValueError as exc:
        assert "unexpected bundle files" in str(exc)
    else:
        raise AssertionError("invalid bundle staged")
    assert not output.exists()
    assert not list(tmp_path.glob(".release-upload.*"))


def test_validate_to_output_publishes_only_after_validation(tmp_path, monkeypatch):
    bundle = make_valid_bundle(tmp_path)
    output = tmp_path / "release-upload"
    original = MODULE._validate

    def fail_after_snapshot(path, *args, **kwargs):
        assert path != bundle
        raise ValueError("validation sentinel")

    monkeypatch.setattr(MODULE, "_validate", fail_after_snapshot)
    try:
        MODULE.validate_to_output(bundle, VERSION, output)
    except ValueError as exc:
        assert "validation sentinel" in str(exc)
    else:
        raise AssertionError("failed validation published output")
    assert not output.exists()
    assert not list(tmp_path.glob(".release-upload.*"))
    monkeypatch.setattr(MODULE, "_validate", original)


def test_snapshot_copy_uses_bounded_reads(tmp_path, monkeypatch):
    bundle = make_valid_bundle(tmp_path)
    observed = []
    original_read = MODULE.os.read

    def bounded_read(fd, size):
        observed.append(size)
        return original_read(fd, size)

    monkeypatch.setattr(MODULE.os, "read", bounded_read)
    MODULE.validate_to_output(bundle, VERSION, tmp_path / "release-upload")
    assert observed
    assert max(observed) <= MODULE._COPY_CHUNK_BYTES


def test_invalid_sbom_fails(tmp_path):
    bundle = make_valid_bundle(tmp_path)
    add_file(bundle, f"open-iem-server-{VERSION}-sbom.json", b"not-json")
    try:
        MODULE.validate(bundle, VERSION)
    except ValueError as exc:
        assert "SBOM is not valid JSON" in str(exc)
    else:
        raise AssertionError("invalid SBOM accepted")


def test_manifest_symlink_fails(tmp_path):
    bundle = make_valid_bundle(tmp_path)
    target = tmp_path / "outside-manifest"
    target.write_bytes(b"keep")
    manifest = tmp_path / "manifest"
    manifest.symlink_to(target)
    try:
        MODULE.validate(bundle, VERSION, manifest=manifest)
    except ValueError as exc:
        assert "manifest" in str(exc)
    else:
        raise AssertionError("manifest symlink accepted")
    assert target.read_bytes() == b"keep"


def test_file_size_limit_fails(tmp_path, monkeypatch):
    bundle = make_valid_bundle(tmp_path)
    monkeypatch.setattr(MODULE, "_MAX_FILE_BYTES", 1)
    try:
        MODULE.validate(bundle, VERSION)
    except ValueError as exc:
        assert "size limit" in str(exc)
    else:
        raise AssertionError("oversized bundle file accepted")


def test_missing_server_signature_fails(tmp_path):
    bundle = make_valid_bundle(tmp_path)
    (bundle / f"open-iem-server-{VERSION}-aarch64-linux.tar.gz.sig").unlink()
    try:
        MODULE.validate(bundle, VERSION)
    except ValueError as exc:
        assert "missing or empty signature" in str(exc)
    else:
        raise AssertionError("bundle without server signature accepted")


def test_bad_checksum_fails(tmp_path):
    bundle = make_valid_bundle(tmp_path)
    checksum = bundle / f"open-iem-server-{VERSION}-x86_64-linux.tar.gz.sha256"
    checksum.write_text("0" * 64 + "  wrong.tar.gz\n", encoding="utf-8")
    try:
        MODULE.validate(bundle, VERSION)
    except ValueError as exc:
        assert "checksum mismatch" in str(exc)
    else:
        raise AssertionError("bundle with invalid checksum accepted")


def test_missing_architecture_fails(tmp_path):
    bundle = make_valid_bundle(tmp_path)
    for path in bundle.glob(f"*aarch64-linux.tar.gz*"):
        path.unlink()
    try:
        MODULE.validate(bundle, VERSION)
    except ValueError as exc:
        assert "exactly x86_64-linux and aarch64-linux" in str(exc)
    else:
        raise AssertionError("bundle missing architecture accepted")


def test_wrong_version_fails(tmp_path):
    bundle = make_valid_bundle(tmp_path)
    archive = bundle / f"open-iem-engineer-ui-{VERSION}.tar.gz"
    archive.rename(bundle / "open-iem-engineer-ui-9.9.9.tar.gz")
    try:
        MODULE.validate(bundle, VERSION)
    except ValueError as exc:
        assert "version mismatch" in str(exc)
    else:
        raise AssertionError("bundle with wrong version accepted")


def test_unexpected_file_fails(tmp_path):
    bundle = make_valid_bundle(tmp_path)
    add_file(bundle, "release-notes.txt")
    try:
        MODULE.validate(bundle, VERSION)
    except ValueError as exc:
        assert "unexpected bundle files" in str(exc)
    else:
        raise AssertionError("bundle with unexpected file accepted")


def test_non_regular_entry_fails(tmp_path):
    bundle = make_valid_bundle(tmp_path)
    (bundle / "nested").mkdir()
    try:
        MODULE.validate(bundle, VERSION)
    except ValueError as exc:
        assert "non-regular" in str(exc)
    else:
        raise AssertionError("bundle with directory accepted")


def test_symlink_entry_fails(tmp_path):
    bundle = make_valid_bundle(tmp_path)
    target = tmp_path / "outside"
    target.write_bytes(b"outside bundle")
    (bundle / f"open-iem-server-{VERSION}-x86_64-linux.tar.gz").unlink()
    (bundle / f"open-iem-server-{VERSION}-x86_64-linux.tar.gz").symlink_to(target)
    try:
        MODULE.validate(bundle, VERSION)
    except ValueError as exc:
        assert "non-regular" in str(exc)
    else:
        raise AssertionError("bundle symlink accepted")


def test_symlink_checksum_fails(tmp_path):
    bundle = make_valid_bundle(tmp_path)
    checksum = bundle / f"open-iem-server-{VERSION}-x86_64-linux.tar.gz.sha256"
    target = tmp_path / "checksum"
    target.write_bytes(checksum.read_bytes())
    checksum.unlink()
    checksum.symlink_to(target)
    try:
        MODULE.validate(bundle, VERSION)
    except ValueError as exc:
        assert "non-regular" in str(exc)
    else:
        raise AssertionError("checksum symlink accepted")


def test_fifo_entry_fails_without_blocking(tmp_path):
    bundle = make_valid_bundle(tmp_path)
    fifo = bundle / f"open-iem-server-{VERSION}-x86_64-linux.tar.gz"
    fifo.unlink()
    os.mkfifo(fifo)
    try:
        MODULE.validate(bundle, VERSION)
    except ValueError as exc:
        assert "non-regular" in str(exc)
    else:
        raise AssertionError("FIFO bundle entry accepted")


def test_snapshot_enforces_aggregate_copy_limit(tmp_path, monkeypatch):
    bundle = make_valid_bundle(tmp_path)
    monkeypatch.setattr(MODULE, "_MAX_BUNDLE_BYTES", 1)
    output = tmp_path / "release-upload"
    try:
        MODULE.validate_to_output(bundle, VERSION, output)
    except ValueError as exc:
        assert "total size limit" in str(exc)
    else:
        raise AssertionError("oversized aggregate snapshot accepted")
    assert not output.exists()
    assert not list(tmp_path.glob(".release-upload.*"))

import hashlib
import importlib.machinery
import importlib.util
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


def test_valid_bundle_passes(tmp_path):
    MODULE.validate(make_valid_bundle(tmp_path), VERSION)


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

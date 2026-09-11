import importlib.machinery
import importlib.util
import tarfile
from pathlib import Path
from typing import cast


SPEC = cast(
    importlib.machinery.ModuleSpec,
    importlib.util.spec_from_file_location(
        "validate_release_archive", Path(__file__).parents[1] / "scripts/validate-release-archive.py"
    ),
)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader
SPEC.loader.exec_module(MODULE)


def make_archive(path, members):
    with tarfile.open(path, "w:gz") as bundle:
        for name, kind in members:
            info = tarfile.TarInfo(name)
            if kind == "dir":
                info.type = tarfile.DIRTYPE
            elif kind == "symlink":
                info.type = tarfile.SYMTYPE
                info.linkname = "/etc/passwd"
            else:
                info.size = 1
            bundle.addfile(info, None if kind != "file" else __import__("io").BytesIO(b"x"))


def valid_members():
    root = "open-iem-server-1.2.3-aarch64-linux"
    return [(f"{root}/", "dir"), (f"{root}/api-server", "file"), (f"{root}/open-iem-admin", "file")]


def test_valid_archive_passes(tmp_path):
    archive = tmp_path / "valid.tar.gz"
    make_archive(archive, valid_members())
    MODULE.validate(archive, {"api-server", "open-iem-admin"})


def test_explicit_root_directory_required(tmp_path):
    archive = tmp_path / "missing-root.tar.gz"
    make_archive(
        archive,
        [
            ("open-iem-server-1.2.3-aarch64-linux/api-server", "file"),
            ("open-iem-server-1.2.3-aarch64-linux/open-iem-admin", "file"),
        ],
    )
    try:
        MODULE.validate(archive, {"api-server", "open-iem-admin"})
    except ValueError as exc:
        assert "archive root directory missing" in str(exc)
    else:
        raise AssertionError("archive without explicit root directory accepted")


def test_path_traversal_fails(tmp_path):
    archive = tmp_path / "traversal.tar.gz"
    make_archive(archive, valid_members() + [("../escape", "file")])
    try:
        MODULE.validate(archive, {"api-server", "open-iem-admin"})
    except ValueError as exc:
        assert "unsafe archive path" in str(exc)
    else:
        raise AssertionError("traversal archive accepted")


def test_symlink_fails(tmp_path):
    archive = tmp_path / "symlink.tar.gz"
    make_archive(archive, valid_members() + [("open-iem-server-1.2.3-aarch64-linux/link", "symlink")])
    try:
        MODULE.validate(archive, {"api-server", "open-iem-admin"})
    except ValueError as exc:
        assert "links" in str(exc)
    else:
        raise AssertionError("symlink archive accepted")


def test_duplicate_file_fails(tmp_path):
    archive = tmp_path / "duplicate.tar.gz"
    make_archive(archive, valid_members() + [("open-iem-server-1.2.3-aarch64-linux/api-server", "file")])
    try:
        MODULE.validate(archive, {"api-server", "open-iem-admin"})
    except ValueError as exc:
        assert "duplicate archive member" in str(exc)
    else:
        raise AssertionError("duplicate archive member accepted")


def test_unexpected_file_fails(tmp_path):
    archive = tmp_path / "unexpected.tar.gz"
    make_archive(archive, valid_members() + [("open-iem-server-1.2.3-aarch64-linux/secret", "file")])
    try:
        MODULE.validate(archive, {"api-server", "open-iem-admin"})
    except ValueError as exc:
        assert "unexpected archive member" in str(exc)
    else:
        raise AssertionError("unexpected archive member accepted")

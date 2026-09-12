import importlib.machinery
import importlib.util
import tarfile
import stat
from pathlib import Path
from types import SimpleNamespace
from typing import cast
from unittest.mock import MagicMock, patch


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


def test_member_count_limit_fails(tmp_path):
    archive = tmp_path / "too-many-members.tar.gz"
    root = "open-iem-server-1.2.3-aarch64-linux"
    make_archive(archive, valid_members() + [(f"{root}/README.md", "file")] * 30)
    try:
        MODULE.validate(archive, {"api-server", "open-iem-admin"})
    except ValueError as exc:
        assert "member count limit" in str(exc)
    else:
        raise AssertionError("archive exceeding member count accepted")


def test_compressed_size_limit_fails(tmp_path):
    archive = tmp_path / "too-large-compressed.tar.gz"
    archive.touch()
    metadata = SimpleNamespace(
        st_mode=stat.S_IFREG,
        st_size=MODULE.MAX_ARCHIVE_BYTES + 1,
    )
    archive_file = MagicMock()
    archive_file.fileno.return_value = 10
    with patch.object(MODULE.os, "open", return_value=10), patch.object(
        MODULE.os, "fstat", return_value=metadata
    ), patch.object(MODULE.os, "fdopen", return_value=archive_file):
        try:
            MODULE.validate(archive, {"api-server", "open-iem-admin"})
        except ValueError as exc:
            assert "compressed size limit" in str(exc)
        else:
            raise AssertionError("archive exceeding compressed size accepted")


def test_missing_nofollow_support_fails_closed(tmp_path):
    archive = tmp_path / "archive.tar.gz"
    archive.touch()
    with patch.object(MODULE.os, "O_NOFOLLOW", None, create=True):
        try:
            MODULE.validate(archive, {"api-server", "open-iem-admin"})
        except RuntimeError as exc:
            assert "fail-closed archive verification" in str(exc)
        else:
            raise AssertionError("archive validation accepted missing O_NOFOLLOW support")


def test_symlink_input_fails_closed(tmp_path):
    target = tmp_path / "target.tar.gz"
    target.write_bytes(b"not an archive")
    archive = tmp_path / "archive.tar.gz"
    archive.symlink_to(target)
    try:
        MODULE.validate(archive, {"api-server", "open-iem-admin"})
    except ValueError as exc:
        assert "readable regular file" in str(exc)
    else:
        raise AssertionError("symlink archive input accepted")


def test_directory_input_fails_closed(tmp_path):
    archive = tmp_path / "archive.tar.gz"
    archive.mkdir()
    try:
        MODULE.validate(archive, {"api-server", "open-iem-admin"})
    except ValueError as exc:
        assert "readable regular file" in str(exc)
    else:
        raise AssertionError("directory archive input accepted")


def test_total_uncompressed_size_limit_fails(tmp_path):
    archive = tmp_path / "too-large-uncompressed.tar.gz"
    archive.touch()
    members = [
        SimpleNamespace(name="open-iem-server-1.2.3-aarch64-linux/api-server", size=MODULE.MAX_MEMBER_BYTES),
        SimpleNamespace(name="open-iem-server-1.2.3-aarch64-linux/open-iem-admin", size=MODULE.MAX_MEMBER_BYTES),
        SimpleNamespace(name="open-iem-server-1.2.3-aarch64-linux/README.md", size=1),
    ]
    with patch.object(MODULE.tarfile, "open") as open_archive:
        open_archive.return_value.__enter__.return_value.__iter__.return_value = iter(members)
        try:
            MODULE.validate(archive, {"api-server", "open-iem-admin"})
        except ValueError as exc:
            assert "uncompressed size limit" in str(exc)
        else:
            raise AssertionError("archive exceeding uncompressed size accepted")


def test_member_size_limit_fails(tmp_path):
    archive = tmp_path / "too-large.tar.gz"
    archive.touch()
    oversized = SimpleNamespace(name="open-iem-server-1.2.3-aarch64-linux/api-server", size=MODULE.MAX_MEMBER_BYTES + 1)
    with patch.object(MODULE.tarfile, "open") as open_archive:
        open_archive.return_value.__enter__.return_value.__iter__.return_value = iter([oversized])
        try:
            MODULE.validate(archive, {"api-server", "open-iem-admin"})
        except ValueError as exc:
            assert "size limit" in str(exc)
        else:
            raise AssertionError("archive exceeding member size accepted")


def test_unknown_required_basename_fails_cli(tmp_path):
    archive = tmp_path / "valid.tar.gz"
    make_archive(archive, valid_members())
    result = __import__("subprocess").run(
        ["python3", "scripts/validate-release-archive.py", str(archive), "not-allowlisted"],
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 2
    assert "required basenames are not allowed" in result.stderr

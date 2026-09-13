import importlib.util
import json
from pathlib import Path


SCRIPT = Path(__file__).parents[1] / "scripts" / "validate-version.py"
spec = importlib.util.spec_from_file_location("validate_version", SCRIPT)
assert spec is not None and spec.loader is not None
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


def make_root(tmp_path, version="0.3.1"):
    (tmp_path / "VERSION").write_text(version + "\n", encoding="utf-8")
    (tmp_path / "server").mkdir()
    (tmp_path / "server" / "Cargo.toml").write_text(
        f'[workspace.package]\nversion = "{version}"\n', encoding="utf-8"
    )
    for directory in ("musician", "engineer"):
        path = tmp_path / "web" / directory
        path.mkdir(parents=True)
        (path / "package.json").write_text(json.dumps({"version": version}), encoding="utf-8")
    return tmp_path


def test_validates_canonical_version_and_tag(tmp_path):
    assert module.validate(make_root(tmp_path), "v0.3.1") == "0.3.1"


def test_rejects_manifest_drift(tmp_path):
    root = make_root(tmp_path)
    (root / "web" / "engineer" / "package.json").write_text('{"version":"0.3.2"}', encoding="utf-8")
    try:
        module.validate(root)
    except ValueError as exc:
        assert "every project manifest" in str(exc)
    else:
        raise AssertionError("manifest drift accepted")


def test_rejects_workspace_version_outside_workspace_package(tmp_path):
    root = make_root(tmp_path)
    (root / "server" / "Cargo.toml").write_text(
        '[workspace]\nmembers = []\n[package]\nversion = "0.3.1"\n', encoding="utf-8"
    )
    try:
        module.validate(root)
    except ValueError as exc:
        assert "workspace package version missing" in str(exc)
    else:
        raise AssertionError("invalid Cargo structure accepted")


def test_rejects_invalid_tag(tmp_path):
    try:
        module.validate(make_root(tmp_path), "0.3.1")
    except ValueError as exc:
        assert "exact vX.Y.Z" in str(exc)
    else:
        raise AssertionError("invalid tag accepted")

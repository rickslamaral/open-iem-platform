from pathlib import Path

def test_package_contract_files_exist():
    root = Path(__file__).parents[1]
    for path in (
        'packaging/deb/build-deb.sh', 'packaging/deb/DEBIAN/control.in',
        'packaging/deb/DEBIAN/postinst', 'packaging/deb/DEBIAN/prerm',
        'packaging/deb/DEBIAN/postrm', 'scripts/validate-deb-package.py',
    ):
        assert (root / path).is_file()

def test_release_evidence_terms_are_separated():
    text = (Path(__file__).parents[1] / 'docs/RELEASE-GATES.md').read_text()
    assert 'SOFTWARE_RELEASE_GATE' in text
    assert 'HARDWARE_CERTIFICATION' in text

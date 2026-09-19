#!/usr/bin/env python3
"""Validate Open IEM .deb structure and lifecycle metadata without installing it."""
from __future__ import annotations
import argparse, json, pathlib, subprocess, sys

REQUIRED = {
    "usr/lib/openiem/api-server", "usr/lib/openiem/open-iem-admin",
    "usr/lib/systemd/system/openiem-server.service",
    "usr/share/doc/openiem/README.Debian", "DEBIAN/control",
    "DEBIAN/postinst", "DEBIAN/prerm", "DEBIAN/postrm",
}

def main() -> int:
    p = argparse.ArgumentParser(); p.add_argument("package"); args = p.parse_args()
    pkg = pathlib.Path(args.package)
    if not pkg.is_file(): print(f"missing package: {pkg}", file=sys.stderr); return 2
    control = subprocess.check_output(["dpkg-deb", "-f", str(pkg)], text=True)
    fields = dict(line.split(": ", 1) for line in control.splitlines() if ": " in line)
    errors=[]
    if fields.get("Package") != "openiem": errors.append("Package must be openiem")
    if fields.get("Architecture") not in {"amd64", "arm64"}: errors.append("Architecture must be amd64 or arm64")
    files = subprocess.check_output(["dpkg-deb", "-c", str(pkg)], text=True)
    names={line.split()[-1].lstrip("./") for line in files.splitlines()}
    payload_required = REQUIRED - {"DEBIAN/control", "DEBIAN/postinst", "DEBIAN/prerm", "DEBIAN/postrm"}
    errors.extend(f"missing payload path: {x}" for x in sorted(payload_required - names))
    control_dir = pathlib.Path(subprocess.check_output(["mktemp", "-d"], text=True).strip())
    try:
        subprocess.run(["dpkg-deb", "--control", str(pkg), str(control_dir)], check=True, stdout=subprocess.DEVNULL)
        control_files = {p.name for p in control_dir.iterdir()}
        errors.extend(f"missing control script: {x}" for x in {"control", "postinst", "prerm", "postrm"} - control_files)
        for script in ("postinst", "prerm", "postrm"):
            path = control_dir / script
            if path.exists() and subprocess.run(["bash", "-n", str(path)], capture_output=True).returncode:
                errors.append(f"invalid shell syntax: {script}")
    finally:
        import shutil
        shutil.rmtree(control_dir, ignore_errors=True)
    if "password" in control.lower() or "secret" in control.lower(): errors.append("secret-like metadata")
    result={"package":str(pkg),"version":fields.get("Version"),"architecture":fields.get("Architecture"),"files":len(names),"status":"PASS" if not errors else "FAIL","errors":errors}
    print(json.dumps(result, indent=2))
    return 0 if not errors else 1
if __name__ == "__main__": raise SystemExit(main())

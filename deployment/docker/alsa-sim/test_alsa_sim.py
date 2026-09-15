"""
ALSA simulation smoke test — deployment/docker/alsa-sim/

Validates that the alsa-sim container can enumerate and play audio
against the host snd-dummy driver.

Pre-requisites on host:
  modprobe snd-dummy          (loads ALSA dummy card 0)
  docker build -t open-iem-alsa-sim ./deployment/docker/alsa-sim/

Run:
  pytest deployment/docker/alsa-sim/test_alsa_sim.py -v

The test skips automatically when:
  - /dev/snd is absent (no ALSA on host)
  - docker is not available
  - the image has not been built yet
"""

import subprocess
import shutil
import pathlib
import pytest


DEVICE = "null" if __import__("os").environ.get("ALSA_SIM_MODE") == "null" else "hw:0,0"
IMAGE = "open-iem-alsa-sim"
DEV_SND = pathlib.Path("/dev/snd")
SOFTWARE_ONLY = DEVICE == "null"


def _docker_args():
    args = ["docker", "run", "--rm"]
    if not SOFTWARE_ONLY:
        args += ["--device", "/dev/snd"]
    if SOFTWARE_ONLY:
        args += ["-e", "ALSA_SIM_MODE=null"]
    return args


def _skip_if_not_ready():
    if not SOFTWARE_ONLY and not DEV_SND.exists():
        pytest.skip("/dev/snd not present — load snd-dummy first: modprobe snd-dummy")
    if not shutil.which("docker"):
        pytest.skip("docker not found in PATH")
    result = subprocess.run(
        ["docker", "image", "inspect", IMAGE],
        capture_output=True,
    )
    if result.returncode != 0:
        pytest.skip(
            f"Image '{IMAGE}' not built. "
            f"Run: docker build -t {IMAGE} ./deployment/docker/alsa-sim/"
        )


def test_alsa_device_list():
    """Container must enumerate at least one ALSA playback device."""
    _skip_if_not_ready()
    result = subprocess.run(
        _docker_args() + [
            "--entrypoint", "aplay",
            IMAGE, "-L" if SOFTWARE_ONLY else "-l",
        ],
        capture_output=True,
        text=True,
        timeout=30,
    )
    assert result.returncode == 0, f"aplay -l failed:\n{result.stderr}"
    if SOFTWARE_ONLY:
        assert "null" in result.stdout.lower(), (
            f"No null PCM listed in aplay output:\n{result.stdout}"
        )
    else:
        assert "card" in result.stdout.lower(), (
            f"No card listed in aplay output:\n{result.stdout}"
        )


def test_alsa_playback_dummy():
    """Container must play the pre-generated test tone on hw:0,0 without error."""
    _skip_if_not_ready()
    result = subprocess.run(
        _docker_args() + [IMAGE, DEVICE],
        capture_output=True,
        text=True,
        timeout=60,
    )
    combined = result.stdout + result.stderr
    assert result.returncode == 0, (
        f"Playback failed on {DEVICE}.\nOutput:\n{combined}"
    )
    assert "PASS" in combined, f"Expected PASS marker in output:\n{combined}"


def test_hw_params_reported():
    """Container must dump HW params without crashing."""
    _skip_if_not_ready()
    # Run only the aplay --dump-hw-params step via custom entrypoint override
    result = subprocess.run(
        _docker_args() + [
            "--entrypoint", "aplay",
            IMAGE,
            "-D", DEVICE,
            "--dump-hw-params",
            "/test_tone.wav",
        ],
        capture_output=True,
        text=True,
        timeout=30,
    )
    combined = result.stdout + result.stderr
    # aplay exits non-zero after dumping params (it still tries to play) — that's OK.
    # We only care that HW Params block was printed.
    assert "HW Params" in combined, (
        f"Expected HW Params block in output:\n{combined}"
    )

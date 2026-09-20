#!/usr/bin/env python3
"""Signal process only while verified PID identity remains open."""
import os
import re
import signal
import platform
import sys
import ctypes


def send_signal(pidfd: int, sig: signal.Signals) -> None:
    if hasattr(os, "pidfd_send_signal"):
        os.pidfd_send_signal(pidfd, sig)
        return
    syscall_numbers = {"x86_64": 424, "aarch64": 424, "arm64": 424}
    number = syscall_numbers.get(platform.machine())
    if number is None:
        raise OSError("pidfd_send_signal syscall unavailable on architecture")
    libc = ctypes.CDLL(None, use_errno=True)
    result = libc.syscall(number, pidfd, int(sig), 0, 0)
    if result != 0:
        error = ctypes.get_errno()
        raise OSError(error, os.strerror(error))

def process_start_time(pid: int) -> str:
    with open(f"/proc/{pid}/stat", encoding="ascii") as handle:
        match = re.match(r"^\d+ \(.*\) [A-Z] (.*)$", handle.read(), re.DOTALL)
    if match is None:
        raise ValueError("invalid proc stat")
    return match.group(1).split()[18]

def main() -> int:
    if len(sys.argv) != 4 or sys.argv[3] not in {"TERM", "KILL"}:
        return 2
    try:
        pid, expected = int(sys.argv[1]), sys.argv[2]
    except ValueError:
        return 2
    pidfd = None
    try:
        if process_start_time(pid) != expected:
            return 1
        sig = getattr(signal, f"SIG{sys.argv[3]}")
        if not hasattr(os, "pidfd_open") or not hasattr(os, "pidfd_send_signal"):
            return 1
        pidfd = os.pidfd_open(pid)
        if process_start_time(pid) != expected:
            return 1
        send_signal(pidfd, sig)
    except (OSError, ValueError, IndexError):
        return 1
    finally:
        if pidfd is not None:
            os.close(pidfd)
    return 0

if __name__ == "__main__":
    raise SystemExit(main())

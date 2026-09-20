#!/usr/bin/env python3
"""Signal process only while verified PID identity remains open."""
import os
import signal
import sys

def process_start_time(pid: int) -> str:
    with open(f"/proc/{pid}/stat", encoding="ascii") as handle:
        rest = handle.read().rsplit(")", 1)[1].split()
    return rest[19]

def main() -> int:
    if len(sys.argv) != 4 or sys.argv[3] not in {"TERM", "KILL"}:
        return 2
    try:
        pid, expected = int(sys.argv[1]), sys.argv[2]
    except ValueError:
        return 2
    try:
        pidfd_open = os.pidfd_open
        pidfd_send_signal = os.pidfd_send_signal
    except AttributeError:
        return 2
    pidfd = None
    try:
        pidfd = pidfd_open(pid)
        if process_start_time(pid) != expected:
            return 1
        pidfd_send_signal(pidfd, getattr(signal, f"SIG{sys.argv[3]}"))
    except (OSError, ValueError, IndexError):
        return 1
    finally:
        if pidfd is not None:
            os.close(pidfd)
    return 0

if __name__ == "__main__":
    raise SystemExit(main())

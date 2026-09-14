#!/usr/bin/env python3
"""Validate checked-in RBJ peaking-EQ reference vectors with stdlib math."""

import math

SAMPLE_RATE = 48_000.0
CASES = (
    (100.0, 6.0, 0.707, (1.006480037107, -1.986808003509, 0.980498195648, -1.986808003509, 0.986978232755)),
    (1_000.0, -3.0, 1.0, (0.978977346094, -1.840157304773, 0.877058603523, -1.840157304773, 0.856035949617)),
    (12_000.0, 9.0, 2.5, (1.193568133102, 0.0, 0.593530469976, 0.0, 0.787098603079)),
    (20_000.0, -12.0, 0.5, (0.626038301479, 0.867052359030, 0.375147524296, 0.867052359030, 0.001185825775)),
)


def rbj_peaking(frequency_hz: float, gain_db: float, q: float) -> tuple[float, ...]:
    amplitude = 10 ** (gain_db / 40)
    omega = 2 * math.pi * frequency_hz / SAMPLE_RATE
    alpha = math.sin(omega) / (2 * q)
    denominator = 1 + alpha / amplitude
    return (
        (1 + alpha * amplitude) / denominator,
        (-2 * math.cos(omega)) / denominator,
        (1 - alpha * amplitude) / denominator,
        (-2 * math.cos(omega)) / denominator,
        (1 - alpha / amplitude) / denominator,
    )


for frequency, gain, q, expected in CASES:
    actual = rbj_peaking(frequency, gain, q)
    errors = [abs(got - want) for got, want in zip(actual, expected)]
    assert max(errors) < 5e-6, (frequency, gain, q, errors)

print(f"{len(CASES)} Biquad reference vectors passed")

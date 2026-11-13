"""Emit operation sequences and expected order from the selected interpreter."""

import random
import sys

assert sys.implementation.name == "cpython" and sys.hash_info.width == 64
assert sys.version_info >= (3, 12)
rng = random.Random(312)
edge_keys = [-1, -2, (1 << 63) - 1, -(1 << 63), 3, -4, 0, 8, 16]
for case in range(2000):
    values = [
        rng.choice([rng.randrange(-8192, 8192), rng.randrange(-(1 << 63), 1 << 63)])
        for _ in range(rng.randrange(2, 160))
    ]
    if case < 2:
        values = edge_keys if case == 0 else edge_keys[::-1]
        other = [-2, 3] if case == 0 else [-1, (1 << 63) - 1, -(1 << 63)]
        removed = [-1, 8, -(1 << 63)]
        inserted = [-(1 << 63), 24, -1]
    else:
        if case < 7:
            values = list(range(0, 600_000, 8))
        if case % 3 == 0:
            other = values[::40]
        elif case % 3 == 1:
            other = [value for value in values if rng.randrange(3) == 0]
        else:
            other = []
        removed = values[::5]
        inserted = values[::11]
    lhs = set(values)
    initial = list(lhs)
    for value in removed:
        lhs.discard(value)
    for value in inserted:
        lhs.add(value)
    rhs = set(other)
    difference = lhs - rhs
    for sequence in (values, other, initial, removed, inserted, lhs, difference):
        print(" ".join(map(str, sequence)))

# Breeze Collections

`brz-collections` provides a Rust hash set with optional CPython iteration
compatibility. Both builds use `std::collections::HashSet` for membership.

```rust
use brz_collections::BrzHashSet;

let requested: BrzHashSet<i64> = [2092, 2937, 237, 3814, 2040].into_iter().collect();
let found: BrzHashSet<i64> = [237].into_iter().collect();
let missing = &requested - &found;
let ids: Vec<i64> = missing.into_iter().collect();
```

## Build selection

The default build uses Rust's standard hash set iteration. Set the following
environment variable on the command that compiles the Replay application:

```sh
BRZ_COLLECTIONS_BACKEND=cpython cargo build --release
```

For a Docker multi-stage build, set it in the builder stage where Cargo runs.
An equivalent option is to enable the dependency's `cpython-set` Cargo feature.
An unset environment variable follows the feature. Explicit
`BRZ_COLLECTIONS_BACKEND=rust` requires the feature to be disabled and makes
Cargo reject a conflicting build. Cargo tracks changes to the environment
variable and recompiles the crate as needed. The resulting binary uses the
selected behavior throughout its lifetime.

## CPython iteration

The compatibility build targets 64-bit CPython and includes `CPythonHash` for
`i64`. Rust equality must agree with Python key equality. With matching
inputs and operation history, the supported paths reproduce Python's
`set(iterable)`, `add`, `discard`, `clear`, and set difference (`&a - &b`
in Rust), including the order exposed by borrowed and consuming iteration.
Rust `Clone` preserves the current layout. CPython has additional
construction paths for dicts, sets, and optimized literals with their own
layout behavior.

`BrzHashSet::with_capacity(n)` reserves Rust member storage. The CPython
logical layout still starts empty. Membership and equality use the Rust table.
The compatibility build maintains additional bucket metadata, including copies
of keys, on mutation and traverses it when iteration is requested.

The compatibility code derives its probing, resizing, and difference behavior
from [CPython 3.12.11](https://github.com/python/cpython/blob/v3.12.11/Objects/setobject.c).
The tested CPython versions are 3.12.11, 3.13.15, and 3.14.7.

## Verification

```sh
cargo test
cargo test --features cpython-set
BRZ_COLLECTIONS_BACKEND=cpython cargo test
BRZ_TEST_CPYTHON=/path/to/python3.12 cargo test --features cpython-set \
  --test cpython_oracle -- --ignored
```

The CPython test compares construction, deletion, reinsertion, and difference
against the selected interpreter for 2,000 cases.

The Breeze code is available under MIT or Apache-2.0. See
[LICENSE-CPYTHON](LICENSE-CPYTHON) for the upstream license.

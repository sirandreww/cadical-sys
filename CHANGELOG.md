# Changelog

## 0.8.0

### Breaking changes

- Connected callbacks (`connect_terminator`, `connect_learner`, `connect_fixed_listener`, `connect_external_propagator`, `connect_proof_tracer1`) take the callback by value (`T: 'static`) and are owned by the solver. Share an `Rc<RefCell<T>>` to inspect a callback while it is connected.
- `read_dimacs*`, `write_dimacs`, `write_extension` and `build` return `Result<(), String>`; functions taking names or paths take `&str`.
- Updated to `CaDiCaL` 3.0.1: proof and witness clause ids are `i64`, `ProofTracer::add_derived_clause` has a `witness` parameter, `State` has a new `INCONCLUSIVE` variant and new `READY`/`VALID` values.
- `force_backtrack` was removed (it could only abort when called from safe code); `reserve` is deprecated in favour of `resize`.
- The raw FFI module `cadical_sys::bridge` is private.
- `connect_proof_tracer1` takes a `finalize_clauses` argument; without it `ProofTracer::finalize_clause` was never called.
- Rust edition 2024; the minimum supported Rust version is 1.88 (required by `cxx`).

### Fixed

- Use-after-free when a connected callback was dropped or moved while the solver still used it.
- Double free of proof tracers when a solver was dropped with a tracer connected, or when a second tracer was connected.
- `exit(1)` inside the library when a file could not be opened; crash when a proof file could not be opened; `trace_proof1` writing to a closed file.
- Misuse that made `CaDiCaL` abort the process (invalid literals, calls in the wrong solver state, `clone` in the middle of a clause, observed variables without a propagator, ...) panics instead.

### Added

- `TerminateFlag`, a thread-safe terminator for stopping a `solve` from another thread.
- `propagate`, `implied`, `declare_more_variables`, `declare_one_more_variable`, `get_statistic_value`, `signature`.
- `ProofTracer` callbacks `demote_clause`, `report_status`, `begin_proof`, `solve_query`, `notify_equivalence` (with default implementations).

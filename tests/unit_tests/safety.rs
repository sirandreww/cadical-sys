//! Regression tests for memory safety and contract handling at the FFI boundary.
//!
//! Several of these only fail loudly (rather than silently corrupting memory)
//! when run under valgrind, which CI does in the `Valgrind` job.

use cadical_sys::{CaDiCal, ExternalPropagator, Learner, ProofTracer, Status, Terminator};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// Counts how many times it has been dropped.
struct DropCounter(Rc<Cell<usize>>);

impl Drop for DropCounter {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

struct CountingTerminator {
    calls: usize,
    _drops: DropCounter,
}

impl Terminator for CountingTerminator {
    fn terminated(&mut self) -> bool {
        self.calls += 1;
        false
    }
}

struct NoopLearner {
    _drops: DropCounter,
}

impl NoopLearner {
    fn new(drops: &Rc<Cell<usize>>) -> Self {
        Self {
            _drops: DropCounter(Rc::clone(drops)),
        }
    }
}

impl Learner for NoopLearner {
    fn learning(&mut self, _size: i32) -> bool {
        false
    }
    fn learn(&mut self, _lit: i32) {}
}

#[derive(Default)]
struct NoopTracer {
    original_clauses: usize,
}

impl ProofTracer for NoopTracer {
    fn add_original_clause(&mut self, _: i64, _: bool, _: &[i32], _: bool) {
        self.original_clauses += 1;
    }
    fn add_derived_clause(&mut self, _: i64, _: bool, _: i32, _: &[i32], _: &[i64]) {}
    fn delete_clause(&mut self, _: i64, _: bool, _: &[i32]) {}
    fn weaken_minus(&mut self, _: i64, _: &[i32]) {}
    fn strengthen(&mut self, _: i64) {}
    fn finalize_clause(&mut self, _: i64, _: &[i32]) {}
    fn add_assumption(&mut self, _: i32) {}
    fn add_constraint(&mut self, _: &[i32]) {}
    fn reset_assumptions(&mut self) {}
    fn add_assumption_clause(&mut self, _: i64, _: &[i32], _: &[i64]) {}
    fn conclude_sat(&mut self, _: i32, _: &[i32]) {}
    fn conclude_unsat(&mut self, _: i32, _: &[i64]) {}
    fn conclude_unknown(&mut self, _: &[i32]) {}
}

struct NoopPropagator;

impl ExternalPropagator for NoopPropagator {
    fn notify_assignment(&mut self, _: &[i32]) {}
    fn notify_new_decision_level(&mut self) {}
    fn notify_backtrack(&mut self, _: usize) {}
    fn cb_check_found_model(&mut self, _: &[i32]) -> bool {
        true
    }
    fn cb_has_external_clause(&mut self, _: &mut bool) -> bool {
        false
    }
    fn cb_add_external_clause_lit(&mut self) -> i32 {
        0
    }
}

/// A formula that needs search, so the terminator and learner get called.
fn add_pigeonhole(solver: &mut CaDiCal, holes: i32) {
    let pigeons = holes + 1;
    let var = |p: i32, h: i32| p * holes + h + 1;
    for p in 0..pigeons {
        let clause: Vec<i32> = (0..holes).map(|h| var(p, h)).collect();
        solver.clause6(&clause);
    }
    for h in 0..holes {
        for p in 0..pigeons {
            for q in p + 1..pigeons {
                solver.clause2(-var(p, h), -var(q, h));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Callback ownership
// ---------------------------------------------------------------------------

#[test]
fn callbacks_are_dropped_exactly_once_with_the_solver() {
    let drops = Rc::new(Cell::new(0));
    {
        let mut solver = CaDiCal::new();
        solver.connect_terminator(CountingTerminator {
            calls: 0,
            _drops: DropCounter(Rc::clone(&drops)),
        });
        solver.connect_learner(NoopLearner::new(&drops));
        assert_eq!(drops.get(), 0);
    }
    assert_eq!(drops.get(), 2);
}

#[test]
fn disconnect_drops_the_callback() {
    let drops = Rc::new(Cell::new(0));
    let mut solver = CaDiCal::new();
    solver.connect_learner(NoopLearner::new(&drops));
    solver.disconnect_learner();
    assert_eq!(drops.get(), 1);
    // Disconnecting again is a no-op.
    solver.disconnect_learner();
    assert_eq!(drops.get(), 1);
}

#[test]
fn replacing_a_callback_drops_the_old_one_and_keeps_solving() {
    let drops = Rc::new(Cell::new(0));
    let mut solver = CaDiCal::new();
    add_pigeonhole(&mut solver, 5);

    let second = Rc::new(RefCell::new(CountingTerminator {
        calls: 0,
        _drops: DropCounter(Rc::clone(&drops)),
    }));
    solver.connect_terminator(CountingTerminator {
        calls: 0,
        _drops: DropCounter(Rc::clone(&drops)),
    });
    solver.connect_terminator(Rc::clone(&second));
    assert_eq!(drops.get(), 1, "the first terminator should be dropped");

    assert_eq!(solver.solve(), Status::UNSATISFIABLE);
    assert!(
        second.borrow().calls > 0,
        "the replacement terminator should be called"
    );
}

#[test]
fn shared_callback_state_is_readable_while_connected() {
    let tracer = Rc::new(RefCell::new(NoopTracer::default()));
    let mut solver = CaDiCal::new();
    solver.connect_proof_tracer1(Rc::clone(&tracer), false);
    solver.clause2(1, 2);
    solver.clause2(-1, 2);
    assert_eq!(solver.solve(), Status::SATISFIABLE);
    assert_eq!(tracer.borrow().original_clauses, 2);
}

#[test]
fn replacing_a_proof_tracer_disconnects_the_old_one() {
    let first = Rc::new(RefCell::new(NoopTracer::default()));
    let second = Rc::new(RefCell::new(NoopTracer::default()));
    let mut solver = CaDiCal::new();
    solver.connect_proof_tracer1(Rc::clone(&first), false);
    solver.connect_proof_tracer1(Rc::clone(&second), false);
    assert_eq!(
        Rc::strong_count(&first),
        1,
        "solver should release the first tracer"
    );

    solver.clause2(1, 2);
    assert_eq!(solver.solve(), Status::SATISFIABLE);
    assert_eq!(first.borrow().original_clauses, 0);
    assert_eq!(second.borrow().original_clauses, 1);
}

#[test]
fn dropping_a_solver_with_a_connected_proof_tracer() {
    // CaDiCaL deletes tracers that are still connected when it is destroyed;
    // this used to double free the tracer.
    let tracer = Rc::new(RefCell::new(NoopTracer::default()));
    {
        let mut solver = CaDiCal::new();
        solver.connect_proof_tracer1(Rc::clone(&tracer), true);
        add_pigeonhole(&mut solver, 3);
        assert_eq!(solver.solve(), Status::UNSATISFIABLE);
    }
    assert_eq!(Rc::strong_count(&tracer), 1);
}

#[test]
fn clone_does_not_share_callbacks() {
    let learner = Rc::new(RefCell::new(NoopLearner::new(&Rc::default())));
    let mut solver = CaDiCal::new();
    solver.connect_learner(Rc::clone(&learner));
    solver.clause2(1, 2);
    let clone = solver.clone();
    drop(solver);
    assert_eq!(Rc::strong_count(&learner), 1);
    drop(clone);
}

// ---------------------------------------------------------------------------
// Contract violations panic instead of aborting the process
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "val requires the solver to be in state SATISFIED")]
fn val_before_solve_panics() {
    let mut solver = CaDiCal::new();
    solver.clause1(1);
    solver.val(1);
}

#[test]
#[should_panic(expected = "val requires the solver to be in state SATISFIED")]
fn val_after_unsat_panics() {
    let mut solver = CaDiCal::new();
    solver.clause1(1);
    solver.clause1(-1);
    assert_eq!(solver.solve(), Status::UNSATISFIABLE);
    solver.val(1);
}

#[test]
#[should_panic(expected = "invalid literal 0")]
fn val_of_zero_panics() {
    let mut solver = CaDiCal::new();
    solver.clause1(1);
    assert_eq!(solver.solve(), Status::SATISFIABLE);
    solver.val(0);
}

#[test]
#[should_panic(expected = "failed requires the solver to be in state UNSATISFIED")]
fn failed_after_sat_panics() {
    let mut solver = CaDiCal::new();
    solver.clause1(1);
    assert_eq!(solver.solve(), Status::SATISFIABLE);
    solver.failed(1);
}

#[test]
#[should_panic(expected = "flip requires the solver to be in state SATISFIED")]
fn flip_before_solve_panics() {
    let mut solver = CaDiCal::new();
    solver.flip(1);
}

#[test]
#[should_panic(expected = "not supported while an external propagator is connected")]
fn flippable_with_external_propagator_panics() {
    let mut solver = CaDiCal::new();
    solver.connect_external_propagator(NoopPropagator);
    solver.clause1(1);
    assert_eq!(solver.solve(), Status::SATISFIABLE);
    solver.flippable(1);
}

#[test]
#[should_panic(expected = "connect_proof_tracer1 requires the solver to be in state CONFIGURING")]
fn connecting_a_proof_tracer_after_adding_clauses_panics() {
    let mut solver = CaDiCal::new();
    solver.clause1(1);
    solver.connect_proof_tracer1(NoopTracer::default(), false);
}

#[test]
#[should_panic(expected = "add_observed_var requires a connected external propagator")]
fn observing_a_variable_without_a_propagator_panics() {
    let mut solver = CaDiCal::new();
    solver.add_observed_var(1);
}

#[test]
fn observing_a_variable_with_a_propagator() {
    let mut solver = CaDiCal::new();
    solver.connect_external_propagator(NoopPropagator);
    solver.add_observed_var(1);
    solver.remove_observed_var(1);
    solver.reset_observed_vars();
    solver.disconnect_external_propagator();
}

#[test]
#[should_panic(expected = "clause2 called with invalid literal 0")]
fn zero_inside_a_clause_panics() {
    CaDiCal::new().clause2(1, 0);
}

#[test]
#[should_panic(expected = "add called with invalid literal -2147483648")]
fn adding_int_min_panics() {
    CaDiCal::new().add(i32::MIN);
}

#[test]
#[should_panic(expected = "assume called with invalid literal 0")]
fn assuming_zero_panics() {
    CaDiCal::new().assume(0);
}

#[test]
fn zero_still_terminates_a_clause() {
    let mut solver = CaDiCal::new();
    solver.add(1);
    solver.add(0);
    assert_eq!(solver.solve(), Status::SATISFIABLE);
}

#[test]
fn guarded_functions_work_in_the_right_state() {
    let mut solver = CaDiCal::new();
    solver.clause2(1, 2);
    assert_eq!(solver.solve(), Status::SATISFIABLE);
    assert_ne!(solver.val(1), 0);
    let _ = solver.flippable(1);

    solver.assume(-1);
    solver.assume(-2);
    assert_eq!(solver.solve(), Status::UNSATISFIABLE);
    assert!(solver.failed(-1) || solver.failed(-2));
}

// ---------------------------------------------------------------------------
// File handling
// ---------------------------------------------------------------------------

fn temp_path(name: &str) -> String {
    std::env::temp_dir()
        .join(format!("cadical_sys_{}_{name}", std::process::id()))
        .to_str()
        .unwrap()
        .to_owned()
}

#[test]
fn read_dimacs_reports_success_as_ok() {
    let mut solver = CaDiCal::new();
    let mut vars = 0;
    assert_eq!(
        solver.read_dimacs1("./tests/problem.cnf", "problem", &mut vars, 0),
        Ok(())
    );
    assert!(vars > 0);
    // CaDiCaL only reads DIMACS right after initialization.
    let mut solver = CaDiCal::new();
    let mut vars = 0;
    assert_eq!(
        solver.read_dimacs2("./tests/problem.cnf", &mut vars, 0),
        Ok(())
    );
}

#[test]
fn read_dimacs_of_missing_file_is_an_error() {
    // This used to call `exit(1)` from inside the library.
    let mut solver = CaDiCal::new();
    let mut vars = 0;
    let missing = "./tests/does_not_exist.cnf";
    assert!(solver
        .read_dimacs1(missing, "missing", &mut vars, 0)
        .is_err());
    let mut solver = CaDiCal::new();
    assert!(solver.read_dimacs2(missing, &mut vars, 0).is_err());
    let (mut incremental, mut cubes) = (false, Vec::new());
    let mut solver = CaDiCal::new();
    assert!(solver
        .read_dimacs3(
            missing,
            "missing",
            &mut vars,
            0,
            &mut incremental,
            &mut cubes
        )
        .is_err());
    let mut solver = CaDiCal::new();
    assert!(solver
        .read_dimacs4(missing, &mut vars, 0, &mut incremental, &mut cubes)
        .is_err());
}

#[test]
fn write_dimacs_round_trips() {
    let path = temp_path("round_trip.cnf");
    let mut solver = CaDiCal::new();
    // No unit clauses: `write_dimacs` only writes units of frozen variables.
    solver.clause2(1, 2);
    solver.clause2(1, -2);
    assert_eq!(solver.write_dimacs(&path, 0), Ok(()));

    let mut reread = CaDiCal::new();
    let mut vars = 0;
    assert_eq!(reread.read_dimacs2(&path, &mut vars, 1), Ok(()));
    assert_eq!(reread.solve(), Status::SATISFIABLE);
    assert!(reread.val(1) > 0);
    std::fs::remove_file(path).unwrap();
}

#[test]
#[should_panic(expected = "read_dimacs2 requires the solver to be in state CONFIGURING")]
fn read_dimacs_after_adding_clauses_panics() {
    let mut solver = CaDiCal::new();
    solver.clause1(1);
    let mut vars = 0;
    let _ = solver.read_dimacs2("./tests/problem.cnf", &mut vars, 0);
}

#[test]
fn build_to_unwritable_path_is_an_error() {
    assert!(CaDiCal::build("./tests/no_such_dir/build.txt", "c ").is_err());
}

#[test]
fn trace_proof1_writes_the_whole_proof() {
    // `trace_proof1` used to close the file right after handing it to CaDiCaL,
    // which then kept writing the proof to the closed `FILE *`.
    let path = temp_path("proof.drat");
    {
        let mut solver = CaDiCal::new();
        assert!(solver.set("binary", 0));
        assert!(solver.trace_proof1(&path, "proof"));
        add_pigeonhole(&mut solver, 4);
        assert_eq!(solver.solve(), Status::UNSATISFIABLE);
        solver.close_proof_trace(false);
    }
    let proof = std::fs::read_to_string(&path).unwrap();
    assert!(
        proof.lines().any(|line| line.trim() == "0"),
        "proof should end with the empty clause"
    );
    std::fs::remove_file(path).unwrap();
}

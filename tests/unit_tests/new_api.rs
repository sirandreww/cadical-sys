//! Tests for the API added in `CaDiCaL` 3.

use cadical_sys::{CaDiCal, ProofTracer, Status};
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn declare_variables() {
    let mut solver = CaDiCal::new();
    assert_eq!(solver.declare_one_more_variable(), 1);
    assert_eq!(solver.declare_more_variables(3), 4);
    assert_eq!(solver.vars(), 4);
}

#[test]
fn propagate_reports_implied_literals() {
    let mut solver = CaDiCal::new();
    solver.clause2(-1, 2);
    solver.clause2(-2, 3);
    // Needs a decision, so propagation alone cannot satisfy the formula.
    solver.clause3(4, 5, 6);
    solver.assume(1);
    assert_eq!(solver.propagate(), Status::UNKNOWN);
    let implied = solver.implied();
    assert!(implied.contains(&2), "{implied:?}");
    assert!(implied.contains(&3), "{implied:?}");
}

#[test]
fn propagate_detects_conflicts() {
    let mut solver = CaDiCal::new();
    solver.clause1(1);
    solver.clause2(-1, 2);
    solver.assume(-2);
    assert_eq!(solver.propagate(), Status::UNSATISFIABLE);
}

#[test]
#[should_panic(expected = "implied requires the solver to be in one of the states")]
fn implied_before_propagate_panics() {
    let mut solver = CaDiCal::new();
    solver.clause1(1);
    let _ = solver.implied();
}

#[test]
fn statistics_can_be_queried_by_name() {
    let mut solver = CaDiCal::new();
    solver.clause2(1, 2);
    solver.clause2(-1, 2);
    assert_eq!(solver.solve(), Status::SATISFIABLE);
    assert!(solver.get_statistic_value("decisions") >= 0);
    assert_eq!(solver.get_statistic_value("no such statistic"), -1);
}

#[test]
fn signature_names_the_solver() {
    assert!(CaDiCal::signature().contains("cadical"));
}

#[derive(Default)]
struct StatusTracer {
    queries: usize,
    statuses: Vec<i32>,
}

impl ProofTracer for StatusTracer {
    fn add_original_clause(&mut self, _: i64, _: bool, _: &[i32], _: bool) {}
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

    fn solve_query(&mut self) {
        self.queries += 1;
    }
    fn report_status(&mut self, status: i32, _id: i64) {
        self.statuses.push(status);
    }
}

#[test]
fn tracer_is_told_about_each_solve_call() {
    let tracer = Rc::new(RefCell::new(StatusTracer::default()));
    let mut solver = CaDiCal::new();
    solver.connect_proof_tracer1(Rc::clone(&tracer), true);
    solver.clause2(1, 2);
    assert_eq!(solver.solve(), Status::SATISFIABLE);
    solver.clause1(-1);
    solver.clause1(-2);
    assert_eq!(solver.solve(), Status::UNSATISFIABLE);

    let tracer = tracer.borrow();
    assert_eq!(tracer.queries, 2);
    assert_eq!(tracer.statuses, [10, 20]);
}

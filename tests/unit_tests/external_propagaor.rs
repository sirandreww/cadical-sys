use cadical_sys::{CaDiCal, ExternalPropagator, Status};

struct MyExternalPropagator {}

impl ExternalPropagator for MyExternalPropagator {
    fn is_lazy(&mut self) -> bool {
        false
    }

    fn are_reasons_forgettable(&mut self) -> bool {
        false
    }

    fn notify_assignment(&mut self, _: &[i32]) {}
    fn notify_new_decision_level(&mut self) {}
    fn notify_backtrack(&mut self, _: usize) {
        todo!()
    }
    fn cb_check_found_model(&mut self, _: &[i32]) -> bool {
        true // sat
    }
    fn cb_has_external_clause(&mut self, _: &mut bool) -> bool {
        false
    }
    fn cb_add_external_clause_lit(&mut self) -> i32 {
        todo!()
    }
    fn cb_decide(&mut self) -> i32 {
        0
    }

    fn cb_propagate(&mut self) -> i32 {
        0
    }

    fn cb_add_reason_clause_lit(&mut self, _propagated_lit: i32) -> i32 {
        todo!()
    }
}

/// Adds SAT test CNF to solver
fn add_sat_test_cnf(solver: &mut CaDiCal) {
    solver.clause6(&[-1, 3]);
    solver.clause6(&[-2, 3]);
    solver.clause6(&[1, 2, -3]);
}

#[test]
fn test_external_propagator_connect_and_disconnect() {
    let mut solver = CaDiCal::new();
    let mut propagator = MyExternalPropagator {};
    solver.connect_external_propagator(&mut propagator);
    solver.disconnect_external_propagator();
}

#[test]
fn test_external_propagator_flow_disconnect_without_connecting() {
    let mut solver = CaDiCal::new();
    solver.disconnect_external_propagator();

    add_sat_test_cnf(&mut solver);
    let result = solver.solve();
    assert_eq!(result, Status::SATISFIABLE);
}

#[test]
fn test_external_propagator_flow_solve_after_connect_disconnect() {
    let mut solver = CaDiCal::new();
    let mut propagator = MyExternalPropagator {};
    solver.connect_external_propagator(&mut propagator);
    solver.disconnect_external_propagator();

    add_sat_test_cnf(&mut solver);
    let result = solver.solve();
    assert_eq!(result, Status::SATISFIABLE);
}

#[test]
fn test_external_propagator_solve_after_connect() {
    let mut solver = CaDiCal::new();
    let mut propagator = MyExternalPropagator {};

    solver.connect_external_propagator(&mut propagator);

    add_sat_test_cnf(&mut solver);
    let result = solver.solve();
    assert_eq!(result, Status::SATISFIABLE);
}

use cadical_sys::{CaDiCal, ExternalPropagator, FixedAssignmentListener, Status};

struct MyFixedListener {
    fixed_assignments: Vec<i32>,
}

impl MyFixedListener {
    fn new() -> Self {
        Self {
            fixed_assignments: Vec::new(),
        }
    }

    fn get_fixed_assignments(&self) -> &[i32] {
        &self.fixed_assignments
    }

    fn clear(&mut self) {
        self.fixed_assignments.clear();
    }
}

impl FixedAssignmentListener for MyFixedListener {
    fn notify_fixed_assignment(&mut self, lit: i32) {
        self.fixed_assignments.push(lit);
    }
}

/// Adds SAT test CNF to solver
fn add_sat_test_cnf(solver: &mut CaDiCal) {
    solver.clause6(&[-1, 3]);
    solver.clause6(&[-2, 3]);
    solver.clause6(&[1, 2, -3]);
}

/// Adds UNSAT test CNF to solver
fn add_unsat_test_cnf(solver: &mut CaDiCal) {
    solver.clause6(&[1]);
    solver.clause6(&[-1]);
}

/// Adds CNF with fixed assignments
fn add_fixed_assignment_cnf(solver: &mut CaDiCal) {
    // This CNF should result in some fixed assignments during solving
    solver.clause6(&[1, 2]);
    solver.clause6(&[1, -2]);
    solver.clause6(&[-1, 3]);
    solver.clause6(&[-1, -3]);
}

#[test]
fn test_fixed_listener_connect_and_disconnect() {
    let mut solver = CaDiCal::new();
    let mut fixed_listener = MyFixedListener::new();
    solver.connect_fixed_listener(&mut fixed_listener);
    solver.disconnect_fixed_listener();
}

#[test]
fn test_fixed_listener_flow_solve_after_connect_disconnect() {
    let mut solver = CaDiCal::new();
    let mut fixed_listener = MyFixedListener::new();
    solver.connect_fixed_listener(&mut fixed_listener);
    solver.disconnect_fixed_listener();

    add_sat_test_cnf(&mut solver);
    let result = solver.solve();
    assert_eq!(result, Status::SATISFIABLE);
}

#[test]
fn test_fixed_listener_solve_after_connect() {
    let mut solver = CaDiCal::new();
    let mut fixed_listener = MyFixedListener::new();

    solver.connect_fixed_listener(&mut fixed_listener);

    add_sat_test_cnf(&mut solver);
    let result = solver.solve();
    assert_eq!(result, Status::SATISFIABLE);
    let assignments = fixed_listener.get_fixed_assignments();
    assert!(
        assignments.is_empty(),
        "There should not be any fixed literals"
    );
}

#[test]
fn test_fixed_listener_receives_assignments() {
    let mut solver = CaDiCal::new();
    let mut fixed_listener = MyFixedListener::new();

    solver.connect_fixed_listener(&mut fixed_listener);
    add_fixed_assignment_cnf(&mut solver);

    let result = solver.solve();
    assert_eq!(result, Status::UNSATISFIABLE);

    // Check that the listener received some fixed assignments
    let assignments = fixed_listener.get_fixed_assignments();
    assert!(
        !assignments.is_empty(),
        "Fixed listener should receive assignments"
    );
}

#[test]
fn test_fixed_listener_unsat_case() {
    let mut solver = CaDiCal::new();
    let mut fixed_listener = MyFixedListener::new();

    solver.connect_fixed_listener(&mut fixed_listener);
    add_unsat_test_cnf(&mut solver);

    let result = solver.solve();
    assert_eq!(result, Status::UNSATISFIABLE);

    let assignments = fixed_listener.get_fixed_assignments();
    assert!(
        assignments.contains(&1),
        "Should have fixed assignment for literal 1"
    );
    assert!(
        !assignments.contains(&-1),
        "Cannot contain fixed literal -1 since that makes formula unsat"
    );
}

#[test]
fn test_fixed_listener_disconnect_prevents_notifications() {
    let mut solver = CaDiCal::new();
    let mut fixed_listener = MyFixedListener::new();

    solver.connect_fixed_listener(&mut fixed_listener);
    solver.disconnect_fixed_listener();

    add_fixed_assignment_cnf(&mut solver);
    let result = solver.solve();
    assert_eq!(result, Status::UNSATISFIABLE);

    // After disconnection, no assignments should be received
    let assignments = fixed_listener.get_fixed_assignments();
    assert!(
        assignments.is_empty(),
        "Disconnected listener should not receive assignments"
    );
}

#[test]
fn test_fixed_listener_reconnect() {
    let mut solver = CaDiCal::new();
    let mut fixed_listener = MyFixedListener::new();

    // Connect, disconnect, then reconnect
    solver.connect_fixed_listener(&mut fixed_listener);
    solver.disconnect_fixed_listener();
    solver.connect_fixed_listener(&mut fixed_listener);

    add_fixed_assignment_cnf(&mut solver);
    let result = solver.solve();
    assert_eq!(result, Status::UNSATISFIABLE);

    // Should receive assignments after reconnection
    let assignments = fixed_listener.get_fixed_assignments();
    assert!(
        !assignments.is_empty(),
        "Reconnected listener should receive assignments"
    );
}

#[test]
fn test_fixed_listener_empty_cnf() {
    let mut solver = CaDiCal::new();
    let mut fixed_listener = MyFixedListener::new();

    solver.connect_fixed_listener(&mut fixed_listener);

    // Solve with no clauses (empty CNF)
    let result = solver.solve();
    assert_eq!(result, Status::SATISFIABLE);

    // Empty CNF should not produce fixed assignments
    let assignments = fixed_listener.get_fixed_assignments();
    assert!(
        assignments.is_empty(),
        "Empty CNF should not produce fixed assignments"
    );
}

#[test]
fn test_fixed_listener_unit_clauses() {
    let mut solver = CaDiCal::new();
    let mut fixed_listener = MyFixedListener::new();

    solver.connect_fixed_listener(&mut fixed_listener);

    // Add unit clauses which should result in fixed assignments
    solver.clause6(&[1]);
    solver.clause6(&[-2]);

    let result = solver.solve();
    assert_eq!(result, Status::SATISFIABLE);

    let assignments = fixed_listener.get_fixed_assignments();
    assert!(
        !assignments.is_empty(),
        "Unit clauses should produce fixed assignments"
    );

    // Check that we have the expected assignments
    assert!(
        assignments.contains(&1),
        "Should have fixed assignment for literal 1"
    );
    assert!(
        assignments.contains(&-2),
        "Should have fixed assignment for literal -2"
    );
    assert!(
        assignments.len() == 2,
        "Should have exactly 2 fixed assignments"
    );
}

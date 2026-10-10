use cadical_sys::{CaDiCal, ExternalPropagator, Status};
use std::cell::RefCell;
use std::rc::Rc;

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
    solver.connect_external_propagator(MyExternalPropagator {});
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
    solver.connect_external_propagator(MyExternalPropagator {});
    solver.disconnect_external_propagator();

    add_sat_test_cnf(&mut solver);
    let result = solver.solve();
    assert_eq!(result, Status::SATISFIABLE);
}

#[test]
fn test_external_propagator_solve_after_connect() {
    let mut solver = CaDiCal::new();

    solver.connect_external_propagator(MyExternalPropagator {});

    add_sat_test_cnf(&mut solver);
    let result = solver.solve();
    assert_eq!(result, Status::SATISFIABLE);
}

/// Forces variable 2 to be true by adding the external clause `(2)`.
#[derive(Default)]
struct ForceTwo {
    /// Literals of the external clause still to be handed to the solver, in
    /// reverse order (popped from the back).
    pending: Vec<i32>,
    clause_added: bool,
    assignments: Vec<i32>,
    models_checked: usize,
}

impl ExternalPropagator for ForceTwo {
    fn notify_assignment(&mut self, lits: &[i32]) {
        self.assignments.extend_from_slice(lits);
    }
    fn notify_new_decision_level(&mut self) {}
    fn notify_backtrack(&mut self, _: usize) {}
    fn cb_check_found_model(&mut self, model: &[i32]) -> bool {
        self.models_checked += 1;
        model.contains(&2)
    }
    fn cb_has_external_clause(&mut self, is_forgettable: &mut bool) -> bool {
        if !self.clause_added {
            self.clause_added = true;
            self.pending = vec![0, 2];
            *is_forgettable = false;
        }
        !self.pending.is_empty()
    }
    fn cb_add_external_clause_lit(&mut self) -> i32 {
        self.pending.pop().expect("asked for more literals than announced")
    }
}

#[test]
fn test_external_propagator_adds_a_clause() {
    let propagator = Rc::new(RefCell::new(ForceTwo::default()));
    let mut solver = CaDiCal::new();
    solver.connect_external_propagator(Rc::clone(&propagator));
    solver.add_observed_var(1);
    solver.add_observed_var(2);
    solver.clause2(1, -2);

    assert_eq!(solver.solve(), Status::SATISFIABLE);
    assert!(solver.val(2) > 0, "the external clause forces 2");
    assert!(solver.val(1) > 0, "(1 or -2) and 2 force 1");

    let propagator = propagator.borrow();
    assert!(propagator.clause_added);
    assert!(propagator.pending.is_empty());
    assert!(propagator.models_checked > 0);
    assert!(!propagator.assignments.is_empty());
}

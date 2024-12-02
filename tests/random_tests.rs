use cadical_sys::{CaDiCal, ClauseIterator, Status};
use rand::Rng;

fn get_random_cnf<R: Rng>(rng: &mut R) -> Vec<Vec<i32>> {
    let n = rng.gen_range(1..10);
    let mut cnf = Vec::new();
    for _ in 0..n {
        let mut clause = Vec::new();
        let m = rng.gen_range(1..10);
        for _ in 0..m {
            let lit = rng.gen_range(1..10);
            if rng.gen_bool(0.5) {
                clause.push(lit);
            } else {
                clause.push(-lit);
            }
        }
        cnf.push(clause);
    }
    cnf
}

// valgrind --leak-check=full --show-leak-kinds=all --track-origins=yes --verbose --log-file=valgrind-out.txt cargo test --package cadical-sys --test random_tests -- random_test --exact --nocapture
#[test]
fn random_test() {
    let mut rng = rand::thread_rng();
    struct CI {
        v: Vec<Vec<i32>>,
    }

    impl ClauseIterator for CI {
        fn clause(&mut self, clause: &Vec<i32>) -> bool {
            self.v.push(clause.clone());
            true
        }
    }

    const ITERATIONS: usize = 10000;

    for i in 0..ITERATIONS {
        let seed: u64 = rng.gen();
        println!("i = {}\tseed = {}", i, seed);

        let cnf = get_random_cnf(&mut rng);

        // println!("Original CNF:");
        // for clause in cnf.iter() {
        //     println!("{:?}", clause);
        // }

        // make solver
        let mut solver = CaDiCal::new();

        for clause in &cnf {
            solver.clause6(clause);
        }

        let mut i1 = CI { v: Vec::new() };
        solver.traverse_clauses(&mut i1);

        solver.simplify(3);

        let mut i2 = CI { v: Vec::new() };
        solver.traverse_clauses(&mut i2);

        // println!("Simplified CNF:");
        // for clause in i2.v.iter() {
        //     println!("{:?}", clause);
        // }
    }
}

// Example usage of CaDiCal SAT solver

#[test]
/// Basic SAT solving example
fn basic_sat_solving() {
    // Create a new solver instance
    let mut solver = CaDiCal::new();

    // Add clauses (representing a simple propositional logic problem)
    // For example, (x1 OR x2) AND (NOT x1 OR x3) AND (NOT x2 OR NOT x3)
    solver.clause2(1, 2); // x1 OR x2
    solver.clause2(-1, 3); // NOT x1 OR x3
    solver.clause2(-2, -3); // NOT x2 OR NOT x3

    // Solve the problem
    let status = solver.solve();
    match status {
        Status::SATISFIABLE => {
            // Get variable assignments
            println!("x1: {}", solver.val(1));
            println!("x2: {}", solver.val(2));
            println!("x3: {}", solver.val(3));
        }
        Status::UNSATISFIABLE => println!("No solution exists"),
        Status::UNKNOWN => println!("Solution status unknown"),
    }
}

/// Advanced example with assumptions and configuration
#[test]
fn advanced_sat_solving() {
    let mut solver = CaDiCal::new();

    // Configure the solver
    solver.configure("plain".to_string());

    // Set some options
    solver.set("verbose".to_string(), 1);

    // Add complex clauses
    solver.clause3(1, 2, 3); // x1 OR x2 OR x3
    solver.clause3(-1, -2, -3); // NOT x1 OR NOT x2 OR NOT x3

    // Make assumptions
    solver.assume(1); // Assume x1 is true

    // Solve with assumptions
    let status = solver.solve();

    // Check solving results
    if status == Status::SATISFIABLE {
        // Interact with solved model
        let num_vars = solver.vars();
        for var in 1..=num_vars {
            println!("Variable {}: {}", var, solver.val(var));
        }
    }
}

/// Example of reading DIMACS file and solving
#[test]
fn dimacs_solving() {
    let mut solver = CaDiCal::new();
    let mut var_count = 0;

    // Read a DIMACS CNF file
    let result = solver.read_dimacs1(
        "./tests/problem.cnf".to_string(),
        "my_problem".to_string(),
        &mut var_count,
        0,
    );

    println!("Read DIMACS result: {:?}", result);

    // Solve the problem from the file
    let status = solver.solve();

    // Write out results or extension
    if status == Status::SATISFIABLE {
        solver.write_extension("/tmp/solution.ext".to_string());
    }
}

/// Demonstrating advanced solver interactions
#[test]
fn solver_management() {
    let mut solver = CaDiCal::new();

    // Reserve variable space
    solver.reserve(1000);

    // Add observed variables for tracking
    solver.add_observed_var(42);

    // Perform simplification
    let _ = solver.simplify(2);

    // Get solver statistics
    solver.statistics();
    solver.resources();
}

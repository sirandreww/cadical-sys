use cadical_sys::{CaDiCal, ClauseIterator, Learner, Status, Terminator, WitnessIterator};
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

struct CI {
    v: Vec<Vec<i32>>,
}

impl ClauseIterator for CI {
    fn clause(&mut self, clause: &[i32]) -> bool {
        self.v.push(clause.to_vec());
        true
    }
}

struct WI {
    v: Vec<(Vec<i32>, Vec<i32>, u64)>,
}

impl WitnessIterator for WI {
    fn witness(&mut self, clause: &[i32], witness: &[i32], id: u64) -> bool {
        self.v.push((clause.to_vec(), witness.to_vec(), id));
        true
    }
}

// valgrind --leak-check=full --show-leak-kinds=all --track-origins=yes --verbose --log-file=valgrind-out.txt cargo test --package cadical-sys --test random_tests -- random_test --exact --nocapture
#[test]
fn random_test() {
    const ITERATIONS: usize = 10000;

    let mut rng = rand::thread_rng();

    for i in 0..ITERATIONS {
        let seed: u64 = rng.gen();
        println!("i = {i}\tseed = {seed}");

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

    println!("Read DIMACS result: {result:?}");

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

// cargo valgrind test --package cadical-sys --test random_tests -- terminator_memory_test --exact --nocapture
#[test]
fn terminator_memory_test() {
    const ITERATIONS: usize = 100;
    struct Term {
        v: Vec<i32>,
    }

    impl Terminator for Term {
        fn terminated(&mut self) -> bool {
            self.v.push(self.v.last().unwrap() + 1);
            false
        }
    }

    struct L {
        v: Vec<i32>,
    }

    impl Learner for L {
        fn learn(&mut self, lit: i32) {
            self.v.push(lit);
        }

        fn learning(&mut self, size: i32) -> bool {
            self.v.push(size);
            false
        }
    }

    let mut rng = rand::thread_rng();

    for i in 0..ITERATIONS {
        let seed: u64 = rng.gen();
        println!("i = {i}\tseed = {seed}");

        let cnf = get_random_cnf(&mut rng);

        // make solver
        let mut solver = CaDiCal::new();

        for clause in &cnf {
            solver.clause6(clause);
        }

        let mut t = Term { v: vec![7; 1000] };
        let t_ref = &mut t;
        solver.connect_terminator(t_ref);

        let mut l = L { v: Vec::new() };
        let l_ref = &mut l;
        solver.connect_learner(l_ref);

        solver.simplify(3);

        if i % 2 == 0 {
            solver.disconnect_terminator();
        }
    }
}

#[test]
fn frozen_and_simplify_test() {
    let mut solver = CaDiCal::new();

    solver.clause6(&[3, 4, 5]);
    solver.clause6(&[-3, 4, -5]);
    solver.clause6(&[3, -4, -5]);
    solver.clause6(&[-3, -4, -5]);

    println!("Clauses in Solver");
    let mut it = CI { v: Vec::new() };
    solver.traverse_clauses(&mut it);
    for c in it.v.iter() {
        println!("{:?}", c);
    }

    println!("Is frozen: {}", solver.frozen(5));
    solver.freeze(3);
    solver.freeze(4);
    solver.freeze(5);

    println!("Is frozen: {}", solver.frozen(5));
    println!("Clauses after freeze");
    it.v.clear();
    solver.traverse_clauses(&mut it);
    for c in it.v.iter() {
        println!("{:?}", c);
    }

    let r = solver.simplify(3);
    println!("Simplify result: {:?}", r);

    if true {
        println!("Clauses after simplify");
        it.v.clear();
        solver.traverse_clauses(&mut it);
        for c in it.v.iter() {
            println!("{:?}", c);
        }

        println!("Witness");
        let mut it = WI { v: Vec::new() };
        solver.traverse_witnesses_forward(&mut it);
        for c in it.v.iter() {
            println!("{:?}", c);
        }

        for l in [3, 4, 5] {
            println!("Value of {} is {}", l, solver.fixed(l));
        }
    }
}

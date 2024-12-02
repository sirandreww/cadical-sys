use std::{cell::RefCell, rc::Rc, sync::Arc};

use cadical_sys::{CaDiCal, ClauseIterator};
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

#[test]
fn random_test() {
    let mut rng = rand::thread_rng();

    const ITERATIONS: usize = 1000;

    for i in 0..ITERATIONS {
        let seed: u64 = rng.gen();
        println!("i = {}\tseed = {}", i, seed);

        let cnf = get_random_cnf(&mut rng);

        // make solver
        let mut solver = CaDiCal::new();

        for clause in &cnf {
            solver.clause6(clause);
        }

        solver.simplify(3);

        struct CI {
            v: Vec<Vec<i32>>,
        }

        impl ClauseIterator for CI {
            fn clause(&mut self, clause: &Vec<i32>) -> bool {
                self.v.push(clause.clone());
                true
            }
        }

        let mut i = CI { v: Vec::new() };

        solver.traverse_clauses(&mut i);

        for clause in i.v.iter() {
            println!("{:?}", clause);
        }
    }
}

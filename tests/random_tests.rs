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
        let mut solver = cadical_sys::constructor();

        for clause in &cnf {
            cadical_sys::clause6(&mut solver, clause);
        }

        cadical_sys::simplify(&mut solver, 3);

        let mut v = vec![];

        fn clause_iter(clause: &Vec<i32>) {
            v.push(clause.clone());
        }

        cadical_sys::new_clause_iterator(clause_iter);

        for clause in &v {
            println!("{:?}", clause);
        }
    }
}

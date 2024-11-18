fn get_random_cnf<R: Rng>(rng: &mut R, n: usize) -> Vec<Vec<i32>> {
    let mut cnf = Vec::new();
    for _ in 0..n {
        let mut clause = Vec::new();
        let m = rng.gen_range(1, 10);
        for _ in 0..m {
            let lit = rng.gen_range(1, 10);
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
    let cnf = get_random_cnf(&mut rng, 10);

    let mut solver = 
}

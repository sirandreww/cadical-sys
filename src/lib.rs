#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[test]
fn simple_test() {
    unsafe {
        let mut solver = CaDiCaL::Solver::new();

        // 3 = 1 && 2
        solver.add(1);
        solver.add(-3);
        solver.add(0);

        solver.add(2);
        solver.add(-3);
        solver.add(0);

        solver.add(3);
        solver.add(-1);
        solver.add(-2);
        solver.add(0);

        assert_eq!(solver.solve(), true);
    }
}

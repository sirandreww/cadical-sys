#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("cadical-sys/src/cadical_bridge.hpp");

        type Solver;

        // Constructor and basic operations
        fn create_solver() -> UniquePtr<Solver>;
        fn add_literal(solver: &mut UniquePtr<Solver>, literal: i32);
        fn add_clause_end(solver: &mut UniquePtr<Solver>);
        fn solve_solver(solver: &mut UniquePtr<Solver>) -> i32;

        // Advanced clause addition
        fn add_clause(solver: &mut UniquePtr<Solver>, literals: &CxxVector<i32>);
        fn add_clause_with_assumption(solver: &mut UniquePtr<Solver>, assumption: i32);

        // State checking
        fn is_solver_inconsistent(solver: &UniquePtr<Solver>) -> bool;
        fn get_value(solver: &UniquePtr<Solver>, lit: i32) -> i32;
        fn is_failed(solver: &UniquePtr<Solver>, lit: i32) -> bool;

        // Configuration and limits
        fn set_option(solver: &mut UniquePtr<Solver>, name: &CxxString, val: i32) -> bool;
        fn get_option(solver: &UniquePtr<Solver>, name: &CxxString) -> i32;
        fn optimize(solver: &mut UniquePtr<Solver>, val: i32);
        fn set_limit(solver: &mut UniquePtr<Solver>, name: &CxxString, val: i32) -> bool;

        // Statistics and information
        fn get_active_variables(solver: &UniquePtr<Solver>) -> i32;
        fn get_redundant_clauses(solver: &UniquePtr<Solver>) -> i64;
        fn get_irredundant_clauses(solver: &UniquePtr<Solver>) -> i64;
        fn print_statistics(solver: &UniquePtr<Solver>);
        fn print_resources(solver: &UniquePtr<Solver>);

        // Termination and cleanup
        fn terminate_solver(solver: &mut UniquePtr<Solver>);
        fn conclude_solver(solver: &mut UniquePtr<Solver>);
    }
}
#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("cadical-sys/src/cadical_bridge.hpp");

        type Solver;
        type Status;
        type State;
        type ClauseIterator;
        type WitnessIterator;
        type Terminator;
        type Learner;
        type FixedAssignmentListener;
        type ExternalPropagator;
        type Tracer;
        type InternalTracer;
        type StatTracer;
        type FileTracer;

        // Constructor and basic operations
        fn constructor() -> UniquePtr<Solver>;

        fn signature() -> String;

        fn add(solver: &mut UniquePtr<Solver>, literal: i32);

        fn clause1(solver: &mut UniquePtr<Solver>, l1: i32);
        fn clause2(solver: &mut UniquePtr<Solver>, l1: i32, l2: i32);
        fn clause3(solver: &mut UniquePtr<Solver>, l1: i32, l2: i32, l3: i32);
        fn clause4(solver: &mut UniquePtr<Solver>, l1: i32, l2: i32, l3: i32, l4: i32);
        fn clause5(solver: &mut UniquePtr<Solver>, l1: i32, l2: i32, l3: i32, l4: i32, l5: i32);
        fn clause6(solver: &mut UniquePtr<Solver>, v: &Vec<i32>);
        unsafe fn clause7(solver: &mut UniquePtr<Solver>, ptr: *const i32, n: usize);

        fn inconsistent(solver: &mut UniquePtr<Solver>) -> bool;

        fn assume(solver: &mut UniquePtr<Solver>, lit: i32);

        fn solve(solver: &mut UniquePtr<Solver>) -> i32;

        fn val(solver: &mut UniquePtr<Solver>, lit: i32) -> i32;

        fn flip(solver: &mut UniquePtr<Solver>, lit: i32) -> bool;

        fn flippable(solver: &mut UniquePtr<Solver>, lit: i32) -> bool;

        fn failed(solver: &mut UniquePtr<Solver>, lit: i32) -> bool;

        fn connect_terminator(
            solver: &mut UniquePtr<Solver>,
            terminator: &mut UniquePtr<Terminator>,
        );

        fn disconnect_terminator(solver: &mut UniquePtr<Solver>);

        fn connect_learner(solver: &mut UniquePtr<Solver>, learner: &mut UniquePtr<Learner>);

        fn disconnect_learner(solver: &mut UniquePtr<Solver>);

        fn connect_fixed_listener(
            solver: &mut UniquePtr<Solver>,
            fixed_listener: &mut UniquePtr<FixedAssignmentListener>,
        );

        fn disconnect_fixed_listener(solver: &mut UniquePtr<Solver>);

        fn connect_external_propagator(
            solver: &mut UniquePtr<Solver>,
            propagator: &mut UniquePtr<ExternalPropagator>,
        );
        fn disconnect_external_propagator(solver: &mut UniquePtr<Solver>);

        fn add_observed_var(solver: &mut UniquePtr<Solver>, var: i32);

        fn remove_observed_var(solver: &mut UniquePtr<Solver>, var: i32);

        fn reset_observed_vars(solver: &mut UniquePtr<Solver>);

        fn is_decision(solver: &mut UniquePtr<Solver>, lit: i32) -> bool;

        fn force_backtrack(solver: &mut UniquePtr<Solver>, new_level: usize);

        fn constrain(solver: &mut UniquePtr<Solver>, lit: i32);

        fn constraint_failed(solver: &mut UniquePtr<Solver>) -> bool;

        fn lookahead(solver: &mut UniquePtr<Solver>) -> i32;

        fn generate_cubes(
            solver: &mut UniquePtr<Solver>,
            x: i32,
            min_depth: i32,
            result_cubes: &mut Vec<i32>,
        ) -> i32;

        fn reset_assumptions(solver: &mut UniquePtr<Solver>);

        fn reset_constraint(solver: &mut UniquePtr<Solver>);

        fn state(solver: &UniquePtr<Solver>) -> &State;

        fn status(solver: &UniquePtr<Solver>) -> i32;

        fn version() -> String;

        fn copy(source: &UniquePtr<Solver>, destination: &mut UniquePtr<Solver>);

        fn vars(solver: &mut UniquePtr<Solver>) -> i32;

        fn reserve(solver: &mut UniquePtr<Solver>, min_max_var: i32);

        // fn trace_api_calls(solver: &mut UniquePtr<Solver>, file: String);

        fn is_preprocessing_option(name: String) -> bool;

        fn is_valid_long_option(arg: String) -> bool;

        fn get(solver: &mut UniquePtr<Solver>, name: String) -> i32;

        fn prefix(solver: &mut UniquePtr<Solver>, verbose_message_prefix: String);

        fn set(solver: &mut UniquePtr<Solver>, name: String, val: i32) -> bool;

        fn set_long_option(solver: &mut UniquePtr<Solver>, arg: String) -> bool;

        fn is_valid_configuration(name: String) -> bool;

        fn configure(solver: &mut UniquePtr<Solver>, name: String) -> bool;

        fn optimize(solver: &mut UniquePtr<Solver>, val: i32);

        fn limit(solver: &mut UniquePtr<Solver>, arg: String, val: i32) -> bool;

        fn is_valid_limit(solver: &mut UniquePtr<Solver>, arg: String) -> bool;

        fn active(solver: &UniquePtr<Solver>) -> i32;

        fn redundant(solver: &UniquePtr<Solver>) -> i64;

        fn irredundant(solver: &UniquePtr<Solver>) -> i64;

        fn simplify(solver: &mut UniquePtr<Solver>, rounds: i32) -> i32;

        fn terminate(solver: &mut UniquePtr<Solver>);

        fn frozen(solver: &UniquePtr<Solver>, lit: i32) -> bool;

        fn freeze(solver: &mut UniquePtr<Solver>, lit: i32);

        fn melt(solver: &mut UniquePtr<Solver>, lit: i32);

        fn fixed(solver: &UniquePtr<Solver>, lit: i32) -> i32;

        fn phase(solver: &mut UniquePtr<Solver>, lit: i32);

        fn unphase(solver: &mut UniquePtr<Solver>, lit: i32);

        fn trace_proof1(solver: &mut UniquePtr<Solver>, file: String, name: String) -> bool;

        fn trace_proof2(solver: &mut UniquePtr<Solver>, path: String) -> bool;

        fn flush_proof_trace(solver: &mut UniquePtr<Solver>, print: bool);

        fn close_proof_trace(solver: &mut UniquePtr<Solver>, print: bool);

        fn connect_proof_tracer1(
            solver: &mut UniquePtr<Solver>,
            tracer: &mut UniquePtr<Tracer>,
            antecedents: bool,
        );

        fn connect_proof_tracer2(
            solver: &mut UniquePtr<Solver>,
            tracer: &mut UniquePtr<InternalTracer>,
            antecedents: bool,
        );

        fn connect_proof_tracer3(
            solver: &mut UniquePtr<Solver>,
            tracer: &mut UniquePtr<StatTracer>,
            antecedents: bool,
        );

        fn connect_proof_tracer4(
            solver: &mut UniquePtr<Solver>,
            tracer: &mut UniquePtr<FileTracer>,
            antecedents: bool,
        );

        fn conclude(solver: &mut UniquePtr<Solver>);

        fn disconnect_proof_tracer1(
            solver: &mut UniquePtr<Solver>,
            tracer: &mut UniquePtr<Tracer>,
        ) -> bool;

        fn disconnect_proof_tracer2(
            solver: &mut UniquePtr<Solver>,
            tracer: &mut UniquePtr<StatTracer>,
        ) -> bool;

        fn disconnect_proof_tracer3(
            solver: &mut UniquePtr<Solver>,
            tracer: &mut UniquePtr<FileTracer>,
        ) -> bool;

        fn usage();

        fn configurations();

        fn statistics(solver: &mut UniquePtr<Solver>);

        fn resources(solver: &mut UniquePtr<Solver>);

        fn options(solver: &mut UniquePtr<Solver>);

        fn traverse_clauses(solver: &UniquePtr<Solver>, i: &mut UniquePtr<ClauseIterator>) -> bool;

        fn traverse_witnesses_backward(
            solver: &UniquePtr<Solver>,
            i: &mut UniquePtr<WitnessIterator>,
        ) -> bool;

        fn traverse_witnesses_forward(
            solver: &UniquePtr<Solver>,
            i: &mut UniquePtr<WitnessIterator>,
        ) -> bool;

        fn read_dimacs1(
            solver: &mut UniquePtr<Solver>,
            file: String,
            name: String,
            vars: &mut i32,
            strict: i32,
        ) -> String;

        fn read_dimacs2(
            solver: &mut UniquePtr<Solver>,
            path: String,
            vars: &mut i32,
            strict: i32,
        ) -> String;

        fn read_dimacs3(
            solver: &mut UniquePtr<Solver>,
            file: String,
            name: String,
            vars: &mut i32,
            strict: i32,
            incremental: &mut bool,
            cubes: &mut Vec<i32>,
        ) -> String;

        fn read_dimacs4(
            solver: &mut UniquePtr<Solver>,
            path: String,
            vars: &mut i32,
            strict: i32,
            incremental: &mut bool,
            cubes: &mut Vec<i32>,
        ) -> String;

        fn write_dimacs(solver: &mut UniquePtr<Solver>, path: String, min_max_var: i32) -> String;

        fn write_extension(solver: &mut UniquePtr<Solver>, path: String) -> String;

        fn build(file: String, prefix: String);

    }
}

//! Rust bindings for the `CaDiCaL` SAT Solver, providing low-level access to one of the most efficient Boolean Satisfiability (SAT) solving libraries.
//!
//! # Overview
//!
//! `cadical-sys` offers complete Rust bindings to the `CaDiCaL` SAT solver using the `cxx` crate, enabling seamless interoperability between Rust and C++ SAT solving capabilities.
//!
//! ## What is a SAT Solver?
//!
//! A SAT (Boolean Satisfiability) solver is a computational tool that determines whether there exists an assignment of boolean variables that makes a given boolean formula true. SAT solvers are crucial in:
//! - Formal verification
//! - Hardware design
//! - AI planning
//! - Cryptanalysis
//! - Constraint solving
//!
//! ## About `CaDiCaL`
//!
//! [CaDiCaL](https://github.com/arminbiere/cadical) is a state-of-the-art, modern SAT solver developed by Armin Biere. Known for its:
//! - High performance
//! - Extensive features
//! - Compact implementation
//! - Advanced conflict-driven clause learning (CDCL) techniques
//!
//! # Features
//!
//! - Complete binding of `CaDiCaL` C++ API
//! - Safe Rust wrappers using `cxx` (where possible)
//! - Support for:
//!   - Adding clauses
//!   - Solving boolean satisfiability problems
//!   - Assumption handling
//!   - Advanced solver configuration
//!   - Proof tracing
//!   - Incremental solving
//!
//! # Installation
//!
//! Add to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! cadical-sys = "0.1.0"  # Replace with most recent version
//! ```
//!
//! # Usage Examples
//!
//! ## Basic SAT solving example
//! ```rust
//!    use cadical_sys::Status;
//!    use cadical_sys::CaDiCal;
//!
//!    // Create a new solver instance
//!    let mut solver = CaDiCal::new();
//!
//!    // Add clauses (representing a simple propositional logic problem)
//!    // For example, (x1 OR x2) AND (NOT x1 OR x3) AND (NOT x2 OR NOT x3)
//!    solver.clause2(1, 2);    // x1 OR x2
//!    solver.clause2(-1, 3);   // NOT x1 OR x3
//!    solver.clause2(-2, -3);  // NOT x2 OR NOT x3
//!
//!    // Solve the problem
//!    let status = solver.solve();
//!    match status {
//!        Status::SATISFIABLE => {
//!            // Get variable assignments
//!            println!("x1: {}", solver.val(1));
//!            println!("x2: {}", solver.val(2));
//!            println!("x3: {}", solver.val(3));
//!        },
//!        Status::UNSATISFIABLE => println!("No solution exists"),
//!        Status::UNKNOWN => println!("Solution status unknown")
//!    }
//! ```
//!
//! ## Advanced example with assumptions and configuration
//! ```rust
//!    use cadical_sys::Status;
//!    use cadical_sys::CaDiCal;
//!    
//!    let mut solver = CaDiCal::new();
//!
//!    // Configure the solver
//!    solver.configure("plain".to_string());
//!
//!    // Set some options
//!    solver.set("verbose".to_string(), 1);
//!
//!    // Add complex clauses
//!    solver.clause3(1, 2, 3);  // x1 OR x2 OR x3
//!    solver.clause3(-1, -2, -3);  // NOT x1 OR NOT x2 OR NOT x3
//!
//!    // Make assumptions
//!    solver.assume(1);  // Assume x1 is true
//!
//!    // Solve with assumptions
//!    let status = solver.solve();
//!
//!    // Check solving results
//!    if status == Status::SATISFIABLE {
//!        // Interact with solved model
//!        let num_vars = solver.vars();
//!        for var in 1..=num_vars {
//!            println!("Variable {}: {}", var, solver.val(var));
//!        }
//!    }
//! ```
//!
//! ## Example of reading DIMACS file and solving
//! ```rust
//!    use cadical_sys::Status;
//!    use cadical_sys::CaDiCal;
//!
//!    let mut solver = CaDiCal::new();
//!    let mut var_count = 0;
//!
//!    // Read a DIMACS CNF file
//!    let result = solver.read_dimacs1(
//!        "./tests/problem.cnf".to_string(),
//!        "my_problem".to_string(),
//!        &mut var_count,
//!        0
//!    );
//!
//!    // Solve the problem from the file
//!    let status = solver.solve();
//!
//!    // Write out results or extension
//!    if status == Status::SATISFIABLE {
//!        solver.write_extension("/tmp/solution.ext".to_string());
//!    }
//! ```
//!
//! ## Demonstrating advanced solver interactions
//! ```rust
//!    use cadical_sys::CaDiCal;
//!
//!    let mut solver = CaDiCal::new();
//!
//!    // Reserve variable space
//!    solver.reserve(1000);
//!
//!    // Add observed variables for tracking
//!    solver.add_observed_var(42);
//!
//!    // Perform simplification
//!    let simplify_status = solver.simplify(2);
//!
//!    // Get solver statistics
//!    solver.statistics();
//!    solver.resources();
//! ```
//!
//! # Performance Considerations
//!
//! - `CaDiCaL` is highly optimized for complex boolean satisfiability problems
//! - Recommended for problems with thousands to millions of variables
//! - Lower overhead compared to many other SAT solvers
//!
//! # Limitations
//!
//! - Requires understanding of boolean logic and SAT solving
//! - Performance depends on problem complexity
//! - Advanced features require deep knowledge of SAT solving techniques
//!
//! # Contributing
//!
//! Contributions are welcome! Please file issues or submit pull requests on the GitHub repository.
//!
//! # License
//!
//! `CaDiCaL` is distributed under the MIT License. Check the original repository for detailed licensing information.
//!
//! # References
//!
//! - [CaDiCaL GitHub Repository](https://github.com/arminbiere/cadical)
//! - [cxx Rust Bindings](https://cxx.rs/)
//! - [SAT Solver Overview](https://en.wikipedia.org/wiki/Boolean_satisfiability_problem)
//!
//! # Acknowledgments
//!
//! Special thanks to Armin Biere for developing and maintaining `CaDiCaL`.

use bridge::ffi;
use cxx::UniquePtr;

/// This module contains the FFI bindings to the `CaDiCaL` SAT solver.
/// Some functions are unsafe due to necessity.
pub mod bridge;

/// The SAT competition standardized the exit code of SAT solvers to the
/// following which then is also used return code for 'solve' functions.
/// In the following example we use those constants for brevity though.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Status {
    SATISFIABLE = 10,
    UNSATISFIABLE = 20,
    UNKNOWN = 0,
}

impl From<i32> for Status {
    fn from(val: i32) -> Self {
        match val {
            10 => Status::SATISFIABLE,
            20 => Status::UNSATISFIABLE,
            0 => Status::UNKNOWN,
            _ => unreachable!(),
        }
    }
}

/// States are represented by a bit-set in order to combine them.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum State {
    /// during initialization (invalid)
    INITIALIZING = 1,
    /// configure options (with 'set')
    CONFIGURING = 2,
    /// ready to call 'solve'
    STEADY = 4,
    /// adding clause literals (zero missing)
    ADDING = 8,
    /// while solving (within 'solve')
    SOLVING = 16,
    /// satisfiable allows 'val'
    SATISFIED = 32,
    /// unsatisfiable allows 'failed'
    UNSATISFIED = 64,
    /// during and after deletion (invalid)
    DELETING = 128,

    /// These combined states are used to check contracts.
    /// CONFIGURING | STEADY | SATISFIED | UNSATISFIED,
    READY = 102,
    /// READY | ADDING
    VALID = 110,
    /// INITIALIZING | DELETING
    INVALID = 129,
}

impl From<i32> for State {
    fn from(val: i32) -> Self {
        match val {
            1 => State::INITIALIZING,
            2 => State::CONFIGURING,
            4 => State::STEADY,
            8 => State::ADDING,
            16 => State::SOLVING,
            32 => State::SATISFIED,
            64 => State::UNSATISFIED,
            128 => State::DELETING,
            102 => State::READY,
            110 => State::VALID,
            129 => State::INVALID,
            _ => unreachable!(),
        }
    }
}

pub struct CaDiCal {
    solver: UniquePtr<ffi::Solver>,
    last_terminator: Option<UniquePtr<ffi::Terminator>>,
    last_learner: Option<UniquePtr<ffi::Learner>>,
    last_external_propagator: Option<UniquePtr<ffi::ExternalPropagator>>,
}

impl Clone for CaDiCal {
    fn clone(&self) -> Self {
        let mut r = Self::new();
        Self::copy(self, &mut r);
        r
    }
}

impl Default for CaDiCal {
    fn default() -> Self {
        Self::new()
    }
}

impl CaDiCal {
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self {
            solver: ffi::constructor(),
            last_terminator: None,
            last_learner: None,
            last_external_propagator: None,
        }
    }

    /// Core functionality as in the IPASIR incremental SAT solver interface.
    /// (recall 'READY = CONFIGURING | STEADY  | SATISFIED | UNSATISFIED').
    /// Further note that 'lit' is required to be different from '`INT_MIN`' and
    /// different from '0' except for 'add'.
    ///
    /// Add valid literal to clause or zero to terminate clause.
    ///
    ///   require (VALID)                   recall 'VALID = READY | ADDING'
    ///
    ///   if (lit) ensure (ADDING)          and thus VALID but not READY
    ///
    ///   if (!lit) ensure (STEADY )        and thus READY
    ///
    #[inline]
    pub fn add(&mut self, lit: i32) {
        ffi::add(&mut self.solver, lit);
    }

    /// Here are functions simplifying clause addition. The given literals
    /// should all be valid (different from '`INT_MIN`' and different from '0').
    ///
    ///   require (VALID)
    ///   ensure (STEADY )
    ///
    #[inline]
    pub fn clause1(&mut self, l1: i32) {
        ffi::clause1(&mut self.solver, l1);
    }

    #[inline]
    pub fn clause2(&mut self, l1: i32, l2: i32) {
        ffi::clause2(&mut self.solver, l1, l2);
    }

    #[inline]
    pub fn clause3(&mut self, l1: i32, l2: i32, l3: i32) {
        ffi::clause3(&mut self.solver, l1, l2, l3);
    }

    #[inline]
    pub fn clause4(&mut self, l1: i32, l2: i32, l3: i32, l4: i32) {
        ffi::clause4(&mut self.solver, l1, l2, l3, l4);
    }

    #[inline]
    pub fn clause5(&mut self, l1: i32, l2: i32, l3: i32, l4: i32, l5: i32) {
        ffi::clause5(&mut self.solver, l1, l2, l3, l4, l5);
    }

    #[inline]
    pub fn clause6(&mut self, v: &[i32]) {
        ffi::clause6(&mut self.solver, v);
    }

    /// This function can be used to check if the formula is already
    /// inconsistent (contains the empty clause or was proven to be
    /// root-level unsatisfiable).
    #[inline]
    pub fn inconsistent(&mut self) -> bool {
        ffi::inconsistent(&mut self.solver)
    }

    /// Assume valid non zero literal for next call to 'solve'.  These
    /// assumptions are reset after the call to 'solve' as well as after
    /// returning from 'simplify' and 'lookahead.
    ///
    ///   require (READY)
    ///   ensure (STEADY )
    ///
    #[inline]
    pub fn assume(&mut self, lit: i32) {
        ffi::assume(&mut self.solver, lit);
    }

    /// Try to solve the current formula.  Returns
    ///
    ///    0 = UNKNOWN      (limit reached or interrupted through 'terminate')
    ///   10 = SATISFIABLE
    ///   20 = UNSATISFIABLE
    ///
    ///   require (READY)
    ///   ensure (STEADY  | SATISFIED | UNSATISFIED)
    ///
    /// Note, that while in this call the solver actually transitions to state
    /// 'SOLVING', which however is only visible from a different context,
    /// i.e., from a different thread or from a signal handler.  Only right
    /// before returning from this call it goes into a 'READY' state.
    ///
    #[inline]
    pub fn solve(&mut self) -> Status {
        ffi::solve(&mut self.solver).into()
    }

    /// Get value (-lit=false, lit=true) of valid non-zero literal.
    ///
    ///   require (SATISFIED)
    ///   ensure (SATISFIED)
    ///
    #[inline]
    pub fn val(&mut self, lit: i32) -> i32 {
        ffi::val(&mut self.solver, lit)
    }

    /// Try to flip the value of the given literal without falsifying the
    /// formula.  Returns 'true' if this was successful. Otherwise the model is
    /// not changed and 'false' is returned.  If a literal was eliminated or
    /// substituted flipping will fail on that literal and in particular the
    /// solver will not taint it nor restore any clauses.
    ///
    /// The 'flip' function can only flip the value of a variables not acting
    /// as witness on the reconstruction stack.
    ///
    /// As a side effect of calling this function first all assigned variables
    /// are propagated again without using blocking literal.  Thus the first
    /// call to this function after obtaining a model adds a substantial
    /// overhead.  Subsequent calls will not need to properly propagate again.
    ///
    /// Furthermore if the reconstruction stack is non-empty and has been
    /// traversed to reconstruct a full extended model for eliminated
    /// variables (and to satisfy removed blocked clauses), the values of these
    /// witness variables obtained via 'val' before become invalid. The user
    /// thus will need to call 'val' again after calling 'flip' which will
    /// trigger then a traversal of the reconstruction stack.
    ///
    /// So try to avoid mixing 'flip' and 'val' (for efficiency only).
    /// Further, this functionality is currently not supported in the presence
    /// of an external propagator.
    ///
    ///   require (SATISFIED)
    ///   ensure (SATISFIED)
    ///
    #[inline]
    pub fn flip(&mut self, lit: i32) -> bool {
        ffi::flip(&mut self.solver, lit)
    }

    /// Same as 'flip' without actually flipping it. This functionality is
    /// currently not supported in the presence of an external propagator.
    ///
    ///   require (SATISFIED)
    ///   ensure (SATISFIED)
    ///
    #[inline]
    pub fn flippable(&mut self, lit: i32) -> bool {
        ffi::flippable(&mut self.solver, lit)
    }

    /// Determine whether the valid non-zero literal is in the core.
    /// Returns 'true' if the literal is in the core and 'false' otherwise.
    /// Note that the core does not have to be minimal.
    ///
    ///   require (UNSATISFIED)
    ///   ensure (UNSATISFIED)
    ///
    #[inline]
    pub fn failed(&mut self, lit: i32) -> bool {
        ffi::failed(&mut self.solver, lit)
    }

    /// Add call-back which is checked regularly for termination.  There can
    /// only be one terminator connected.  If a second (non-zero) one is added
    /// the first one is implicitly disconnected.
    ///
    ///   require (VALID)
    ///   ensure (VALID)
    ///
    #[inline]
    #[allow(clippy::missing_panics_doc)]
    pub fn connect_terminator<'a, 'b: 'a, T: Terminator>(&'a mut self, terminator: &'b mut T) {
        fn f<T: Terminator>(state: *mut u8) -> bool {
            let ptr: *mut T = state.cast::<T>();
            let i = unsafe { &mut *ptr };
            i.terminated()
        }
        let terminator =
            unsafe { ffi::new_terminator(std::ptr::from_mut(terminator).cast::<u8>(), f::<T>) };
        self.last_terminator = Some(terminator);
        ffi::connect_terminator(&mut self.solver, self.last_terminator.as_mut().unwrap());
    }

    #[inline]
    pub fn disconnect_terminator(&mut self) {
        ffi::disconnect_terminator(&mut self.solver);
        self.last_terminator = None;
    }

    /// Add call-back which allows to export learned clauses.
    ///
    ///   require (VALID)
    ///   ensure (VALID)
    ///
    #[inline]
    #[allow(clippy::missing_panics_doc)]
    pub fn connect_learner<'a, 'b: 'a, T: Learner>(&'a mut self, learner: &'b mut T) {
        fn learning<T: Learner>(state: *mut u8, size: i32) -> bool {
            let ptr: *mut T = state.cast::<T>();
            let i = unsafe { &mut *ptr };
            i.learning(size)
        }

        fn learn<T: Learner>(state: *mut u8, lit: i32) {
            let ptr: *mut T = state.cast::<T>();
            let i = unsafe { &mut *ptr };
            i.learn(lit);
        }

        let learner = unsafe {
            ffi::new_learner(
                std::ptr::from_mut(learner).cast::<u8>(),
                learning::<T>,
                learn::<T>,
            )
        };
        self.last_learner = Some(learner);
        ffi::connect_learner(&mut self.solver, self.last_learner.as_mut().unwrap());
    }

    #[inline]
    pub fn disconnect_learner(&mut self) {
        ffi::disconnect_learner(&mut self.solver);
        self.last_learner = None;
    }

    // /// Add call-back which allows to observe when a variable is fixed.
    // ///
    // ///   require (VALID)
    // ///   ensure (VALID)
    // ///
    // pub fn connect_fixed_listener<F: FixedAssignmentListener>(&mut self, _fixed_listener: F) {
    //     todo!()
    // }
    #[inline]
    pub fn disconnect_fixed_listener(&mut self) {
        ffi::disconnect_fixed_listener(&mut self.solver);
    }

    /// Add call-back which allows to learn, propagate and backtrack based on
    /// external constraints. Only one external propagator can be connected
    /// and after connection every related variables must be 'observed' (use
    /// 'add_observed_var' function).
    /// Disconnection of the external propagator resets all the observed
    /// variables.
    ///
    ///   require (VALID)
    ///   ensure (VALID)
    ///
    pub fn connect_external_propagator<'a, 'b: 'a, T: ExternalPropagator>(
        &'a mut self,
        propagator: &'b mut T,
    ) {
        // prepare propagation functions
        // bool is_lazy,
        let is_lazy = propagator.is_lazy();
        // bool are_reasons_forgettable,
        let are_reasons_forgettable = propagator.are_reasons_forgettable();
        // rust::Fn<void(uint8_t *, const rust::Slice<const int32_t>)> notify_assignment,
        fn notify_assignment<T: ExternalPropagator>(state: *mut u8, x: &[i32]) {
            let ptr: *mut T = state.cast::<T>();
            let i = unsafe { &mut *ptr };
            i.notify_assignment(x)
        }

        fn notify_new_decision_level<T: ExternalPropagator>(state: *mut u8) {
            let ptr: *mut T = state.cast::<T>();
            let i = unsafe { &mut *ptr };
            i.notify_new_decision_level()
        }
        fn notify_backtrack<T: ExternalPropagator>(state: *mut u8, x: usize) {
            let ptr: *mut T = state.cast::<T>();
            let i = unsafe { &mut *ptr };
            i.notify_backtrack(x)
        }
        fn cb_check_found_model<T: ExternalPropagator>(state: *mut u8, x: &[i32]) -> bool {
            let ptr: *mut T = state.cast::<T>();
            let i = unsafe { &mut *ptr };
            i.cb_check_found_model(x)
        }
        fn cb_decide<T: ExternalPropagator>(state: *mut u8) -> i32 {
            let ptr: *mut T = state.cast::<T>();
            let i = unsafe { &mut *ptr };
            i.cb_decide()
        }
        fn cb_propagate<T: ExternalPropagator>(state: *mut u8) -> i32 {
            let ptr: *mut T = state.cast::<T>();
            let i = unsafe { &mut *ptr };
            i.cb_propagate()
        }
        fn cb_add_reason_clause_lit<T: ExternalPropagator>(state: *mut u8, x: i32) -> i32 {
            let ptr: *mut T = state.cast::<T>();
            let i = unsafe { &mut *ptr };
            i.cb_add_reason_clause_lit(x)
        }
        fn cb_has_external_clause<T: ExternalPropagator>(state: *mut u8, x: *mut bool) -> bool {
            let ptr: *mut T = state.cast::<T>();
            let i = unsafe { &mut *ptr };
            i.cb_has_external_clause(unsafe { &mut *x })
        }
        fn cb_add_external_clause_lit<T: ExternalPropagator>(state: *mut u8) -> i32 {
            let ptr: *mut T = state.cast::<T>();
            let i = unsafe { &mut *ptr };
            i.cb_add_external_clause_lit()
        }

        let external_propagator = unsafe {
            ffi::new_external_propagator(
                std::ptr::from_mut(propagator).cast::<u8>(),
                is_lazy,
                are_reasons_forgettable,
                notify_assignment::<T>,
                notify_new_decision_level::<T>,
                notify_backtrack::<T>,
                cb_check_found_model::<T>,
                cb_decide::<T>,
                cb_propagate::<T>,
                cb_add_reason_clause_lit::<T>,
                cb_has_external_clause::<T>,
                cb_add_external_clause_lit::<T>,
            )
        };

        self.last_external_propagator = Some(external_propagator);
        ffi::connect_external_propagator(
            &mut self.solver,
            self.last_external_propagator.as_mut().unwrap(),
        );
    }
    pub fn disconnect_external_propagator(&mut self) {
        ffi::disconnect_external_propagator(&mut self.solver);
        self.last_external_propagator = None;
    }

    /// Mark as 'observed' those variables that are relevant to the external
    /// propagator. External propagation, clause addition during search and
    /// notifications are all over these observed variabes.
    /// A variable can not be observed witouth having an external propagator
    /// connected. Observed variables are "frozen" internally, and so
    /// inprocessing will not consider them as candidates for elimination.
    /// An observed variable is allowed to be a fresh variable and it can be
    /// added also during solving.
    ///
    ///   require (`VALID_OR_SOLVING`)
    ///   ensure (`VALID_OR_SOLVING`)
    ///
    #[inline]
    pub fn add_observed_var(&mut self, var: i32) {
        ffi::add_observed_var(&mut self.solver, var);
    }

    /// Removes the 'observed' flag from the given variable. A variable can be
    /// set unobserved only between solve calls, not during it (to guarantee
    /// that no yet unexplained external propagation involves it).
    ///
    ///   require (VALID)
    ///   ensure (VALID)
    ///
    #[inline]
    pub fn remove_observed_var(&mut self, var: i32) {
        ffi::remove_observed_var(&mut self.solver, var);
    }

    /// Removes all the 'observed' flags from the variables. Disconnecting the
    /// propagator invokes this step as well.
    ///
    ///   require (VALID)
    ///   ensure (VALID)
    ///
    #[inline]
    pub fn reset_observed_vars(&mut self) {
        ffi::reset_observed_vars(&mut self.solver);
    }

    /// Get reason of valid observed literal (true = it is an observed variable
    /// and it got assigned by a decision during the CDCL loop. Otherwise:
    /// false.
    ///
    ///   require (`VALID_OR_SOLVING`)
    ///   ensure (`VALID_OR_SOLVING`)
    ///
    #[inline]
    pub fn is_decision(&mut self, lit: i32) -> bool {
        ffi::is_decision(&mut self.solver, lit)
    }

    /// Force solve to backtrack to certain decision level. Can be called only
    /// during '`cb_decide`' of a connected External Propagator.
    /// Invoking in any other time will not have an effect.
    /// If the call had an effect, the External Propagator will be notified about
    /// the backtrack via '`notify_backtrack`'.
    ///
    ///   require (SOLVING)
    ///   ensure (SOLVING)
    ///
    #[inline]
    pub fn force_backtrack(&mut self, new_level: usize) {
        ffi::force_backtrack(&mut self.solver, new_level);
    }

    //------------------------------------------------------------------------
    /// Adds a literal to the constraint clause. Same functionality as 'add'
    /// but the clause only exists for the next call to solve (same lifetime as
    /// assumptions). Only one constraint may exists at a time. A new
    /// constraint replaces the old. The main application of this functonality
    /// is the model checking algorithm IC3. See our FMCAD'21 paper
    /// [FroleyksBiere-FMCAD'19] for more details.
    ///
    /// Add valid literal to the constraint clause or zero to terminate it.
    ///
    ///   require (VALID)                     /// recall 'VALID = READY |
    ///   ADDING' if (lit) ensure (ADDING)            /// and thus VALID but not
    ///   READY if (!lit) && !`adding_clause` ensure (STEADY ) // and thus READY
    ///
    #[inline]
    pub fn constrain(&mut self, lit: i32) {
        ffi::constrain(&mut self.solver, lit);
    }

    /// Determine whether the constraint was used to proof the
    /// unsatisfiability. Note that the formula might still be unsatisfiable
    /// without the constraint.
    ///
    ///   require (UNSATISFIED)
    ///   ensure (UNSATISFIED)
    ///
    #[inline]
    pub fn constraint_failed(&mut self) -> bool {
        ffi::constraint_failed(&mut self.solver)
    }

    //------------------------------------------------------------------------
    /// This function determines a good splitting literal.  The result can be
    /// zero if the formula is proven to be satisfiable or unsatisfiable.  This
    /// can then be checked by 'state ()'.  If the formula is empty and
    /// the function is not able to determine satisfiability also zero is
    /// returned but the state remains steady.
    ///
    ///   require (READY)
    ///   ensure (STEADY |SATISFIED|UNSATISFIED)
    ///
    #[inline]
    pub fn lookahead(&mut self) -> i32 {
        ffi::lookahead(&mut self.solver)
    }

    #[inline]
    pub fn generate_cubes(&mut self, x: i32, min_depth: i32, result_cubes: &mut Vec<i32>) -> i32 {
        ffi::generate_cubes(&mut self.solver, x, min_depth, result_cubes)
    }

    #[inline]
    pub fn reset_assumptions(&mut self) {
        ffi::reset_assumptions(&mut self.solver);
    }

    #[inline]
    pub fn reset_constraint(&mut self) {
        ffi::reset_constraint(&mut self.solver);
    }

    /// Return the current state of the solver as defined above.
    ///
    #[must_use]
    #[inline]
    pub fn state(&self) -> State {
        ffi::state(&self.solver).into()
    }

    /// Similar to 'state ()' but using the staddard competition exit codes of
    /// '10' for 'SATISFIABLE', '20' for 'UNSATISFIABLE' and '0' otherwise.
    ///
    #[must_use]
    #[inline]
    pub fn status(&self) -> Status {
        ffi::status(&self.solver).into()
    }

    /// return version string
    #[must_use]
    #[inline]
    pub fn version() -> String {
        ffi::version()
    }

    /*----------------------------------------------------------------------*/
    /// Copy 'this' into a fresh 'other'.  The copy procedure is not a deep
    /// clone, but only copies irredundant clauses and units.  It also makes
    /// sure that witness reconstruction works with the copy as with the
    /// original formula such that both solvers have the same models.
    /// Assumptions are not copied.  Options however are copied as well as
    /// flags which remember the current state of variables in preprocessing.
    ///
    ///   require (READY)          /// for 'this'
    ///   ensure (READY)           /// for 'this'
    ///
    ///   other.require (CONFIGURING)
    ///   other.ensure (CONFIGURING | STEADY )
    ///
    #[inline]
    pub fn copy(source: &CaDiCal, destination: &mut CaDiCal) {
        ffi::copy(&source.solver, &mut destination.solver);
    }

    /*----------------------------------------------------------------------*/
    /// Variables are usually added and initialized implicitly whenever a
    /// literal is used as an argument except for the functions 'val', 'fixed',
    /// 'failed' and 'frozen'.  However, the library internally keeps a maximum
    /// variable index, which can be queried.
    ///
    ///   require (VALID | SOLVING)
    ///   ensure (VALID | SOLVING)
    ///
    #[inline]
    pub fn vars(&mut self) -> i32 {
        ffi::vars(&mut self.solver)
    }

    /// Increase the maximum variable index explicitly.  This function makes
    /// sure that at least '`min_max_var`' variables are initialized.  Since it
    /// might need to reallocate tables, it destroys a satisfying assignment
    /// and has the same state transition and conditions as 'assume' etc.
    ///
    ///   require (READY)
    ///   ensure (STEADY )
    ///
    #[inline]
    pub fn reserve(&mut self, min_max_var: i32) {
        ffi::reserve(&mut self.solver, min_max_var);
    }

    // pub fn `trace_api_calls(&mut` self, file: String);

    //------------------------------------------------------------------------
    /// Option handling.
    /// Determine whether 'name' is a valid option name.
    ///
    #[must_use]
    #[inline]
    pub fn is_valid_option(name: String) -> bool {
        ffi::is_valid_option(name)
    }

    /// Determine whether 'name' enables a specific preprocessing technique.
    ///
    #[must_use]
    #[inline]
    pub fn is_preprocessing_option(name: String) -> bool {
        ffi::is_preprocessing_option(name)
    }

    /// Determine whether 'arg' is a valid long option of the form '--<name>',
    /// '--<name>=<val>' or '--no-<name>' similar to '`set_long_option`' below.
    /// Legal values are 'true', 'false', or '[-]<mantissa>[e<exponent>]'.
    #[must_use]
    #[inline]
    pub fn is_valid_long_option(arg: String) -> bool {
        ffi::is_valid_long_option(arg)
    }

    /// Get the current value of the option 'name'.  If 'name' is invalid then
    /// zero is returned.  Here '--...' arguments as invalid options.
    ///
    #[inline]
    pub fn get(&mut self, name: String) -> i32 {
        ffi::get(&mut self.solver, name)
    }

    /// Set the default verbose message prefix (default "c ").
    ///
    #[inline]
    pub fn prefix(&mut self, verbose_message_prefix: String) {
        ffi::prefix(&mut self.solver, verbose_message_prefix);
    }

    /// Explicit version of setting an option.  If the option '<name>' exists
    /// and '<val>' can be parsed then 'true' is returned.  If the option value
    /// is out of range the actual value is computed as the closest (minimum or
    /// maximum) value possible, but still 'true' is returned.
    ///
    ///   require (CONFIGURING)
    ///   ensure (CONFIGURING)
    ///
    /// Thus options can only bet set right after initialization.
    ///
    #[inline]
    pub fn set(&mut self, name: String, val: i32) -> bool {
        ffi::set(&mut self.solver, name, val)
    }

    /// This function accepts options in command line syntax:
    ///
    ///   '--<name>=<val>', '--<name>', or '--no-<name>'
    ///
    /// It actually calls the previous 'set' function after parsing 'arg'.  The
    /// same values are expected as for '`is_valid_long_option`' above and as
    /// with 'set' any value outside of the range of legal values for a
    /// particular option are set to either the minimum or maximum depending on
    /// which side of the valid interval they lie.
    ///
    ///   require (CONFIGURING)
    ///   ensure (CONFIGURING)
    ///
    #[inline]
    pub fn set_long_option(&mut self, arg: String) -> bool {
        ffi::set_long_option(&mut self.solver, arg)
    }

    /// Determine whether 'name' is a valid configuration.
    ///
    #[must_use]
    #[inline]
    pub fn is_valid_configuration(name: String) -> bool {
        ffi::is_valid_configuration(name)
    }

    /// Overwrite (some) options with the forced values of the configuration.
    /// The result is 'true' iff the 'name' is a valid configuration.
    ///
    ///   require (CONFIGURING)
    ///   ensure (CONFIGURING)
    ///
    #[inline]
    pub fn configure(&mut self, name: String) -> bool {
        ffi::configure(&mut self.solver, name)
    }

    /// Increase preprocessing and inprocessing limits by '10^<val>'.  Values
    /// below '0' are ignored and values above '9' are reduced to '9'.
    ///
    ///   require (READY)
    ///   ensure (READY)
    ///
    #[inline]
    pub fn optimize(&mut self, val: i32) {
        ffi::optimize(&mut self.solver, val);
    }

    /// Specify search limits, where currently 'name' can be "conflicts",
    /// "decisions", "preprocessing", or "localsearch".  The first two limits
    /// are unbounded by default.  Thus using a negative limit for conflicts or
    /// decisions switches back to the default of unlimited search (for that
    /// particular limit).  The preprocessing limit determines the number of
    /// preprocessing rounds, which is zero by default.  Similarly, the local
    /// search limit determines the number of local search rounds (also zero by
    /// default).  As with 'set', the return value denotes whether the limit
    /// 'name' is valid.  These limits are only valid for the next 'solve' or
    /// 'simplify' call and reset to their default after 'solve' returns (as
    /// well as overwritten and reset during calls to 'simplify' and
    /// 'lookahead').  We actually also have an internal "terminate" limit
    /// which however should only be used for testing and debugging.
    ///
    ///   require (READY)
    ///   ensure (READY)
    ///
    #[inline]
    pub fn limit(&mut self, arg: String, val: i32) -> bool {
        ffi::limit(&mut self.solver, arg, val)
    }
    #[inline]
    pub fn is_valid_limit(&mut self, arg: String) -> bool {
        ffi::is_valid_limit(&mut self.solver, arg)
    }

    /// The number of currently active variables and clauses can be queried by
    /// these functions.  Variables become active if a clause is added with it.
    /// They become inactive if they are eliminated or fixed at the root level
    /// Clauses become inactive if they are satisfied, subsumed, eliminated.
    /// Redundant clauses are reduced regularly and thus the 'redundant'
    /// function is less useful.
    ///
    ///   require (VALID)
    ///   ensure (VALID)
    ///
    /// Number of active variables.
    #[must_use]
    #[inline]
    pub fn active(&self) -> i32 {
        ffi::active(&self.solver)
    }

    /// Number of active redundant clauses.
    #[must_use]
    #[inline]
    pub fn redundant(&self) -> i64 {
        ffi::redundant(&self.solver)
    }

    /// Number of active irredundant clauses.
    #[must_use]
    #[inline]
    pub fn irredundant(&self) -> i64 {
        ffi::irredundant(&self.solver)
    }

    //------------------------------------------------------------------------
    /// This function executes the given number of preprocessing rounds. It is
    /// similar to 'solve' with 'limits ("preprocessing", rounds)' except that
    /// no CDCL nor local search, nor lucky phases are executed.  The result
    /// values are also the same: 0=UNKNOWN, 10=SATISFIABLE, 20=UNSATISFIABLE.
    /// As 'solve' it resets current assumptions and limits before returning.
    /// The numbers of rounds should not be negative.  If the number of rounds
    /// is zero only clauses are restored (if necessary) and top level unit
    /// propagation is performed, which both take some time.
    ///
    ///   require (READY)
    ///   ensure (STEADY  | SATISFIED | UNSATISFIED)
    ///
    #[inline]
    pub fn simplify(&mut self, rounds: i32) -> Status {
        ffi::simplify(&mut self.solver, rounds).into()
    }

    //------------------------------------------------------------------------
    /// Force termination of 'solve' asynchronously.
    ///
    ///  require (SOLVING | READY)
    ///  ensure (STEADY )           /// actually not immediately (synchronously)
    ///
    #[inline]
    pub fn terminate(&mut self) {
        ffi::terminate(&mut self.solver);
    }

    //------------------------------------------------------------------------

    /// We have the following common reference counting functions, which avoid
    /// to restore clauses but require substantial user guidance.  This was the
    /// only way to use inprocessing in incremental SAT solving in Lingeling
    /// (and before in `MiniSAT`'s 'freeze' / 'thaw') and which did not use
    /// automatic clause restoring.  In general this is slower than
    /// restoring clauses and should not be used.
    ///
    /// In essence the user freezes variables which potentially are still
    /// needed in clauses added or assumptions used after the next 'solve'
    /// call.  As in Lingeling you can freeze a variable multiple times, but
    /// then have to melt it the same number of times again in order to enable
    /// variable eliminating on it etc.  The arguments can be literals
    /// (negative indices) but conceptually variables are frozen.
    ///
    /// In the old way of doing things without restore you should not use a
    /// variable incrementally (in 'add' or 'assume'), which was used before
    /// and potentially could have been eliminated in a previous 'solve' call.
    /// This can lead to spurious satisfying assignment.  In order to check
    /// this API contract one can use the 'checkfrozen' option.  This has the
    /// drawback that restoring clauses implicitly would fail with a fatal
    /// error message even if in principle the solver could just restore
    /// clauses. Thus this option is disabled by default.
    ///
    /// See our SAT'19 paper [FazekasBiereScholl-SAT'19] for more details.
    ///
    ///   require (VALID)
    ///   ensure (VALID)
    ///
    #[must_use]
    #[inline]
    pub fn frozen(&self, lit: i32) -> bool {
        ffi::frozen(&self.solver, lit)
    }

    #[inline]
    pub fn freeze(&mut self, lit: i32) {
        ffi::freeze(&mut self.solver, lit);
    }

    #[inline]
    pub fn melt(&mut self, lit: i32) {
        ffi::melt(&mut self.solver, lit);
    }

    //------------------------------------------------------------------------
    /// Root level assigned variables can be queried with this function.
    /// It returns '1' if the literal is implied by the formula, '-1' if its
    /// negation is implied, or '0' if this is unclear at this point.
    ///
    ///   require (VALID)
    ///   ensure (VALID)
    ///
    #[must_use]
    #[inline]
    pub fn fixed(&self, lit: i32) -> i32 {
        ffi::fixed(&self.solver, lit)
    }

    //------------------------------------------------------------------------
    /// Force the default decision phase of a variable to a certain value.
    ///
    #[inline]
    pub fn phase(&mut self, lit: i32) {
        ffi::phase(&mut self.solver, lit);
    }

    #[inline]
    pub fn unphase(&mut self, lit: i32) {
        ffi::unphase(&mut self.solver, lit);
    }

    //------------------------------------------------------------------------
    /// Enables clausal proof tracing in DRAT format and returns 'true' if
    /// successfully opened for writing.  Writing proofs has to be enabled
    /// before calling 'solve', 'add' and 'dimacs', that is in state
    /// 'CONFIGURING'.  Otherwise only partial proofs would be written.
    ///
    ///   require (CONFIGURING)
    ///   ensure (CONFIGURING)
    ///
    /// Write DRAT proof.
    #[inline]
    pub fn trace_proof1(&mut self, file: String, name: String) -> bool {
        ffi::trace_proof1(&mut self.solver, file, name)
    }

    /// Open & write proof.
    #[inline]
    pub fn trace_proof2(&mut self, path: String) -> bool {
        ffi::trace_proof2(&mut self.solver, path)
    }

    /// Flushing the proof trace file eventually calls 'fflush' on the actual
    /// file or pipe and thus if this function returns all the proof steps
    /// should have been written (with the same guarantees as 'fflush').
    ///
    /// The additional optional argument forces to print the number of addition
    /// and deletion steps in the proof even if the verbosity level is zero but
    /// not if quiet is set as well.  The default for the stand-alone solver is
    /// to print this information (in the 'closing proof' section) but for API
    /// usage of the library we want to stay silent unless explicitly requested
    /// or verbosity is non-zero (and as explained quiet is not set).
    ///
    /// This function can be called multiple times.
    ///
    ///   require (VALID)
    ///   ensure (VALID)
    ///
    #[inline]
    pub fn flush_proof_trace(&mut self, print: bool) {
        ffi::flush_proof_trace(&mut self.solver, print);
    }

    /// Close proof trace early.  Similar to 'flush' we allow the user to
    /// control with 'print' in a more fine-grained way whether statistics
    /// about the size of the written proof file and if compressed on-the-fly
    /// the number of actual bytes written (including deflation percentage) are
    /// printed.  Before actually closing (or detaching in case of writing to
    /// '<stdout>') we check whether '`flush_proof_trace`' was called since the
    /// last time a proof step (addition or deletion) was traced.  If this is
    /// not the case we would call '`flush_proof_trace`' with the same 'print'
    /// argument.
    ///
    ///   require (VALID)
    ///   ensure (VALID)
    ///
    #[inline]
    pub fn close_proof_trace(&mut self, print: bool) {
        ffi::close_proof_trace(&mut self.solver, print);
    }

    // /// Enables clausal proof tracing with or without antecedents using
    // /// the Tracer interface defined in 'tracer.hpp'
    // ///
    // /// InternalTracer, StatTracer and FileTracer for internal use
    // ///
    // ///   require (CONFIGURING)
    // ///   ensure (CONFIGURING)
    // ///
    // pub fn connect_proof_tracer1(&mut self, tracer: &mut UniquePtr<Tracer>, antecedents: bool) {
    //     todo!()
    // }

    // pub fn connect_proof_tracer2(
    //     &mut self,
    //     tracer: &mut UniquePtr<InternalTracer>,
    //     antecedents: bool,
    // ) {
    //     todo!()
    // }
    // pub fn connect_proof_tracer3(&mut self, tracer: &mut UniquePtr<StatTracer>, antecedents: bool) {
    //     todo!()
    // }

    // pub fn connect_proof_tracer4(&mut self, tracer: &mut UniquePtr<FileTracer>, antecedents: bool) {
    //     todo!()
    // }

    /// Triggers the conclusion of incremental proofs.
    /// if the solver is SATISFIED it will trigger extend ()
    /// and give the model to the proof tracer through `conclude_sat` ()
    /// if the solver is UNSATISFIED it will trigger failing ()
    /// which will learn new clauses as explained below:
    /// In case of failed assumptions will provide a core negated
    /// as a clause through the proof tracer interface.
    /// With a failing contraint these can be multiple clauses.
    /// Then it will trigger a `conclude_unsat` event with the id(s)
    /// of the newly learnt clauses or the id of the global conflict.
    ///
    ///   require (SATISFIED || UNSATISFIED)
    ///   ensure (SATISFIED || UNSATISFIED)
    ///
    #[inline]
    pub fn conclude(&mut self) {
        ffi::conclude(&mut self.solver);
    }

    // /// Disconnect proof tracer. If this is not done before deleting
    // /// the tracer will be deleted. Returns true if successful.
    // ///
    // ///   require (VALID)
    // ///   ensure (VALID)
    // ///
    // pub fn disconnect_proof_tracer1(&mut self, tracer: &mut UniquePtr<Tracer>) -> bool {
    //     todo!()
    // }
    // pub fn disconnect_proof_tracer2(&mut self, tracer: &mut UniquePtr<StatTracer>) -> bool {
    //     todo!()
    // }
    // pub fn disconnect_proof_tracer3(&mut self, tracer: &mut UniquePtr<FileTracer>) -> bool {
    //     todo!()
    // }

    /// print usage information for long options
    #[inline]
    pub fn usage() {
        ffi::usage();
    }

    /// print configuration usage options
    #[inline]
    pub fn configurations() {
        ffi::configurations();
    }

    ///   require (!DELETING)
    ///   ensure (!DELETING)
    ///
    /// print statistics
    #[inline]
    pub fn statistics(&mut self) {
        ffi::statistics(&mut self.solver);
    }

    /// print resource usage (time and memory)
    #[inline]
    pub fn resources(&mut self) {
        ffi::resources(&mut self.solver);
    }

    ///   require (VALID)
    ///   ensure (VALID)
    ///
    /// print current option and value list
    #[inline]
    pub fn options(&mut self) {
        ffi::options(&mut self.solver);
    }

    //------------------------------------------------------------------------
    /// Traverse irredundant clauses or the extension stack in reverse order.
    ///
    /// The return value is false if traversal is aborted early due to one of
    /// the visitor functions returning false.  See description of the
    /// iterators below for more details on how to use these functions.
    ///
    ///   require (VALID)
    ///   ensure (VALID)
    ///
    #[inline]
    pub fn traverse_clauses<I: ClauseIterator>(&self, i: &mut I) -> bool {
        fn f<I: ClauseIterator>(state: *mut u8, clause: &[i32]) -> bool {
            let ptr: *mut I = state.cast::<I>();
            let i = unsafe { &mut *ptr };
            i.clause(clause)
        }
        let mut iter =
            unsafe { ffi::new_clause_iterator(std::ptr::from_mut(i).cast::<u8>(), f::<I>) };
        ffi::traverse_clauses(&self.solver, &mut iter)
    }

    #[inline]
    pub fn traverse_witnesses_backward<I: WitnessIterator>(&self, i: &mut I) -> bool {
        fn f<I: WitnessIterator>(state: *mut u8, clause: &[i32], witness: &[i32], id: u64) -> bool {
            let ptr: *mut I = state.cast::<I>();
            let i = unsafe { &mut *ptr };
            i.witness(clause, witness, id)
        }
        let mut iter =
            unsafe { ffi::new_witness_iterator(std::ptr::from_mut(i).cast::<u8>(), f::<I>) };
        ffi::traverse_witnesses_backward(&self.solver, &mut iter)
    }

    #[inline]
    pub fn traverse_witnesses_forward<I: WitnessIterator>(&self, i: &mut I) -> bool {
        fn f<I: WitnessIterator>(state: *mut u8, clause: &[i32], witness: &[i32], id: u64) -> bool {
            let ptr: *mut I = state.cast::<I>();
            let i = unsafe { &mut *ptr };
            i.witness(clause, witness, id)
        }
        let mut iter =
            unsafe { ffi::new_witness_iterator(std::ptr::from_mut(i).cast::<u8>(), f::<I>) };
        ffi::traverse_witnesses_forward(&self.solver, &mut iter)
    }

    //------------------------------------------------------------------------
    /// Files with explicit path argument support compressed input and output
    /// if appropriate helper functions 'gzip' etc. are available.  They are
    /// called through opening a pipe to an external command.
    ///
    /// If the 'strict' argument is zero then the number of variables and
    /// clauses specified in the DIMACS headers are ignored, i.e., the header
    /// 'p cnf 0 0' is always legal.  If the 'strict' argument is larger '1'
    /// strict formatting of the header is required, i.e., single spaces
    /// everywhere and no trailing white space.
    ///
    /// Returns zero if successful and otherwise an error message.
    ///
    ///   require (VALID)
    ///   ensure (VALID)
    ///
    #[inline]
    pub fn read_dimacs1(
        &mut self,
        file: String,
        name: String,
        vars: &mut i32,
        strict: i32,
    ) -> String {
        ffi::read_dimacs1(&mut self.solver, file, name, vars, strict)
    }

    #[inline]
    pub fn read_dimacs2(&mut self, path: String, vars: &mut i32, strict: i32) -> String {
        ffi::read_dimacs2(&mut self.solver, path, vars, strict)
    }

    /// The following routines work the same way but parse both DIMACS and
    /// INCCNF files (with 'p inccnf' header and 'a <cube>' lines).  If the
    /// parser finds and 'p inccnf' header or cubes then '*incremental' is set
    /// to true and the cubes are stored in the given vector (each cube
    /// terminated by a zero).
    #[inline]
    pub fn read_dimacs3(
        &mut self,
        file: String,
        name: String,
        vars: &mut i32,
        strict: i32,
        incremental: &mut bool,
        cubes: &mut Vec<i32>,
    ) -> String {
        ffi::read_dimacs3(
            &mut self.solver,
            file,
            name,
            vars,
            strict,
            incremental,
            cubes,
        )
    }

    #[inline]
    pub fn read_dimacs4(
        &mut self,
        path: String,
        vars: &mut i32,
        strict: i32,
        incremental: &mut bool,
        cubes: &mut Vec<i32>,
    ) -> String {
        ffi::read_dimacs4(&mut self.solver, path, vars, strict, incremental, cubes)
    }

    //------------------------------------------------------------------------
    /// Write current irredundant clauses and all derived unit clauses
    /// to a file in DIMACS format.  Clauses on the extension stack are
    /// not included, nor any redundant clauses.
    ///
    /// The '`min_max_var`' parameter gives a lower bound on the number '<vars>'
    /// of variables used in the DIMACS 'p cnf <vars> ...' header.
    ///
    /// Returns zero if successful and otherwise an error message.
    ///
    ///   require (VALID)
    ///   ensure (VALID)
    ///
    #[inline]
    pub fn write_dimacs(&mut self, path: String, min_max_var: i32) -> String {
        ffi::write_dimacs(&mut self.solver, path, min_max_var)
    }

    /// The extension stack for reconstruction a solution can be written too.
    ///
    #[inline]
    pub fn write_extension(&mut self, path: String) -> String {
        ffi::write_extension(&mut self.solver, path)
    }

    /// Print build configuration to a file with prefix 'c '.  If the file
    /// is '<stdout>' or '<stderr>' then terminal color codes might be used.
    ///
    #[inline]
    pub fn build(file: String, prefix: String) {
        ffi::build(file, prefix);
    }
}

/// Connected terminators are checked for termination regularly.  If the
/// 'terminate' function of the terminator returns true the solver is
/// terminated synchronously as soon it calls this function.
pub trait Terminator {
    fn terminated(&mut self) -> bool;
}

/// Connected learners which can be used to export learned clauses.
/// The 'learning' can check the size of the learn clause and only if it
/// returns true then the individual literals of the learned clause are given
/// to the learn through 'learn' one by one terminated by a zero literal.
pub trait Learner {
    fn learning(&mut self, size: i32) -> bool;
    fn learn(&mut self, lit: i32);
}

/// Connected listener gets notified whenever the truth value of a variable is
/// fixed (for example during inprocessing or due to some derived unit clauses).
pub trait FixedAssignmentListener {
    fn notify_fixed_assignment(&mut self, lit: i32);
}

use std::vec::Vec;

/// Allows to connect an external propagator to propagate values to variables
/// with an external clause as a reason or to learn new clauses during the
/// CDCL loop (without restart).
pub trait ExternalPropagator {
    /// lazy propagator only checks complete assignments
    fn is_lazy(&self) -> bool {
        false
    }

    /// Reason external clauses can be deleted
    fn are_reasons_forgettable(&self) -> bool {
        false
    }

    /// Notify the propagator about assignments to observed variables.
    /// The notification is not necessarily eager. It usually happens before
    /// the call of propagator callbacks and when a driving clause is leading
    /// to an assignment.
    fn notify_assignment(&mut self, lits: &[i32]);

    fn notify_new_decision_level(&mut self);

    fn notify_backtrack(&mut self, new_level: usize);

    /// Check by the external propagator the found complete solution (after
    /// solution reconstruction). If it returns false, the propagator must
    /// provide an external clause during the next callback.
    fn cb_check_found_model(&self, model: &[i32]) -> bool;

    /// Ask the external propagator for the next decision literal. If it
    /// returns 0, the solver makes its own choice.
    fn cb_decide(&self) -> i32 {
        0
    }

    /// Ask the external propagator if there is an external propagation to make
    /// under the current assignment. It returns either a literal to be
    /// propagated or 0, indicating that there is no external propagation under
    /// the current assignment.
    fn cb_propagate(&self) -> i32 {
        0
    }

    /// Ask the external propagator for the reason clause of a previous
    /// external propagation step (done by `cb_propagate`). The clause must be
    /// added literal-by-literal closed with a 0. Further, the clause must
    /// contain the propagated literal.
    ///
    /// The clause will be learned as an Irredundant Non-Forgettable Clause (see
    /// below at '`cb_has_external_clause`' more details about it).
    fn cb_add_reason_clause_lit(&self, _propagated_lit: i32) -> i32 {
        0
    }

    /// The following two functions are used to add external clauses to the
    /// solver during the CDCL loop. The external clause is added
    /// literal-by-literal and learned by the solver as an irredundant
    /// (original) input clause. The clause can be arbitrary, but if it is
    /// root-satisfied or tautology, the solver will ignore it without learning
    /// it. Root-falsified literals are eagerly removed from the clause.
    /// Falsified clauses trigger conflict analysis, propagating clauses
    /// trigger propagation. In case chrono is 0, the solver backtracks to
    /// propagate the new literal on the right decision level, otherwise it
    /// potentially will be an out-of-order assignment on the current level.
    /// Unit clauses always (unless root-satisfied, see above) trigger
    /// backtracking (independently from the value of the chrono option and
    /// independently from being falsified or satisfied or unassigned) to level
    /// 0. Empty clause (or root falsified clause, see above) makes the problem
    /// unsat and stops the search immediately. A literal 0 must close the
    /// clause.
    ///
    /// The external propagator indicates that there is a clause to add.
    /// The parameter of the function allows the user to indicate that how
    /// 'forgettable' is the external clause. Forgettable clauses are allowed
    /// to be removed by the SAT solver during clause database reduction.
    /// However, it is up to the solver to decide when actually the clause is
    /// deleted. For example, unit clauses, even forgettable ones, will not be
    /// deleted. In case the clause is not 'forgettable' (the parameter is false),
    /// the solver considers the clause to be irredundant.
    ///
    /// In case the solver produces incremental proofs, these external clauses
    /// are added to the proof during solving at real-time, i.e., the proof
    /// checker can ignore them until that point (so added as input clause, but
    /// input after the query line).
    ///
    /// Reason clauses of external propagation steps are assumed to be
    /// forgettable, parameter '`reason_forgettable`' can be used to change it.
    ///
    /// Currently, every external clause is expected to be over observed
    /// (therefore frozen) variables, hence no tainting or restore steps
    /// are performed upon their addition. This will be changed in later
    /// versions probably.
    fn cb_has_external_clause(&self, is_forgettable: &mut bool) -> bool;

    /// The actual function called to add the external clause.
    fn cb_add_external_clause_lit(&self) -> i32;
}

/// Allows to traverse all remaining irredundant clauses.  Satisfied and
/// eliminated clauses are not included, nor any derived units unless such
/// a unit literal is frozen. Falsified literals are skipped.  If the solver
/// is inconsistent only the empty clause is traversed.
///
/// If 'clause' returns false traversal aborts early.
pub trait ClauseIterator {
    fn clause(&mut self, clause: &[i32]) -> bool;
}

/// Allows to traverse all clauses on the extension stack together with their
/// witness cubes.  If the solver is inconsistent, i.e., an empty clause is
/// found and the formula is unsatisfiable, then nothing is traversed.
///
/// The clauses traversed in '`traverse_clauses`' together with the clauses on
/// the extension stack are logically equivalent to the original clauses.
/// See our SAT'19 paper for more details.
///
/// The witness literals can be used to extend and fix an assignment on the
/// remaining clauses to satisfy the clauses on the extension stack too.
///
/// All derived units of non-frozen variables are included too.
///
/// If 'witness' returns false traversal aborts early.
pub trait WitnessIterator {
    fn witness(&mut self, clause: &[i32], witness: &[i32], id: u64) -> bool;
}

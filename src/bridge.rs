#[cxx::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("cadical-sys/src/cadical_bridge.hpp");

        type Solver;

        /// The SAT competition standardized the exit code of SAT solvers to the
        /// following which then is also used return code for 'solve' functions.
        /// In the following example we use those constants for brevity though.
        type Status;

        /// Solver State
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

        /// Constructor and basic operations
        #[must_use]
        pub fn constructor() -> UniquePtr<Solver>;

        #[must_use]
        pub fn signature() -> String;

        /// Core functionality as in the IPASIR incremental SAT solver interface.
        /// (recall 'READY = CONFIGURING | STEADY  | SATISFIED | UNSATISFIED').
        /// Further note that 'lit' is required to be different from 'INT_MIN' and
        /// different from '0' except for 'add'.
        ///
        /// Add valid literal to clause or zero to terminate clause.
        ///
        ///   require (VALID)                  // recall 'VALID = READY | ADDING'
        ///   if (lit) ensure (ADDING)         // and thus VALID but not READY
        ///   if (!lit) ensure (STEADY )       // and thus READY
        ///
        pub fn add(solver: &mut UniquePtr<Solver>, literal: i32);

        /// Here are functions simplifying clause addition. The given literals
        /// should all be valid (different from 'INT_MIN' and different from '0').
        ///
        ///   require (VALID)
        ///   ensure (STEADY )
        ///
        pub fn clause1(solver: &mut UniquePtr<Solver>, l1: i32);
        pub fn clause2(solver: &mut UniquePtr<Solver>, l1: i32, l2: i32);
        pub fn clause3(solver: &mut UniquePtr<Solver>, l1: i32, l2: i32, l3: i32);
        pub fn clause4(solver: &mut UniquePtr<Solver>, l1: i32, l2: i32, l3: i32, l4: i32);
        pub fn clause5(solver: &mut UniquePtr<Solver>, l1: i32, l2: i32, l3: i32, l4: i32, l5: i32);
        pub fn clause6(solver: &mut UniquePtr<Solver>, v: &[i32]);

        /// Function that makes clause from any slice of integers.
        ///
        /// # Safety
        ///
        /// This function must be called with a valid pointer to a slice of integers.
        #[allow(clippy::missing_safety_doc)]
        pub unsafe fn clause7(solver: &mut UniquePtr<Solver>, ptr: *const i32, n: usize);

        /// This function can be used to check if the formula is already
        /// inconsistent (contains the empty clause or was proven to be
        /// root-level unsatisfiable).
        pub fn inconsistent(solver: &mut UniquePtr<Solver>) -> bool;

        /// Assume valid non zero literal for next call to 'solve'.  These
        /// assumptions are reset after the call to 'solve' as well as after
        /// returning from 'simplify' and 'lookahead.
        ///
        ///   require (READY)
        ///   ensure (STEADY )
        ///
        pub fn assume(solver: &mut UniquePtr<Solver>, lit: i32);

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
        pub fn solve(solver: &mut UniquePtr<Solver>) -> i32;

        /// Get value (-lit=false, lit=true) of valid non-zero literal.
        ///
        ///   require (SATISFIED)
        ///   ensure (SATISFIED)
        ///
        pub fn val(solver: &mut UniquePtr<Solver>, lit: i32) -> i32;

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
        pub fn flip(solver: &mut UniquePtr<Solver>, lit: i32) -> bool;

        /// Same as 'flip' without actually flipping it. This functionality is
        /// currently not supported in the presence of an external propagator.
        ///
        ///   require (SATISFIED)
        ///   ensure (SATISFIED)
        ///
        pub fn flippable(solver: &mut UniquePtr<Solver>, lit: i32) -> bool;

        /// Determine whether the valid non-zero literal is in the core.
        /// Returns 'true' if the literal is in the core and 'false' otherwise.
        /// Note that the core does not have to be minimal.
        ///
        ///   require (UNSATISFIED)
        ///   ensure (UNSATISFIED)
        ///
        pub fn failed(solver: &mut UniquePtr<Solver>, lit: i32) -> bool;

        /// Add call-back which is checked regularly for termination.  There can
        /// only be one terminator connected.  If a second (non-zero) one is added
        /// the first one is implicitly disconnected.
        ///
        ///   require (VALID)
        ///   ensure (VALID)
        ///
        pub fn connect_terminator(
            solver: &mut UniquePtr<Solver>,
            terminator: &mut UniquePtr<Terminator>,
        );
        pub fn disconnect_terminator(solver: &mut UniquePtr<Solver>);

        /// Add call-back which allows to export learned clauses.
        ///
        ///   require (VALID)
        ///   ensure (VALID)
        ///
        pub fn connect_learner(solver: &mut UniquePtr<Solver>, learner: &mut UniquePtr<Learner>);
        pub fn disconnect_learner(solver: &mut UniquePtr<Solver>);

        /// Add call-back which allows to observe when a variable is fixed.
        ///
        ///   require (VALID)
        ///   ensure (VALID)
        ///
        pub fn connect_fixed_listener(
            solver: &mut UniquePtr<Solver>,
            fixed_listener: &mut UniquePtr<FixedAssignmentListener>,
        );
        pub fn disconnect_fixed_listener(solver: &mut UniquePtr<Solver>);

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
        pub fn connect_external_propagator(
            solver: &mut UniquePtr<Solver>,
            propagator: &mut UniquePtr<ExternalPropagator>,
        );
        pub fn disconnect_external_propagator(solver: &mut UniquePtr<Solver>);

        /// Mark as 'observed' those variables that are relevant to the external
        /// propagator. External propagation, clause addition during search and
        /// notifications are all over these observed variabes.
        /// A variable can not be observed witouth having an external propagator
        /// connected. Observed variables are "frozen" internally, and so
        /// inprocessing will not consider them as candidates for elimination.
        /// An observed variable is allowed to be a fresh variable and it can be
        /// added also during solving.
        ///
        ///   require (VALID_OR_SOLVING)
        ///   ensure (VALID_OR_SOLVING)
        ///
        pub fn add_observed_var(solver: &mut UniquePtr<Solver>, var: i32);

        /// Removes the 'observed' flag from the given variable. A variable can be
        /// set unobserved only between solve calls, not during it (to guarantee
        /// that no yet unexplained external propagation involves it).
        ///
        ///   require (VALID)
        ///   ensure (VALID)
        ///
        pub fn remove_observed_var(solver: &mut UniquePtr<Solver>, var: i32);

        /// Removes all the 'observed' flags from the variables. Disconnecting the
        /// propagator invokes this step as well.
        ///
        ///   require (VALID)
        ///   ensure (VALID)
        ///
        pub fn reset_observed_vars(solver: &mut UniquePtr<Solver>);

        /// Get reason of valid observed literal (true = it is an observed variable
        /// and it got assigned by a decision during the CDCL loop. Otherwise:
        /// false.
        ///
        ///   require (VALID_OR_SOLVING)
        ///   ensure (VALID_OR_SOLVING)
        ///
        pub fn is_decision(solver: &mut UniquePtr<Solver>, lit: i32) -> bool;

        /// Force solve to backtrack to certain decision level. Can be called only
        /// during 'cb_decide' of a connected External Propagator.
        /// Invoking in any other time will not have an effect.
        /// If the call had an effect, the External Propagator will be notified about
        /// the backtrack via 'notify_backtrack'.
        ///
        ///   require (SOLVING)
        ///   ensure (SOLVING)
        ///
        pub fn force_backtrack(solver: &mut UniquePtr<Solver>, new_level: usize);

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
        ///   READY if (!lit) && !adding_clause ensure (STEADY ) // and thus READY
        ///
        pub fn constrain(solver: &mut UniquePtr<Solver>, lit: i32);

        /// Determine whether the constraint was used to proof the
        /// unsatisfiability. Note that the formula might still be unsatisfiable
        /// without the constraint.
        ///
        ///   require (UNSATISFIED)
        ///   ensure (UNSATISFIED)
        ///
        pub fn constraint_failed(solver: &mut UniquePtr<Solver>) -> bool;

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
        pub fn lookahead(solver: &mut UniquePtr<Solver>) -> i32;

        pub fn generate_cubes(
            solver: &mut UniquePtr<Solver>,
            x: i32,
            min_depth: i32,
            result_cubes: &mut Vec<i32>,
        ) -> i32;

        pub fn reset_assumptions(solver: &mut UniquePtr<Solver>);

        pub fn reset_constraint(solver: &mut UniquePtr<Solver>);

        /// Return the current state of the solver as defined above.
        ///
        #[must_use]
        pub fn state(solver: &UniquePtr<Solver>) -> i32;

        /// Similar to 'state ()' but using the staddard competition exit codes of
        /// '10' for 'SATISFIABLE', '20' for 'UNSATISFIABLE' and '0' otherwise.
        ///
        #[must_use]
        pub fn status(solver: &UniquePtr<Solver>) -> i32;

        /// return version string
        #[must_use]
        pub fn version() -> String;

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
        pub fn copy(source: &UniquePtr<Solver>, destination: &mut UniquePtr<Solver>);

        /*----------------------------------------------------------------------*/
        /// Variables are usually added and initialized implicitly whenever a
        /// literal is used as an argument except for the functions 'val', 'fixed',
        /// 'failed' and 'frozen'.  However, the library internally keeps a maximum
        /// variable index, which can be queried.
        ///
        ///   require (VALID | SOLVING)
        ///   ensure (VALID | SOLVING)
        ///
        pub fn vars(solver: &mut UniquePtr<Solver>) -> i32;

        /// Increase the maximum variable index explicitly.  This function makes
        /// sure that at least 'min_max_var' variables are initialized.  Since it
        /// might need to reallocate tables, it destroys a satisfying assignment
        /// and has the same state transition and conditions as 'assume' etc.
        ///
        ///   require (READY)
        ///   ensure (STEADY )
        ///
        pub fn reserve(solver: &mut UniquePtr<Solver>, min_max_var: i32);

        /// pub fn trace_api_calls(solver: &mut UniquePtr<Solver>, file: String);

        //------------------------------------------------------------------------
        /// Option handling.
        /// Determine whether 'name' is a valid option name.
        ///
        #[must_use]
        pub fn is_valid_option(name: String) -> bool;

        /// Determine whether 'name' enables a specific preprocessing technique.
        ///
        #[must_use]
        pub fn is_preprocessing_option(name: String) -> bool;

        /// Determine whether 'arg' is a valid long option of the form '--<name>',
        /// '--<name>=<val>' or '--no-<name>' similar to 'set_long_option' below.
        /// Legal values are 'true', 'false', or '[-]<mantissa>[e<exponent>]'.
        #[must_use]
        pub fn is_valid_long_option(arg: String) -> bool;

        /// Get the current value of the option 'name'.  If 'name' is invalid then
        /// zero is returned.  Here '--...' arguments as invalid options.
        ///
        pub fn get(solver: &mut UniquePtr<Solver>, name: String) -> i32;

        /// Set the default verbose message prefix (default "c ").
        ///
        pub fn prefix(solver: &mut UniquePtr<Solver>, verbose_message_prefix: String);

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
        pub fn set(solver: &mut UniquePtr<Solver>, name: String, val: i32) -> bool;

        /// This function accepts options in command line syntax:
        ///
        ///   '--<name>=<val>', '--<name>', or '--no-<name>'
        ///
        /// It actually calls the previous 'set' function after parsing 'arg'.  The
        /// same values are expected as for 'is_valid_long_option' above and as
        /// with 'set' any value outside of the range of legal values for a
        /// particular option are set to either the minimum or maximum depending on
        /// which side of the valid interval they lie.
        ///
        ///   require (CONFIGURING)
        ///   ensure (CONFIGURING)
        ///
        pub fn set_long_option(solver: &mut UniquePtr<Solver>, arg: String) -> bool;

        /// Determine whether 'name' is a valid configuration.
        ///
        #[must_use]
        pub fn is_valid_configuration(name: String) -> bool;

        /// Overwrite (some) options with the forced values of the configuration.
        /// The result is 'true' iff the 'name' is a valid configuration.
        ///
        ///   require (CONFIGURING)
        ///   ensure (CONFIGURING)
        ///
        pub fn configure(solver: &mut UniquePtr<Solver>, name: String) -> bool;

        /// Increase preprocessing and inprocessing limits by '10^<val>'.  Values
        /// below '0' are ignored and values above '9' are reduced to '9'.
        ///
        ///   require (READY)
        ///   ensure (READY)
        ///
        pub fn optimize(solver: &mut UniquePtr<Solver>, val: i32);

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
        pub fn limit(solver: &mut UniquePtr<Solver>, arg: String, val: i32) -> bool;
        pub fn is_valid_limit(solver: &mut UniquePtr<Solver>, arg: String) -> bool;

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
        pub fn active(solver: &UniquePtr<Solver>) -> i32;

        /// Number of active redundant clauses.
        #[must_use]
        pub fn redundant(solver: &UniquePtr<Solver>) -> i64;

        /// Number of active irredundant clauses.
        #[must_use]
        pub fn irredundant(solver: &UniquePtr<Solver>) -> i64;

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
        pub fn simplify(solver: &mut UniquePtr<Solver>, rounds: i32) -> i32;

        //------------------------------------------------------------------------
        /// Force termination of 'solve' asynchronously.
        ///
        ///  require (SOLVING | READY)
        ///  ensure (STEADY )           /// actually not immediately (synchronously)
        ///
        pub fn terminate(solver: &mut UniquePtr<Solver>);

        //------------------------------------------------------------------------

        /// We have the following common reference counting functions, which avoid
        /// to restore clauses but require substantial user guidance.  This was the
        /// only way to use inprocessing in incremental SAT solving in Lingeling
        /// (and before in MiniSAT's 'freeze' / 'thaw') and which did not use
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
        pub fn frozen(solver: &UniquePtr<Solver>, lit: i32) -> bool;
        pub fn freeze(solver: &mut UniquePtr<Solver>, lit: i32);
        pub fn melt(solver: &mut UniquePtr<Solver>, lit: i32);

        //------------------------------------------------------------------------
        /// Root level assigned variables can be queried with this function.
        /// It returns '1' if the literal is implied by the formula, '-1' if its
        /// negation is implied, or '0' if this is unclear at this point.
        ///
        ///   require (VALID)
        ///   ensure (VALID)
        ///
        #[must_use]
        pub fn fixed(solver: &UniquePtr<Solver>, lit: i32) -> i32;

        //------------------------------------------------------------------------
        /// Force the default decision phase of a variable to a certain value.
        ///
        pub fn phase(solver: &mut UniquePtr<Solver>, lit: i32);
        pub fn unphase(solver: &mut UniquePtr<Solver>, lit: i32);

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
        pub fn trace_proof1(solver: &mut UniquePtr<Solver>, file: String, name: String) -> bool;

        /// Open & write proof.
        pub fn trace_proof2(solver: &mut UniquePtr<Solver>, path: String) -> bool;

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
        pub fn flush_proof_trace(solver: &mut UniquePtr<Solver>, print: bool);

        /// Close proof trace early.  Similar to 'flush' we allow the user to
        /// control with 'print' in a more fine-grained way whether statistics
        /// about the size of the written proof file and if compressed on-the-fly
        /// the number of actual bytes written (including deflation percentage) are
        /// printed.  Before actually closing (or detaching in case of writing to
        /// '<stdout>') we check whether 'flush_proof_trace' was called since the
        /// last time a proof step (addition or deletion) was traced.  If this is
        /// not the case we would call 'flush_proof_trace' with the same 'print'
        /// argument.
        ///
        ///   require (VALID)
        ///   ensure (VALID)
        ///
        pub fn close_proof_trace(solver: &mut UniquePtr<Solver>, print: bool);

        /// Enables clausal proof tracing with or without antecedents using
        /// the Tracer interface defined in 'tracer.hpp'
        ///
        /// InternalTracer, StatTracer and FileTracer for internal use
        ///
        ///   require (CONFIGURING)
        ///   ensure (CONFIGURING)
        ///
        pub fn connect_proof_tracer1(
            solver: &mut UniquePtr<Solver>,
            tracer: &mut UniquePtr<Tracer>,
            antecedents: bool,
        );
        pub fn connect_proof_tracer2(
            solver: &mut UniquePtr<Solver>,
            tracer: &mut UniquePtr<InternalTracer>,
            antecedents: bool,
        );
        pub fn connect_proof_tracer3(
            solver: &mut UniquePtr<Solver>,
            tracer: &mut UniquePtr<StatTracer>,
            antecedents: bool,
        );
        pub fn connect_proof_tracer4(
            solver: &mut UniquePtr<Solver>,
            tracer: &mut UniquePtr<FileTracer>,
            antecedents: bool,
        );

        /// Triggers the conclusion of incremental proofs.
        /// if the solver is SATISFIED it will trigger extend ()
        /// and give the model to the proof tracer through conclude_sat ()
        /// if the solver is UNSATISFIED it will trigger failing ()
        /// which will learn new clauses as explained below:
        /// In case of failed assumptions will provide a core negated
        /// as a clause through the proof tracer interface.
        /// With a failing contraint these can be multiple clauses.
        /// Then it will trigger a conclude_unsat event with the id(s)
        /// of the newly learnt clauses or the id of the global conflict.
        ///
        ///   require (SATISFIED || UNSATISFIED)
        ///   ensure (SATISFIED || UNSATISFIED)
        ///
        pub fn conclude(solver: &mut UniquePtr<Solver>);

        /// Disconnect proof tracer. If this is not done before deleting
        /// the tracer will be deleted. Returns true if successful.
        ///
        ///   require (VALID)
        ///   ensure (VALID)
        ///
        pub fn disconnect_proof_tracer1(
            solver: &mut UniquePtr<Solver>,
            tracer: &mut UniquePtr<Tracer>,
        ) -> bool;
        pub fn disconnect_proof_tracer2(
            solver: &mut UniquePtr<Solver>,
            tracer: &mut UniquePtr<StatTracer>,
        ) -> bool;
        pub fn disconnect_proof_tracer3(
            solver: &mut UniquePtr<Solver>,
            tracer: &mut UniquePtr<FileTracer>,
        ) -> bool;

        /// print usage information for long options
        pub fn usage();

        /// print configuration usage options
        pub fn configurations();

        ///   require (!DELETING)
        ///   ensure (!DELETING)
        ///
        /// print statistics
        pub fn statistics(solver: &mut UniquePtr<Solver>);

        /// print resource usage (time and memory)
        pub fn resources(solver: &mut UniquePtr<Solver>);

        ///   require (VALID)
        ///   ensure (VALID)
        ///
        /// print current option and value list
        pub fn options(solver: &mut UniquePtr<Solver>);

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
        pub fn traverse_clauses(
            solver: &UniquePtr<Solver>,
            i: &mut UniquePtr<ClauseIterator>,
        ) -> bool;
        pub fn traverse_witnesses_backward(
            solver: &UniquePtr<Solver>,
            i: &mut UniquePtr<WitnessIterator>,
        ) -> bool;
        pub fn traverse_witnesses_forward(
            solver: &UniquePtr<Solver>,
            i: &mut UniquePtr<WitnessIterator>,
        ) -> bool;

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
        pub fn read_dimacs1(
            solver: &mut UniquePtr<Solver>,
            file: String,
            name: String,
            vars: &mut i32,
            strict: i32,
        ) -> String;

        pub fn read_dimacs2(
            solver: &mut UniquePtr<Solver>,
            path: String,
            vars: &mut i32,
            strict: i32,
        ) -> String;

        /// The following routines work the same way but parse both DIMACS and
        /// INCCNF files (with 'p inccnf' header and 'a <cube>' lines).  If the
        /// parser finds and 'p inccnf' header or cubes then '*incremental' is set
        /// to true and the cubes are stored in the given vector (each cube
        /// terminated by a zero).
        pub fn read_dimacs3(
            solver: &mut UniquePtr<Solver>,
            file: String,
            name: String,
            vars: &mut i32,
            strict: i32,
            incremental: &mut bool,
            cubes: &mut Vec<i32>,
        ) -> String;

        pub fn read_dimacs4(
            solver: &mut UniquePtr<Solver>,
            path: String,
            vars: &mut i32,
            strict: i32,
            incremental: &mut bool,
            cubes: &mut Vec<i32>,
        ) -> String;

        //------------------------------------------------------------------------
        /// Write current irredundant clauses and all derived unit clauses
        /// to a file in DIMACS format.  Clauses on the extension stack are
        /// not included, nor any redundant clauses.
        ///
        /// The 'min_max_var' parameter gives a lower bound on the number '<vars>'
        /// of variables used in the DIMACS 'p cnf <vars> ...' header.
        ///
        /// Returns zero if successful and otherwise an error message.
        ///
        ///   require (VALID)
        ///   ensure (VALID)
        ///
        pub fn write_dimacs(
            solver: &mut UniquePtr<Solver>,
            path: String,
            min_max_var: i32,
        ) -> String;

        /// The extension stack for reconstruction a solution can be written too.
        ///
        pub fn write_extension(solver: &mut UniquePtr<Solver>, path: String) -> String;

        /// Print build configuration to a file with prefix 'c '.  If the file
        /// is '<stdout>' or '<stderr>' then terminal color codes might be used.
        ///
        pub fn build(file: String, prefix: String);

        /// Connected terminators are checked for termination regularly.  If the
        /// 'terminate' function of the terminator returns true the solver is
        /// terminated synchronously as soon it calls this function.
        pub fn new_terminator(terminate: fn() -> bool) -> UniquePtr<Terminator>;

        /// Connected learners which can be used to export learned clauses.
        /// The 'learning' can check the size of the learn clause and only if it
        /// returns true then the individual literals of the learned clause are given
        /// to the learn through 'learn' one by one terminated by a zero literal.
        pub fn new_learner(learning: fn(i32) -> bool, learn: fn(i32)) -> UniquePtr<Learner>;

        // Connected listener gets notified whenever the truth value of a variable is
        // fixed (for example during inprocessing or due to some derived unit clauses).
        pub fn new_fixed_assignment_listener(fixed: fn(i32)) -> UniquePtr<FixedAssignmentListener>;

        /// Allows to traverse all remaining irredundant clauses.  Satisfied and
        /// eliminated clauses are not included, nor any derived units unless such
        /// a unit literal is frozen. Falsified literals are skipped.  If the solver
        /// is inconsistent only the empty clause is traversed.
        ///
        /// If 'clause' returns false traversal aborts early.
        ///
        /// # Safety
        ///
        /// The pointers in this function and in the function passed to it are
        /// there to allow the state changes. Where the pointer points to a
        /// generic state that the user of this function wants. This pointer must
        /// remain valid throughout the run.
        #[allow(clippy::missing_safety_doc)]
        pub unsafe fn new_clause_iterator(
            s: *mut u8,
            clause: unsafe fn(*mut u8, &[i32]) -> bool,
        ) -> UniquePtr<ClauseIterator>;

        /// Allows to traverse all clauses on the extension stack together with their
        /// witness cubes.  If the solver is inconsistent, i.e., an empty clause is
        /// found and the formula is unsatisfiable, then nothing is traversed.
        ///
        /// The clauses traversed in 'traverse_clauses' together with the clauses on
        /// the extension stack are logically equivalent to the original clauses.
        /// See our SAT'19 paper for more details.
        ///
        /// The witness literals can be used to extend and fix an assignment on the
        /// remaining clauses to satisfy the clauses on the extension stack too.
        ///
        /// All derived units of non-frozen variables are included too.
        ///
        /// If 'witness' returns false traversal aborts early.
        ///
        /// # Safety
        ///
        /// The pointers in this function and in the function passed to it are
        /// there to allow the state changes. Where the pointer points to a
        /// generic state that the user of this function wants. This pointer must
        /// remain valid throughout the run.
        #[allow(clippy::missing_safety_doc)]
        pub unsafe fn new_witness_iterator(
            s: *mut u8,
            witness: unsafe fn(*mut u8, &[i32], &[i32], u64) -> bool,
        ) -> UniquePtr<WitnessIterator>;
    }
}

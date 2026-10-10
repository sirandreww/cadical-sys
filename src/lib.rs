#![doc = include_str!("../README.md")]

use bridge::ffi;
use cxx::memory::UniquePtrTarget;
use cxx::UniquePtr;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// The raw FFI bindings. They are private: they take `UniquePtr`s that may be
// null and C++ objects whose lifetimes they cannot check, so exposing them as
// safe functions would let safe code cause undefined behaviour.
mod bridge;

/// The C++ shim reports success as an empty string and failure as an error message.
fn into_result(message: String) -> Result<(), String> {
    if message.is_empty() {
        Ok(())
    } else {
        Err(message)
    }
}

/// Literals must be non-zero and not `i32::MIN`; `CaDiCaL` aborts the process
/// otherwise, so the safe API panics first.
#[track_caller]
fn require_valid_lit(function: &str, lit: i32) {
    assert!(
        lit != 0 && lit != i32::MIN,
        "{function} called with invalid literal {lit}"
    );
}

/// Like [`require_valid_lit`] for functions where zero terminates a clause.
#[track_caller]
fn require_valid_lit_or_zero(function: &str, lit: i32) {
    assert!(
        lit != i32::MIN,
        "{function} called with invalid literal {lit}"
    );
}

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
    /// unknown allows 'implied'
    INCONCLUSIVE = 256,

    /// These combined states are used to check contracts.
    /// CONFIGURING | STEADY | SATISFIED | UNSATISFIED | INCONCLUSIVE,
    READY = 358,
    /// READY | ADDING
    VALID = 366,
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
            256 => State::INCONCLUSIVE,
            358 => State::READY,
            366 => State::VALID,
            129 => State::INVALID,
            _ => unreachable!(),
        }
    }
}

/// A Rust callback object together with the C++ wrapper that calls into it.
///
/// The C++ wrapper only holds a raw pointer to the callback, so the callback
/// is boxed and owned here rather than borrowed from the caller. This is what
/// guarantees the callback outlives every call the solver makes into it.
struct Connected<W: UniquePtrTarget> {
    wrapper: UniquePtr<W>,
    callback: *mut u8,
    drop_callback: unsafe fn(*mut u8),
}

impl<W: UniquePtrTarget> Connected<W> {
    /// Boxes `callback` and builds its C++ wrapper from a pointer to it.
    fn new<T>(callback: T, make_wrapper: impl FnOnce(*mut u8) -> UniquePtr<W>) -> Self {
        unsafe fn drop_box<T>(ptr: *mut u8) {
            drop(unsafe { Box::from_raw(ptr.cast::<T>()) });
        }
        let callback = Box::into_raw(Box::new(callback)).cast::<u8>();
        Self {
            wrapper: make_wrapper(callback),
            callback,
            drop_callback: drop_box::<T>,
        }
    }
}

impl<W: UniquePtrTarget> Drop for Connected<W> {
    fn drop(&mut self) {
        // Free the wrapper before the callback it points to.
        drop(std::mem::replace(&mut self.wrapper, UniquePtr::null()));
        unsafe { (self.drop_callback)(self.callback) };
    }
}

/// A `CaDiCaL` solver.
///
/// Methods that take a literal panic if it is `0` or `i32::MIN` (except
/// [`CaDiCal::add`] and [`CaDiCal::constrain`], where `0` ends the clause), and
/// methods with a state precondition panic when it does not hold. `CaDiCaL`
/// itself would abort the whole process in both cases.
///
/// One case is not checked: with the `factor` option (off by default) the
/// solver adds extension variables of its own, and passing one of those to a
/// method that takes a literal still aborts. Use
/// [`CaDiCal::declare_more_variables`] to reserve variables that the solver
/// will not use for this.
pub struct CaDiCal {
    // Declared first so the solver is destroyed before the callbacks it may call.
    solver: UniquePtr<ffi::Solver>,
    last_terminator: Option<Connected<ffi::Terminator>>,
    last_learner: Option<Connected<ffi::Learner>>,
    last_external_propagator: Option<Connected<ffi::ExternalPropagator>>,
    last_fixed_listener: Option<Connected<ffi::FixedAssignmentListener>>,
    last_tracer: Option<Connected<ffi::Tracer>>,
    // Whether the last proof file opened with `trace_proof*` is still open.
    proof_file_open: bool,
}

/// Cloning copies the formula and options (see [`CaDiCal::copy`]) but not any
/// connected callbacks; the clone starts with none connected. Like `copy`, it
/// panics if the solver is not READY, e.g. in the middle of adding a clause.
impl Clone for CaDiCal {
    fn clone(&self) -> Self {
        let mut r = Self::new();
        Self::copy(self, &mut r);
        r
    }
}

impl Drop for CaDiCal {
    fn drop(&mut self) {
        // CaDiCaL deletes tracers that are still connected when it is destroyed,
        // but ours is owned (and deleted) by `last_tracer`.
        self.disconnect_proof_tracer1();
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
            last_fixed_listener: None,
            last_tracer: None,
            proof_file_open: false,
        }
    }

    /// Core functionality as in the IPASIR incremental SAT solver interface.
    /// (recall 'READY = CONFIGURING | STEADY  | SATISFIED | UNSATISFIED | INCONCLUSIVE').
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
    #[track_caller]
    pub fn add(&mut self, lit: i32) {
        require_valid_lit_or_zero("add", lit);
        ffi::add(&mut self.solver, lit);
    }

    /// Here are functions simplifying clause addition. The given literals
    /// should all be valid (different from '`INT_MIN`' and different from '0').
    ///
    ///   require (VALID)
    ///   ensure (STEADY )
    ///
    #[inline]
    #[track_caller]
    pub fn clause1(&mut self, l1: i32) {
        require_valid_lit("clause1", l1);
        ffi::clause1(&mut self.solver, l1);
    }

    #[inline]
    #[track_caller]
    pub fn clause2(&mut self, l1: i32, l2: i32) {
        for lit in [l1, l2] {
            require_valid_lit("clause2", lit);
        }
        ffi::clause2(&mut self.solver, l1, l2);
    }

    #[inline]
    #[track_caller]
    pub fn clause3(&mut self, l1: i32, l2: i32, l3: i32) {
        for lit in [l1, l2, l3] {
            require_valid_lit("clause3", lit);
        }
        ffi::clause3(&mut self.solver, l1, l2, l3);
    }

    #[inline]
    #[track_caller]
    pub fn clause4(&mut self, l1: i32, l2: i32, l3: i32, l4: i32) {
        for lit in [l1, l2, l3, l4] {
            require_valid_lit("clause4", lit);
        }
        ffi::clause4(&mut self.solver, l1, l2, l3, l4);
    }

    #[inline]
    #[track_caller]
    pub fn clause5(&mut self, l1: i32, l2: i32, l3: i32, l4: i32, l5: i32) {
        for lit in [l1, l2, l3, l4, l5] {
            require_valid_lit("clause5", lit);
        }
        ffi::clause5(&mut self.solver, l1, l2, l3, l4, l5);
    }

    #[inline]
    #[track_caller]
    pub fn clause6(&mut self, v: &[i32]) {
        for &lit in v {
            require_valid_lit("clause6", lit);
        }
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
    #[track_caller]
    pub fn assume(&mut self, lit: i32) {
        require_valid_lit("assume", lit);
        ffi::assume(&mut self.solver, lit);
    }

    /// Try to solve the current formula.  Returns
    ///
    ///    0 = UNKNOWN      (limit reached or interrupted through 'terminate')
    ///   10 = SATISFIABLE
    ///   20 = UNSATISFIABLE
    ///
    ///   require (READY)
    ///   ensure (INCONCLUSIVE  | SATISFIED | UNSATISFIED)
    ///
    /// Note, that while in this call the solver actually transitions to state
    /// 'SOLVING', which however is only visible from a different context,
    /// i.e., from a different thread or from a signal handler.  Only right
    /// before returning from this call it goes into a 'READY' state.
    ///
    /// # Panics
    ///
    /// If the solver is in the middle of adding a clause or constraint (not READY). `CaDiCaL` would otherwise abort the process.
    ///
    #[inline]
    #[track_caller]
    pub fn solve(&mut self) -> Status {
        self.require_ready("solve");
        ffi::solve(&mut self.solver).into()
    }

    /// Get value (-lit=false, lit=true) of valid non-zero literal.
    ///
    ///   require (SATISFIED)
    ///   ensure (SATISFIED)
    ///
    /// # Panics
    ///
    /// If the solver is not in the `SATISFIED` state or `lit` is not a valid
    /// literal (zero or `i32::MIN`). `CaDiCaL` would otherwise abort the process.
    ///
    #[inline]
    #[track_caller]
    pub fn val(&mut self, lit: i32) -> i32 {
        self.require("val", State::SATISFIED, lit);
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
    /// # Panics
    ///
    /// If the solver is not in the `SATISFIED` state, `lit` is not a valid
    /// literal, or an external propagator is connected. `CaDiCaL` would
    /// otherwise abort the process.
    ///
    #[inline]
    #[track_caller]
    pub fn flip(&mut self, lit: i32) -> bool {
        self.require("flip", State::SATISFIED, lit);
        assert!(
            self.last_external_propagator.is_none(),
            "flip is not supported while an external propagator is connected"
        );
        ffi::flip(&mut self.solver, lit)
    }

    /// Same as 'flip' without actually flipping it. This functionality is
    /// currently not supported in the presence of an external propagator.
    ///
    ///   require (SATISFIED)
    ///   ensure (SATISFIED)
    ///
    /// # Panics
    ///
    /// If the solver is not in the `SATISFIED` state, `lit` is not a valid
    /// literal, or an external propagator is connected. `CaDiCaL` would
    /// otherwise abort the process.
    ///
    #[inline]
    #[track_caller]
    pub fn flippable(&mut self, lit: i32) -> bool {
        self.require("flippable", State::SATISFIED, lit);
        assert!(
            self.last_external_propagator.is_none(),
            "flippable is not supported while an external propagator is connected"
        );
        ffi::flippable(&mut self.solver, lit)
    }

    /// Determine whether the valid non-zero literal is in the core.
    /// Returns 'true' if the literal is in the core and 'false' otherwise.
    /// Note that the core does not have to be minimal.
    ///
    ///   require (UNSATISFIED)
    ///   ensure (UNSATISFIED)
    ///
    /// # Panics
    ///
    /// If the solver is not in the `UNSATISFIED` state or `lit` is not a valid
    /// literal (zero or `i32::MIN`). `CaDiCaL` would otherwise abort the process.
    ///
    #[inline]
    #[track_caller]
    pub fn failed(&mut self, lit: i32) -> bool {
        self.require("failed", State::UNSATISFIED, lit);
        ffi::failed(&mut self.solver, lit)
    }

    /// Add call-back which is checked regularly for termination.  There can
    /// only be one terminator connected.  If a second (non-zero) one is added
    /// the first one is implicitly disconnected.
    ///
    ///   require (VALID)
    ///   ensure (VALID)
    ///
    /// The solver takes ownership of `terminator` and drops it when it is
    /// disconnected, replaced, or the solver is dropped. To read its state
    /// afterwards, pass an `Rc<RefCell<_>>` and keep a clone of the `Rc`.
    ///
    /// The callback must be `'static`, so it cannot borrow from a value that
    /// might be dropped while the solver still calls it:
    ///
    /// ```compile_fail,E0597
    /// # use cadical_sys::{CaDiCal, Terminator};
    /// # struct Cb<'a>(&'a mut u32);
    /// # impl Terminator for Cb<'_> { fn terminated(&mut self) -> bool { false } }
    /// let mut solver = CaDiCal::new();
    /// let mut calls = 0;
    /// solver.connect_terminator(Cb(&mut calls)); // error: `calls` does not live long enough
    /// ```
    ///
    #[inline]
    pub fn connect_terminator<T: Terminator + 'static>(&mut self, terminator: T) {
        fn f<T: Terminator>(state: *mut u8) -> bool {
            let ptr: *mut T = state.cast::<T>();
            let i = unsafe { &mut *ptr };
            i.terminated()
        }
        let mut terminator = Connected::new(terminator, |state| unsafe {
            ffi::new_terminator(state, f::<T>)
        });
        ffi::connect_terminator(&mut self.solver, &mut terminator.wrapper);
        // Only drop the previous terminator once the solver no longer points to it.
        self.last_terminator = Some(terminator);
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
    /// The solver takes ownership of `learner` and drops it when it is
    /// disconnected, replaced, or the solver is dropped. To read its state
    /// afterwards, pass an `Rc<RefCell<_>>` and keep a clone of the `Rc`.
    ///
    /// The callback must be `'static`, so it cannot borrow from a value that
    /// might be dropped while the solver still calls it:
    ///
    /// ```compile_fail,E0597
    /// # use cadical_sys::{CaDiCal, Learner};
    /// # struct Cb<'a>(&'a mut u32);
    /// # impl Learner for Cb<'_> { fn learning(&mut self, _: i32) -> bool { false } fn learn(&mut self, _: i32) {} }
    /// let mut solver = CaDiCal::new();
    /// let mut calls = 0;
    /// solver.connect_learner(Cb(&mut calls)); // error: `calls` does not live long enough
    /// ```
    ///
    #[inline]
    pub fn connect_learner<T: Learner + 'static>(&mut self, learner: T) {
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

        let mut learner = Connected::new(learner, |state| unsafe {
            ffi::new_learner(state, learning::<T>, learn::<T>)
        });
        ffi::connect_learner(&mut self.solver, &mut learner.wrapper);
        self.last_learner = Some(learner);
    }

    #[inline]
    pub fn disconnect_learner(&mut self) {
        ffi::disconnect_learner(&mut self.solver);
        self.last_learner = None;
    }

    /// Add call-back which allows to observe when a variable is fixed.
    ///
    ///   require (VALID)
    ///   ensure (VALID)
    ///
    /// The solver takes ownership of `fixed_listener` and drops it when it is
    /// disconnected, replaced, or the solver is dropped. To read its state
    /// afterwards, pass an `Rc<RefCell<_>>` and keep a clone of the `Rc`.
    ///
    /// The callback must be `'static`, so it cannot borrow from a value that
    /// might be dropped while the solver still calls it:
    ///
    /// ```compile_fail,E0597
    /// # use cadical_sys::{CaDiCal, FixedAssignmentListener};
    /// # struct Cb<'a>(&'a mut u32);
    /// # impl FixedAssignmentListener for Cb<'_> { fn notify_fixed_assignment(&mut self, _: i32) {} }
    /// let mut solver = CaDiCal::new();
    /// let mut calls = 0;
    /// solver.connect_fixed_listener(Cb(&mut calls)); // error: `calls` does not live long enough
    /// ```
    ///
    pub fn connect_fixed_listener<F: FixedAssignmentListener + 'static>(
        &mut self,
        fixed_listener: F,
    ) {
        fn notify_fixed_assignment<F: FixedAssignmentListener>(state: *mut u8, lit: i32) {
            let ptr: *mut F = state.cast::<F>();
            let i = unsafe { &mut *ptr };
            i.notify_fixed_assignment(lit);
        }

        let mut listener = Connected::new(fixed_listener, |state| unsafe {
            ffi::new_fixed_assignment_listener(state, notify_fixed_assignment::<F>)
        });
        ffi::connect_fixed_listener(&mut self.solver, &mut listener.wrapper);
        self.last_fixed_listener = Some(listener);
    }

    #[inline]
    pub fn disconnect_fixed_listener(&mut self) {
        ffi::disconnect_fixed_listener(&mut self.solver);
        self.last_fixed_listener = None;
    }

    /// Add call-back which allows to learn, propagate and backtrack based on
    /// external constraints. Only one external propagator can be connected
    /// and after connection every related variables must be 'observed' (use
    /// '`add_observed_var`' function).
    /// Disconnection of the external propagator resets all the observed
    /// variables.
    ///
    ///   require (VALID)
    ///   ensure (VALID)
    ///
    /// The solver takes ownership of `propagator` and drops it when it is
    /// disconnected, replaced, or the solver is dropped. To read its state
    /// afterwards, pass an `Rc<RefCell<_>>` and keep a clone of the `Rc`.
    /// Connecting a propagator disconnects any propagator connected earlier,
    /// which also resets the observed variables.
    ///
    pub fn connect_external_propagator<T: ExternalPropagator + 'static>(
        &mut self,
        mut propagator: T,
    ) {
        fn notify_assignment<T: ExternalPropagator>(state: *mut u8, x: &[i32]) {
            let ptr: *mut T = state.cast::<T>();
            let i = unsafe { &mut *ptr };
            i.notify_assignment(x);
        }
        fn notify_new_decision_level<T: ExternalPropagator>(state: *mut u8) {
            let ptr: *mut T = state.cast::<T>();
            let i = unsafe { &mut *ptr };
            i.notify_new_decision_level();
        }
        fn notify_backtrack<T: ExternalPropagator>(state: *mut u8, x: usize) {
            let ptr: *mut T = state.cast::<T>();
            let i = unsafe { &mut *ptr };
            i.notify_backtrack(x);
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
        // CaDiCaL aborts when connecting a second propagator.
        self.disconnect_external_propagator();

        let is_lazy = propagator.is_lazy();
        let are_reasons_forgettable = propagator.are_reasons_forgettable();

        let mut external_propagator = Connected::new(propagator, |state| unsafe {
            ffi::new_external_propagator(
                state,
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
        });
        ffi::connect_external_propagator(&mut self.solver, &mut external_propagator.wrapper);
        self.last_external_propagator = Some(external_propagator);
    }
    pub fn disconnect_external_propagator(&mut self) {
        // CaDiCaL 3 aborts when disconnecting without a connected propagator.
        // `_propagator` is only dropped after CaDiCaL has let go of it.
        if let Some(_propagator) = self.last_external_propagator.take() {
            ffi::disconnect_external_propagator(&mut self.solver);
        }
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
    /// # Panics
    ///
    /// If no external propagator is connected. `CaDiCaL` would otherwise
    /// abort the process.
    ///
    #[inline]
    #[track_caller]
    pub fn add_observed_var(&mut self, var: i32) {
        require_valid_lit("add_observed_var", var);
        self.require_propagator("add_observed_var");
        ffi::add_observed_var(&mut self.solver, var);
    }

    /// Removes the 'observed' flag from the given variable. A variable can be
    /// set unobserved only between solve calls, not during it (to guarantee
    /// that no yet unexplained external propagation involves it).
    ///
    ///   require (VALID)
    ///   ensure (VALID)
    ///
    /// # Panics
    ///
    /// If no external propagator is connected. `CaDiCaL` would otherwise
    /// abort the process.
    ///
    #[inline]
    #[track_caller]
    pub fn remove_observed_var(&mut self, var: i32) {
        require_valid_lit("remove_observed_var", var);
        self.require_propagator("remove_observed_var");
        ffi::remove_observed_var(&mut self.solver, var);
    }

    /// Removes all the 'observed' flags from the variables. Disconnecting the
    /// propagator invokes this step as well.
    ///
    ///   require (VALID)
    ///   ensure (VALID)
    ///
    /// # Panics
    ///
    /// If no external propagator is connected. `CaDiCaL` would otherwise
    /// abort the process.
    ///
    #[inline]
    #[track_caller]
    pub fn reset_observed_vars(&mut self) {
        self.require_propagator("reset_observed_vars");
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
    #[track_caller]
    pub fn is_decision(&mut self, lit: i32) -> bool {
        require_valid_lit("is_decision", lit);
        ffi::is_decision(&mut self.solver, lit)
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
    #[track_caller]
    pub fn constrain(&mut self, lit: i32) {
        require_valid_lit_or_zero("constrain", lit);
        ffi::constrain(&mut self.solver, lit);
    }

    /// Determine whether the constraint was used to proof the
    /// unsatisfiability. Note that the formula might still be unsatisfiable
    /// without the constraint.
    ///
    ///   require (UNSATISFIED)
    ///   ensure (UNSATISFIED)
    ///
    /// # Panics
    ///
    /// If the solver is not in the `UNSATISFIED` state. `CaDiCaL` would otherwise abort the process.
    ///
    #[inline]
    #[track_caller]
    pub fn constraint_failed(&mut self) -> bool {
        self.require_state("constraint_failed", State::UNSATISFIED);
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
    /// # Panics
    ///
    /// If the solver is in the middle of adding a clause or constraint (not
    /// READY), which `CaDiCaL` would silently cut short.
    ///
    #[inline]
    #[track_caller]
    pub fn lookahead(&mut self) -> i32 {
        self.require_ready("lookahead");
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

    /// Panics where `CaDiCaL` would abort the process on a contract violation,
    /// so misuse of the safe API is reported as a catchable Rust panic.
    #[track_caller]
    fn require(&self, function: &str, expected: State, lit: i32) {
        self.require_state(function, expected);
        require_valid_lit(function, lit);
    }

    #[track_caller]
    fn require_state(&self, function: &str, expected: State) {
        let state = self.state();
        assert!(
            state == expected,
            "{function} requires the solver to be in state {expected:?}, but it is in state {state:?}"
        );
    }

    #[track_caller]
    fn require_ready(&self, function: &str) {
        let state = self.state();
        assert!(
            (state as i32) & (State::READY as i32) != 0,
            "{function} requires the solver to be READY, but it is in state {state:?}"
        );
    }

    #[track_caller]
    fn require_one_of(&self, function: &str, expected: &[State]) {
        let state = self.state();
        assert!(
            expected.contains(&state),
            "{function} requires the solver to be in one of the states {expected:?}, but it is in state {state:?}"
        );
    }

    #[track_caller]
    fn require_proof_file(&self, function: &str) {
        assert!(
            self.proof_file_open,
            "{function} requires an open proof file (see trace_proof1 and trace_proof2)"
        );
    }

    #[track_caller]
    fn require_propagator(&self, function: &str) {
        assert!(
            self.last_external_propagator.is_some(),
            "{function} requires a connected external propagator"
        );
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

    /// return the name and version of the incremental SAT solving library
    #[must_use]
    #[inline]
    pub fn signature() -> String {
        ffi::signature()
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
    /// # Panics
    ///
    /// If `source` is not READY (e.g. a clause was started with
    /// [`CaDiCal::add`] but not terminated) or `destination` is not in the
    /// `CONFIGURING` state. `CaDiCaL` would otherwise abort the process.
    ///
    #[inline]
    #[track_caller]
    pub fn copy(source: &CaDiCal, destination: &mut CaDiCal) {
        source.require_ready("copy");
        destination.require_state("copy", State::CONFIGURING);
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
    /// # Panics
    ///
    /// If the solver is in the middle of adding a clause or constraint (not
    /// READY), which `CaDiCaL` would silently cut short.
    ///
    #[inline]
    #[track_caller]
    pub fn resize(&mut self, min_max_var: i32) {
        self.require_ready("resize");
        ffi::resize(&mut self.solver, min_max_var);
    }

    /// Increase the maximum variable index by a number of new variables.
    /// initializes '`number_of_additional_new_vars`' new variables and protects
    /// them from being used by the solver as extension variables (BVA).
    ///
    /// It returns the new maximum variable index which is the highest
    /// variable name of the consecutive range of newly declared variables.
    /// It has the same state transition and conditions as 'resize' above.
    ///
    ///   require (READY)
    ///   ensure (STEADY)
    ///
    /// # Panics
    ///
    /// If the solver is in the middle of adding a clause or constraint (not
    /// READY), which `CaDiCaL` would silently cut short, or if
    /// `number_of_additional_new_vars` is negative or would overflow the
    /// variable index.
    ///
    #[inline]
    #[track_caller]
    pub fn declare_more_variables(&mut self, number_of_additional_new_vars: i32) -> i32 {
        self.require_ready("declare_more_variables");
        let vars = self.vars();
        assert!(
            number_of_additional_new_vars >= 0
                && vars.checked_add(number_of_additional_new_vars).is_some(),
            "declare_more_variables called with {number_of_additional_new_vars} variables, but {vars} already exist"
        );
        ffi::declare_more_variables(&mut self.solver, number_of_additional_new_vars)
    }

    /// Returns the next fresh variable that was not used internally.
    ///
    /// # Panics
    ///
    /// If the solver is in the middle of adding a clause or constraint (not
    /// READY), which `CaDiCaL` would silently cut short.
    ///
    #[inline]
    #[track_caller]
    pub fn declare_one_more_variable(&mut self) -> i32 {
        self.require_ready("declare_one_more_variable");
        ffi::declare_one_more_variable(&mut self.solver)
    }

    /// Get the value of some statistics or -1 if the statistics does not
    /// exist or is not supported. Only requires the state to be initialized.
    ///
    #[must_use]
    #[inline]
    pub fn get_statistic_value(&self, name: &str) -> i64 {
        ffi::get_statistic_value(&self.solver, name.to_owned())
    }

    /// Renamed to [`CaDiCal::resize`] in `CaDiCaL` 3.
    #[deprecated(since = "0.8.0", note = "renamed to `resize` in CaDiCaL 3")]
    #[inline]
    pub fn reserve(&mut self, min_max_var: i32) {
        self.resize(min_max_var);
    }

    // pub fn `trace_api_calls(&mut` self, file: String);

    //------------------------------------------------------------------------
    /// Option handling.
    /// Determine whether 'name' is a valid option name.
    ///
    #[must_use]
    #[inline]
    pub fn is_valid_option(name: &str) -> bool {
        ffi::is_valid_option(name.to_owned())
    }

    /// Determine whether 'name' enables a specific preprocessing technique.
    ///
    #[must_use]
    #[inline]
    pub fn is_preprocessing_option(name: &str) -> bool {
        ffi::is_preprocessing_option(name.to_owned())
    }

    /// Determine whether 'arg' is a valid long option of the form `--<name>`,
    /// `--<name>=<val>` or `--no-<name>` similar to '`set_long_option`' below.
    /// Legal values are 'true', 'false', or `[-]<mantissa>[e<exponent>]`.
    #[must_use]
    #[inline]
    pub fn is_valid_long_option(arg: &str) -> bool {
        ffi::is_valid_long_option(arg.to_owned())
    }

    /// Get the current value of the option 'name'.  If 'name' is invalid then
    /// zero is returned.  Here '--...' arguments as invalid options.
    ///
    #[inline]
    pub fn get(&mut self, name: &str) -> i32 {
        ffi::get(&mut self.solver, name.to_owned())
    }

    /// Set the default verbose message prefix (default "c ").
    ///
    #[inline]
    pub fn prefix(&mut self, verbose_message_prefix: &str) {
        ffi::prefix(&mut self.solver, verbose_message_prefix.to_owned());
    }

    /// Explicit version of setting an option.  If the option `<name>` exists
    /// and `<val>` can be parsed then 'true' is returned.  If the option value
    /// is out of range the actual value is computed as the closest (minimum or
    /// maximum) value possible, but still 'true' is returned.
    ///
    ///   require (CONFIGURING)
    ///   ensure (CONFIGURING)
    ///
    /// Thus options can only bet set right after initialization.
    ///
    /// # Panics
    ///
    /// If the solver is not in the `CONFIGURING` state, unless `name` is one of
    /// `log`, `quiet`, `report` and `verbose`. `CaDiCaL` would otherwise abort the process.
    ///
    #[inline]
    #[track_caller]
    pub fn set(&mut self, name: &str, val: i32) -> bool {
        if !matches!(name, "log" | "quiet" | "report" | "verbose") {
            self.require_state("set", State::CONFIGURING);
        }
        ffi::set(&mut self.solver, name.to_owned(), val)
    }

    /// This function accepts options in command line syntax:
    ///
    ///   `--<name>=<val>`, `--<name>`, or `--no-<name>`
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
    /// # Panics
    ///
    /// If the solver is not in the `CONFIGURING` state. `CaDiCaL` would otherwise abort the process.
    ///
    #[inline]
    #[track_caller]
    pub fn set_long_option(&mut self, arg: &str) -> bool {
        self.require_state("set_long_option", State::CONFIGURING);
        ffi::set_long_option(&mut self.solver, arg.to_owned())
    }

    /// Determine whether 'name' is a valid configuration.
    ///
    #[must_use]
    #[inline]
    pub fn is_valid_configuration(name: &str) -> bool {
        ffi::is_valid_configuration(name.to_owned())
    }

    /// Overwrite (some) options with the forced values of the configuration.
    /// The result is 'true' iff the 'name' is a valid configuration.
    ///
    ///   require (CONFIGURING)
    ///   ensure (CONFIGURING)
    ///
    /// # Panics
    ///
    /// If the solver is not in the `CONFIGURING` state. `CaDiCaL` would otherwise abort the process.
    ///
    #[inline]
    #[track_caller]
    pub fn configure(&mut self, name: &str) -> bool {
        self.require_state("configure", State::CONFIGURING);
        ffi::configure(&mut self.solver, name.to_owned())
    }

    /// Increase preprocessing and inprocessing limits by `10^<val>`.  Values
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
    pub fn limit(&mut self, arg: &str, val: i32) -> bool {
        ffi::limit(&mut self.solver, arg.to_owned(), val)
    }
    #[inline]
    pub fn is_valid_limit(&mut self, arg: &str) -> bool {
        ffi::is_valid_limit(&mut self.solver, arg.to_owned())
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
    ///   ensure (INCONCLUSIVE | SATISFIED | UNSATISFIED)
    ///
    /// # Panics
    ///
    /// If the solver is in the middle of adding a clause or constraint (not READY),
    /// or if `rounds` is negative. `CaDiCaL` would otherwise abort the process.
    ///
    #[inline]
    #[track_caller]
    pub fn simplify(&mut self, rounds: i32) -> Status {
        self.require_ready("simplify");
        assert!(rounds >= 0, "simplify called with negative rounds {rounds}");
        ffi::simplify(&mut self.solver, rounds).into()
    }

    /// This function executes the given number of preprocessing rounds
    /// without search: it only performs unit propagation by assuming the
    /// currently defined (potentially empty) set of assumptions. In case unit
    /// propagation leads to a conflict it returns `UNSATISFIABLE`. Otherwise
    /// it usually returns `UNKNOWN`, and the implied literals can be queried
    /// with [`CaDiCal::implied`]. In rare cases (no decision needed) it
    /// returns `SATISFIABLE`.
    ///
    ///   require (READY)
    ///   ensure (INCONCLUSIVE | SATISFIED | UNSATISFIED)
    ///
    /// # Panics
    ///
    /// If the solver is in the middle of adding a clause or constraint (not
    /// READY), which `CaDiCaL` would silently cut short.
    ///
    #[inline]
    #[track_caller]
    pub fn propagate(&mut self) -> Status {
        self.require_ready("propagate");
        ffi::propagate(&mut self.solver).into()
    }

    /// The literals implied by the last call to [`CaDiCal::propagate`].
    ///
    ///   require (INCONCLUSIVE | SATISFIED)
    ///   ensure (INCONCLUSIVE | SATISFIED)
    ///
    /// # Panics
    ///
    /// If the solver is not in the `INCONCLUSIVE` or `SATISFIED` state.
    /// `CaDiCaL` would otherwise abort the process.
    ///
    #[must_use]
    #[track_caller]
    pub fn implied(&mut self) -> Vec<i32> {
        self.require_one_of("implied", &[State::INCONCLUSIVE, State::SATISFIED]);
        let mut implicants = Vec::new();
        ffi::implied(&mut self.solver, &mut implicants);
        implicants
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
    #[track_caller]
    pub fn frozen(&self, lit: i32) -> bool {
        require_valid_lit("frozen", lit);
        ffi::frozen(&self.solver, lit)
    }

    #[inline]
    #[track_caller]
    pub fn freeze(&mut self, lit: i32) {
        require_valid_lit("freeze", lit);
        ffi::freeze(&mut self.solver, lit);
    }

    /// Undo one [`CaDiCal::freeze`] of `lit`.
    ///
    /// # Panics
    ///
    /// If `lit` is not frozen. `CaDiCaL` would otherwise abort the process.
    ///
    #[inline]
    #[track_caller]
    pub fn melt(&mut self, lit: i32) {
        require_valid_lit("melt", lit);
        assert!(
            self.frozen(lit),
            "melt called on literal {lit}, which is not frozen"
        );
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
    #[track_caller]
    pub fn fixed(&self, lit: i32) -> i32 {
        require_valid_lit("fixed", lit);
        ffi::fixed(&self.solver, lit)
    }

    //------------------------------------------------------------------------
    /// Force the default decision phase of a variable to a certain value.
    ///
    #[inline]
    #[track_caller]
    pub fn phase(&mut self, lit: i32) {
        require_valid_lit("phase", lit);
        ffi::phase(&mut self.solver, lit);
    }

    #[inline]
    #[track_caller]
    pub fn unphase(&mut self, lit: i32) {
        require_valid_lit("unphase", lit);
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
    /// Write DRAT proof. `name` is ignored. As with [`CaDiCal::trace_proof2`],
    /// a `.gz`, `.bz2`, `.xz` or `.7z` suffix compresses the proof through
    /// the external program of that name, and `false` is returned if it is
    /// not in `PATH`.
    ///
    /// # Panics
    ///
    /// If the solver is not in the `CONFIGURING` state. `CaDiCaL` would
    /// otherwise abort the process.
    ///
    #[inline]
    #[track_caller]
    pub fn trace_proof1(&mut self, file: &str, name: &str) -> bool {
        self.require_state("trace_proof1", State::CONFIGURING);
        let opened = ffi::trace_proof1(&mut self.solver, file.to_owned(), name.to_owned());
        self.proof_file_open |= opened;
        opened
    }

    /// Open & write proof. A `.gz`, `.bz2`, `.xz` or `.7z` suffix compresses
    /// the proof through the external program of that name; `false` is
    /// returned if it is not in `PATH` or the file cannot be written.
    ///
    /// # Panics
    ///
    /// If the solver is not in the `CONFIGURING` state. `CaDiCaL` would
    /// otherwise abort the process.
    ///
    #[inline]
    #[track_caller]
    pub fn trace_proof2(&mut self, path: &str) -> bool {
        self.require_state("trace_proof2", State::CONFIGURING);
        let opened = ffi::trace_proof2(&mut self.solver, path.to_owned());
        self.proof_file_open |= opened;
        opened
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
    /// # Panics
    ///
    /// If no proof file is open (see [`CaDiCal::trace_proof2`]). `CaDiCaL`
    /// would otherwise abort the process.
    ///
    #[inline]
    #[track_caller]
    pub fn flush_proof_trace(&mut self, print: bool) {
        self.require_proof_file("flush_proof_trace");
        ffi::flush_proof_trace(&mut self.solver, print);
    }

    /// Close proof trace early.  Similar to 'flush' we allow the user to
    /// control with 'print' in a more fine-grained way whether statistics
    /// about the size of the written proof file and if compressed on-the-fly
    /// the number of actual bytes written (including deflation percentage) are
    /// printed.  Before actually closing (or detaching in case of writing to
    /// `<stdout>`) we check whether '`flush_proof_trace`' was called since the
    /// last time a proof step (addition or deletion) was traced.  If this is
    /// not the case we would call '`flush_proof_trace`' with the same 'print'
    /// argument.
    ///
    ///   require (VALID)
    ///   ensure (VALID)
    ///
    /// # Panics
    ///
    /// If no proof file is open, including when it was already closed.
    /// `CaDiCaL` would otherwise abort the process.
    ///
    #[inline]
    #[track_caller]
    pub fn close_proof_trace(&mut self, print: bool) {
        self.require_proof_file("close_proof_trace");
        self.proof_file_open = false;
        ffi::close_proof_trace(&mut self.solver, print);
    }

    /// Enables clausal proof tracing with or without antecedents using
    /// the Tracer interface defined in 'tracer.hpp'.
    ///
    /// `CaDiCaL`'s built-in `FileTracer` and `StatTracer` are not exposed
    /// directly; use [`CaDiCal::trace_proof1`] or [`CaDiCal::trace_proof2`]
    /// to write a proof file.
    ///
    ///   require (CONFIGURING)
    ///   ensure (CONFIGURING)
    ///
    /// The solver takes ownership of `tracer` and drops it when it is
    /// disconnected, replaced, or the solver is dropped. To read its state
    /// afterwards, pass an `Rc<RefCell<_>>` and keep a clone of the `Rc`.
    /// Connecting a tracer disconnects any tracer connected earlier.
    ///
    /// [`ProofTracer::finalize_clause`] is only called when `finalize_clauses`
    /// is true.
    ///
    /// # Panics
    ///
    /// If the solver is not in the `CONFIGURING` state, i.e. clauses have
    /// already been added. `CaDiCaL` would otherwise abort the process.
    ///
    #[allow(clippy::too_many_lines)]
    #[track_caller]
    pub fn connect_proof_tracer1<T: ProofTracer + 'static>(
        &mut self,
        tracer: T,
        antecedents: bool,
        finalize_clauses: bool,
    ) {
        fn add_original_clause<T: ProofTracer>(
            state: *mut u8,
            id: i64,
            redundant: bool,
            clause: &[i32],
            restored: bool,
        ) {
            let ptr: *mut T = state.cast::<T>();
            let t = unsafe { &mut *ptr };
            t.add_original_clause(id, redundant, clause, restored);
        }

        fn add_derived_clause<T: ProofTracer>(
            state: *mut u8,
            id: i64,
            redundant: bool,
            witness: i32,
            clause: &[i32],
            antecedents: &[i64],
        ) {
            let ptr: *mut T = state.cast::<T>();
            let t = unsafe { &mut *ptr };
            t.add_derived_clause(id, redundant, witness, clause, antecedents);
        }

        fn delete_clause<T: ProofTracer>(state: *mut u8, id: i64, redundant: bool, clause: &[i32]) {
            let ptr: *mut T = state.cast::<T>();
            let t = unsafe { &mut *ptr };
            t.delete_clause(id, redundant, clause);
        }

        fn weaken_minus<T: ProofTracer>(state: *mut u8, id: i64, clause: &[i32]) {
            let ptr: *mut T = state.cast::<T>();
            let t = unsafe { &mut *ptr };
            t.weaken_minus(id, clause);
        }

        fn strengthen<T: ProofTracer>(state: *mut u8, id: i64) {
            let ptr: *mut T = state.cast::<T>();
            let t = unsafe { &mut *ptr };
            t.strengthen(id);
        }

        fn finalize_clause<T: ProofTracer>(state: *mut u8, id: i64, clause: &[i32]) {
            let ptr: *mut T = state.cast::<T>();
            let t = unsafe { &mut *ptr };
            t.finalize_clause(id, clause);
        }

        fn add_assumption<T: ProofTracer>(state: *mut u8, lit: i32) {
            let ptr: *mut T = state.cast::<T>();
            let t = unsafe { &mut *ptr };
            t.add_assumption(lit);
        }

        fn add_constraint<T: ProofTracer>(state: *mut u8, clause: &[i32]) {
            let ptr: *mut T = state.cast::<T>();
            let t = unsafe { &mut *ptr };
            t.add_constraint(clause);
        }

        fn reset_assumptions<T: ProofTracer>(state: *mut u8) {
            let ptr: *mut T = state.cast::<T>();
            let t = unsafe { &mut *ptr };
            t.reset_assumptions();
        }

        fn add_assumption_clause<T: ProofTracer>(
            state: *mut u8,
            id: i64,
            clause: &[i32],
            antecedents: &[i64],
        ) {
            let ptr: *mut T = state.cast::<T>();
            let t = unsafe { &mut *ptr };
            t.add_assumption_clause(id, clause, antecedents);
        }

        fn conclude_sat<T: ProofTracer>(state: *mut u8, conclusion_type: i32, model: &[i32]) {
            let ptr: *mut T = state.cast::<T>();
            let t = unsafe { &mut *ptr };
            t.conclude_sat(conclusion_type, model);
        }

        fn conclude_unsat<T: ProofTracer>(
            state: *mut u8,
            conclusion_type: i32,
            clause_ids: &[i64],
        ) {
            let ptr: *mut T = state.cast::<T>();
            let t = unsafe { &mut *ptr };
            t.conclude_unsat(conclusion_type, clause_ids);
        }

        fn conclude_unknown<T: ProofTracer>(state: *mut u8, trail: &[i32]) {
            let ptr: *mut T = state.cast::<T>();
            let t = unsafe { &mut *ptr };
            t.conclude_unknown(trail);
        }

        fn demote_clause<T: ProofTracer>(state: *mut u8, id: i64, clause: &[i32]) {
            let ptr: *mut T = state.cast::<T>();
            let t = unsafe { &mut *ptr };
            t.demote_clause(id, clause);
        }

        fn report_status<T: ProofTracer>(state: *mut u8, status: i32, id: i64) {
            let ptr: *mut T = state.cast::<T>();
            let t = unsafe { &mut *ptr };
            t.report_status(status, id);
        }

        fn begin_proof<T: ProofTracer>(state: *mut u8, first_derived_id: i64) {
            let ptr: *mut T = state.cast::<T>();
            let t = unsafe { &mut *ptr };
            t.begin_proof(first_derived_id);
        }

        fn solve_query<T: ProofTracer>(state: *mut u8) {
            let ptr: *mut T = state.cast::<T>();
            let t = unsafe { &mut *ptr };
            t.solve_query();
        }

        fn notify_equivalence<T: ProofTracer>(state: *mut u8, lit: i32, other: i32) {
            let ptr: *mut T = state.cast::<T>();
            let t = unsafe { &mut *ptr };
            t.notify_equivalence(lit, other);
        }

        let solver_state = self.state();
        assert!(
            solver_state == State::CONFIGURING,
            "connect_proof_tracer1 requires the solver to be in state CONFIGURING, but it is in state {solver_state:?}"
        );
        // Unlike the other callbacks, CaDiCaL keeps every connected tracer, so
        // the previous one must be disconnected before it is dropped.
        self.disconnect_proof_tracer1();

        let mut tracer = Connected::new(tracer, |state| unsafe {
            ffi::new_tracer(
                state,
                add_original_clause::<T>,
                add_derived_clause::<T>,
                delete_clause::<T>,
                weaken_minus::<T>,
                strengthen::<T>,
                finalize_clause::<T>,
                add_assumption::<T>,
                add_constraint::<T>,
                reset_assumptions::<T>,
                add_assumption_clause::<T>,
                conclude_sat::<T>,
                conclude_unsat::<T>,
                conclude_unknown::<T>,
                demote_clause::<T>,
                report_status::<T>,
                begin_proof::<T>,
                solve_query::<T>,
                notify_equivalence::<T>,
            )
        });
        ffi::connect_proof_tracer1(
            &mut self.solver,
            &mut tracer.wrapper,
            antecedents,
            finalize_clauses,
        );
        self.last_tracer = Some(tracer);
    }

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
    /// # Panics
    ///
    /// If the solver has not finished a `solve` (it is not `SATISFIED`,
    /// `UNSATISFIED` or `INCONCLUSIVE`). `CaDiCaL` would otherwise abort the process.
    ///
    #[inline]
    #[track_caller]
    pub fn conclude(&mut self) {
        self.require_one_of(
            "conclude",
            &[State::SATISFIED, State::UNSATISFIED, State::INCONCLUSIVE],
        );
        ffi::conclude(&mut self.solver);
    }

    /// Disconnect the proof tracer connected with
    /// [`CaDiCal::connect_proof_tracer1`] and drop it. Does nothing if no
    /// tracer is connected.
    ///
    ///   require (VALID)
    ///   ensure (VALID)
    ///
    pub fn disconnect_proof_tracer1(&mut self) {
        if let Some(mut tracer) = self.last_tracer.take() {
            ffi::disconnect_proof_tracer1(&mut self.solver, &mut tracer.wrapper);
        }
    }

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
        fn f<I: WitnessIterator>(state: *mut u8, clause: &[i32], witness: &[i32], id: i64) -> bool {
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
        fn f<I: WitnessIterator>(state: *mut u8, clause: &[i32], witness: &[i32], id: i64) -> bool {
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
    /// Returns `Err` with `CaDiCaL`'s error message if unsuccessful.
    ///
    ///   require (VALID)
    ///   ensure (VALID)
    ///
    /// # Errors
    ///
    /// If the file cannot be opened or parsed.
    ///
    /// # Panics
    ///
    /// If the solver is not in the `CONFIGURING` state, i.e. clauses have
    /// already been added. `CaDiCaL` would otherwise abort the process.
    ///
    #[inline]
    #[track_caller]
    pub fn read_dimacs1(
        &mut self,
        file: &str,
        name: &str,
        vars: &mut i32,
        strict: i32,
    ) -> Result<(), String> {
        self.require_state("read_dimacs1", State::CONFIGURING);
        into_result(ffi::read_dimacs1(
            &mut self.solver,
            file.to_owned(),
            name.to_owned(),
            vars,
            strict,
        ))
    }

    /// # Errors
    ///
    /// If the file cannot be opened or parsed.
    ///
    /// # Panics
    ///
    /// If the solver is not in the `CONFIGURING` state, i.e. clauses have
    /// already been added. `CaDiCaL` would otherwise abort the process.
    ///
    #[inline]
    #[track_caller]
    pub fn read_dimacs2(&mut self, path: &str, vars: &mut i32, strict: i32) -> Result<(), String> {
        self.require_state("read_dimacs2", State::CONFIGURING);
        into_result(ffi::read_dimacs2(
            &mut self.solver,
            path.to_owned(),
            vars,
            strict,
        ))
    }

    /// The following routines work the same way but parse both DIMACS and
    /// INCCNF files (with 'p inccnf' header and `a <cube>` lines).  If the
    /// parser finds and 'p inccnf' header or cubes then '*incremental' is set
    /// to true and the cubes are stored in the given vector (each cube
    /// terminated by a zero).
    ///
    /// # Errors
    ///
    /// If the file cannot be opened or parsed.
    ///
    /// # Panics
    ///
    /// If the solver is not in the `CONFIGURING` state, i.e. clauses have
    /// already been added. `CaDiCaL` would otherwise abort the process.
    ///
    #[inline]
    #[track_caller]
    pub fn read_dimacs3(
        &mut self,
        file: &str,
        name: &str,
        vars: &mut i32,
        strict: i32,
        incremental: &mut bool,
        cubes: &mut Vec<i32>,
    ) -> Result<(), String> {
        self.require_state("read_dimacs3", State::CONFIGURING);
        into_result(ffi::read_dimacs3(
            &mut self.solver,
            file.to_owned(),
            name.to_owned(),
            vars,
            strict,
            incremental,
            cubes,
        ))
    }

    /// # Errors
    ///
    /// If the file cannot be opened or parsed.
    ///
    /// # Panics
    ///
    /// If the solver is not in the `CONFIGURING` state, i.e. clauses have
    /// already been added. `CaDiCaL` would otherwise abort the process.
    ///
    #[inline]
    #[track_caller]
    pub fn read_dimacs4(
        &mut self,
        path: &str,
        vars: &mut i32,
        strict: i32,
        incremental: &mut bool,
        cubes: &mut Vec<i32>,
    ) -> Result<(), String> {
        self.require_state("read_dimacs4", State::CONFIGURING);
        into_result(ffi::read_dimacs4(
            &mut self.solver,
            path.to_owned(),
            vars,
            strict,
            incremental,
            cubes,
        ))
    }

    //------------------------------------------------------------------------
    /// Write current irredundant clauses and all derived unit clauses
    /// to a file in DIMACS format.  Clauses on the extension stack are
    /// not included, nor any redundant clauses.
    ///
    /// The '`min_max_var`' parameter gives a lower bound on the number `<vars>`
    /// of variables used in the DIMACS `p cnf <vars> ...` header.
    ///
    /// Returns `Err` with `CaDiCaL`'s error message if unsuccessful.
    ///
    ///   require (VALID)
    ///   ensure (VALID)
    ///
    /// # Errors
    ///
    /// If the file cannot be written.
    ///
    #[inline]
    pub fn write_dimacs(&mut self, path: &str, min_max_var: i32) -> Result<(), String> {
        into_result(ffi::write_dimacs(
            &mut self.solver,
            path.to_owned(),
            min_max_var,
        ))
    }

    /// The extension stack for reconstruction a solution can be written too.
    ///
    /// # Errors
    ///
    /// If the file cannot be written.
    ///
    #[inline]
    pub fn write_extension(&mut self, path: &str) -> Result<(), String> {
        into_result(ffi::write_extension(&mut self.solver, path.to_owned()))
    }

    /// Print build configuration to a file with prefix 'c '.  If the file
    /// is `<stdout>` or `<stderr>` then terminal color codes might be used.
    ///
    /// # Errors
    ///
    /// If the file cannot be opened.
    ///
    #[inline]
    pub fn build(file: &str, prefix: &str) -> Result<(), String> {
        into_result(ffi::build(file.to_owned(), prefix.to_owned()))
    }
}

/// Connected terminators are checked for termination regularly.  If the
/// 'terminate' function of the terminator returns true the solver is
/// terminated synchronously as soon it calls this function.
pub trait Terminator {
    fn terminated(&mut self) -> bool;
}

/// A [`Terminator`] that can be triggered from any thread, e.g. to stop a
/// long-running [`CaDiCal::solve`] after a timeout. Clones share the same flag.
#[derive(Debug, Clone, Default)]
pub struct TerminateFlag(Arc<AtomicBool>);

impl TerminateFlag {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Ask the solver to stop as soon as possible; `solve` then returns
    /// [`Status::UNKNOWN`].
    pub fn terminate(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    /// Clear the flag so that the solver can be used again.
    pub fn reset(&self) {
        self.0.store(false, Ordering::Relaxed);
    }

    #[must_use]
    pub fn is_terminated(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

impl Terminator for TerminateFlag {
    fn terminated(&mut self) -> bool {
        self.is_terminated()
    }
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
    fn is_lazy(&mut self) -> bool {
        false
    }

    /// Reason external clauses can be deleted
    fn are_reasons_forgettable(&mut self) -> bool {
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
    fn cb_check_found_model(&mut self, model: &[i32]) -> bool;

    /// Ask the external propagator for the next decision literal. If it
    /// returns 0, the solver makes its own choice.
    fn cb_decide(&mut self) -> i32 {
        0
    }

    /// Ask the external propagator if there is an external propagation to make
    /// under the current assignment. It returns either a literal to be
    /// propagated or 0, indicating that there is no external propagation under
    /// the current assignment.
    fn cb_propagate(&mut self) -> i32 {
        0
    }

    /// Ask the external propagator for the reason clause of a previous
    /// external propagation step (done by `cb_propagate`). The clause must be
    /// added literal-by-literal closed with a 0. Further, the clause must
    /// contain the propagated literal.
    ///
    /// The clause will be learned as an Irredundant Non-Forgettable Clause (see
    /// below at '`cb_has_external_clause`' more details about it).
    fn cb_add_reason_clause_lit(&mut self, _propagated_lit: i32) -> i32 {
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
    fn cb_has_external_clause(&mut self, is_forgettable: &mut bool) -> bool;

    /// The actual function called to add the external clause.
    fn cb_add_external_clause_lit(&mut self) -> i32;
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
    fn witness(&mut self, clause: &[i32], witness: &[i32], id: i64) -> bool;
}

/// Trait for proof tracing that allows you to track proof events in real-time.
/// This is useful for SMT solvers that need to interleave SAT solver clauses
/// with theory clauses to generate eDRAT proofs.
pub trait ProofTracer {
    /// Called when an original clause is added to the solver
    fn add_original_clause(&mut self, id: i64, redundant: bool, clause: &[i32], restored: bool);

    /// Called when a derived clause is learned by the SAT solver. If `witness`
    /// is not zero the clause is a RAT clause with that witness literal.
    fn add_derived_clause(
        &mut self,
        id: i64,
        redundant: bool,
        witness: i32,
        clause: &[i32],
        antecedents: &[i64],
    );

    /// Called when a clause is deleted
    fn delete_clause(&mut self, id: i64, redundant: bool, clause: &[i32]);

    /// Called when a clause is weakened (might be restored later)
    fn weaken_minus(&mut self, id: i64, clause: &[i32]);

    /// Called when a clause is strengthened
    fn strengthen(&mut self, id: i64);

    /// Called when a clause is finalized
    fn finalize_clause(&mut self, id: i64, clause: &[i32]);

    /// Called when an assumption is added
    fn add_assumption(&mut self, lit: i32);

    /// Called when a constraint is added
    fn add_constraint(&mut self, clause: &[i32]);

    /// Called when assumptions are reset
    fn reset_assumptions(&mut self);

    /// Called when an assumption clause is added (negation of failing assumptions)
    fn add_assumption_clause(&mut self, id: i64, clause: &[i32], antecedents: &[i64]);

    /// Called when the solver concludes SAT
    fn conclude_sat(&mut self, conclusion_type: i32, model: &[i32]);

    /// Called when the solver concludes UNSAT
    fn conclude_unsat(&mut self, conclusion_type: i32, clause_ids: &[i64]);

    /// Called when the solver concludes UNKNOWN
    fn conclude_unknown(&mut self, trail: &[i32]);

    /// Called when a clause is demoted from irredundant to redundant
    fn demote_clause(&mut self, _id: i64, _clause: &[i32]) {}

    /// Called when a solve call ends with the given status. If the status is
    /// UNSAT and an empty clause has been derived, `id` is its id.
    fn report_status(&mut self, _status: i32, _id: i64) {}

    /// Called when the proof begins; ids below `first_derived_id` are
    /// reserved for original clauses
    fn begin_proof(&mut self, _first_derived_id: i64) {}

    /// Called when a solve call (query) starts
    fn solve_query(&mut self) {}

    /// Called when two literals are found to be equivalent (possibly
    /// including variables added by the solver, e.g. through BVA)
    fn notify_equivalence(&mut self, _lit: i32, _other: i32) {}
}

// ************************************************************************************************
// Shared callbacks
// ************************************************************************************************

// The solver owns its callbacks, so these impls let the caller keep an `Rc` to
// a callback and inspect it while it is connected (e.g. after `solve`).

impl<T: Terminator> Terminator for Rc<RefCell<T>> {
    fn terminated(&mut self) -> bool {
        self.borrow_mut().terminated()
    }
}

impl<T: Learner> Learner for Rc<RefCell<T>> {
    fn learning(&mut self, size: i32) -> bool {
        self.borrow_mut().learning(size)
    }
    fn learn(&mut self, lit: i32) {
        self.borrow_mut().learn(lit);
    }
}

impl<T: FixedAssignmentListener> FixedAssignmentListener for Rc<RefCell<T>> {
    fn notify_fixed_assignment(&mut self, lit: i32) {
        self.borrow_mut().notify_fixed_assignment(lit);
    }
}

impl<T: ExternalPropagator> ExternalPropagator for Rc<RefCell<T>> {
    fn is_lazy(&mut self) -> bool {
        self.borrow_mut().is_lazy()
    }
    fn are_reasons_forgettable(&mut self) -> bool {
        self.borrow_mut().are_reasons_forgettable()
    }
    fn notify_assignment(&mut self, lits: &[i32]) {
        self.borrow_mut().notify_assignment(lits);
    }
    fn notify_new_decision_level(&mut self) {
        self.borrow_mut().notify_new_decision_level();
    }
    fn notify_backtrack(&mut self, new_level: usize) {
        self.borrow_mut().notify_backtrack(new_level);
    }
    fn cb_check_found_model(&mut self, model: &[i32]) -> bool {
        self.borrow_mut().cb_check_found_model(model)
    }
    fn cb_decide(&mut self) -> i32 {
        self.borrow_mut().cb_decide()
    }
    fn cb_propagate(&mut self) -> i32 {
        self.borrow_mut().cb_propagate()
    }
    fn cb_add_reason_clause_lit(&mut self, propagated_lit: i32) -> i32 {
        self.borrow_mut().cb_add_reason_clause_lit(propagated_lit)
    }
    fn cb_has_external_clause(&mut self, is_forgettable: &mut bool) -> bool {
        self.borrow_mut().cb_has_external_clause(is_forgettable)
    }
    fn cb_add_external_clause_lit(&mut self) -> i32 {
        self.borrow_mut().cb_add_external_clause_lit()
    }
}

impl<T: ProofTracer> ProofTracer for Rc<RefCell<T>> {
    fn add_original_clause(&mut self, id: i64, redundant: bool, clause: &[i32], restored: bool) {
        self.borrow_mut()
            .add_original_clause(id, redundant, clause, restored);
    }
    fn add_derived_clause(
        &mut self,
        id: i64,
        redundant: bool,
        witness: i32,
        clause: &[i32],
        antecedents: &[i64],
    ) {
        self.borrow_mut()
            .add_derived_clause(id, redundant, witness, clause, antecedents);
    }
    fn delete_clause(&mut self, id: i64, redundant: bool, clause: &[i32]) {
        self.borrow_mut().delete_clause(id, redundant, clause);
    }
    fn weaken_minus(&mut self, id: i64, clause: &[i32]) {
        self.borrow_mut().weaken_minus(id, clause);
    }
    fn strengthen(&mut self, id: i64) {
        self.borrow_mut().strengthen(id);
    }
    fn finalize_clause(&mut self, id: i64, clause: &[i32]) {
        self.borrow_mut().finalize_clause(id, clause);
    }
    fn add_assumption(&mut self, lit: i32) {
        self.borrow_mut().add_assumption(lit);
    }
    fn add_constraint(&mut self, clause: &[i32]) {
        self.borrow_mut().add_constraint(clause);
    }
    fn reset_assumptions(&mut self) {
        self.borrow_mut().reset_assumptions();
    }
    fn add_assumption_clause(&mut self, id: i64, clause: &[i32], antecedents: &[i64]) {
        self.borrow_mut()
            .add_assumption_clause(id, clause, antecedents);
    }
    fn conclude_sat(&mut self, conclusion_type: i32, model: &[i32]) {
        self.borrow_mut().conclude_sat(conclusion_type, model);
    }
    fn conclude_unsat(&mut self, conclusion_type: i32, clause_ids: &[i64]) {
        self.borrow_mut()
            .conclude_unsat(conclusion_type, clause_ids);
    }
    fn conclude_unknown(&mut self, trail: &[i32]) {
        self.borrow_mut().conclude_unknown(trail);
    }
    fn demote_clause(&mut self, id: i64, clause: &[i32]) {
        self.borrow_mut().demote_clause(id, clause);
    }
    fn report_status(&mut self, status: i32, id: i64) {
        self.borrow_mut().report_status(status, id);
    }
    fn begin_proof(&mut self, first_derived_id: i64) {
        self.borrow_mut().begin_proof(first_derived_id);
    }
    fn solve_query(&mut self) {
        self.borrow_mut().solve_query();
    }
    fn notify_equivalence(&mut self, lit: i32, other: i32) {
        self.borrow_mut().notify_equivalence(lit, other);
    }
}

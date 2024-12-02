//! # cadical-sys
//!
//! Rust bindings for the CaDiCaL SAT Solver, providing low-level access to one of the most efficient Boolean Satisfiability (SAT) solving libraries.
//!
//! ## Overview
//!
//! `cadical-sys` offers complete Rust bindings to the CaDiCaL SAT solver using the `cxx` crate, enabling seamless interoperability between Rust and C++ SAT solving capabilities.
//!
//! ### What is a SAT Solver?
//!
//! A SAT (Boolean Satisfiability) solver is a computational tool that determines whether there exists an assignment of boolean variables that makes a given boolean formula true. SAT solvers are crucial in:
//! - Formal verification
//! - Hardware design
//! - AI planning
//! - Cryptanalysis
//! - Constraint solving
//!
//! ### About CaDiCaL
//!
//! [CaDiCaL](https://github.com/arminbiere/cadical) is a state-of-the-art, modern SAT solver developed by Armin Biere. Known for its:
//! - High performance
//! - Extensive features
//! - Compact implementation
//! - Advanced conflict-driven clause learning (CDCL) techniques
//!
//! ## Features
//!
//! - Complete binding of CaDiCaL C++ API
//! - Safe Rust wrappers using `cxx`
//! - Support for:
//!   - Adding clauses
//!   - Solving boolean satisfiability problems
//!   - Assumption handling
//!   - Advanced solver configuration
//!   - Proof tracing
//!   - Incremental solving
//!
//! ## Installation
//!
//! Add to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! cadical-sys = "0.1.0"  # Replace with most recent version
//! ```
//!
//! ## Basic Usage Example
//!
//! ```rust
//! use cadical_sys::ffi;
//!
//!     //! Create a new solver instance
//!     let mut solver = ffi::constructor();
//!
//!     //! Add clauses (represent boolean constraints)
//!     ffi::clause2(&mut solver, 1, -2);   //! (x1 OR NOT x2)
//!     ffi::clause2(&mut solver, -1, 2);   //! (NOT x1 OR x2)
//!
//!     //! Solve the SAT problem
//!     let result = ffi::solve(&mut solver);
//!
//!     match result {
//!         10 => println!("Satisfiable!"),
//!         20 => println!("Unsatisfiable!"),
//!         _ => println!("Unknown result"),
//!     }
//! ```
//!
//! ## Advanced Features
//!
//! ### Solver Configuration
//!
//! ```rust
//! //! Set solver-specific options
//! solver.set("verbose".to_string(), 1);
//! solver.configure("plain".to_string());
//! ```
//!
//! ### Incremental Solving
//!
//! ```rust
//! //! Add assumptions for solving
//! ffi::assume(&mut solver, 1);   //! Assume x1 is true
//! let result = ffi::solve(&mut solver);
//! ```
//!
//! ### Proof Tracing
//!
//! ```rust
//! //! Trace proof to a file
//! ffi::trace_proof1(&mut solver, "proof.out".to_string(), "myproof".to_string());
//! ```
//!
//! ## Performance Considerations
//!
//! - CaDiCaL is highly optimized for complex boolean satisfiability problems
//! - Recommended for problems with thousands to millions of variables
//! - Lower overhead compared to many other SAT solvers
//!
//! ## Limitations
//!
//! - Requires understanding of boolean logic and SAT solving
//! - Performance depends on problem complexity
//! - Advanced features require deep knowledge of SAT solving techniques
//!
//! ## Contributing
//!
//! Contributions are welcome! Please file issues or submit pull requests on the GitHub repository.
//!
//! ## License
//!
//! CaDiCaL is distributed under the MIT License. Check the original repository for detailed licensing information.
//!
//! ## References
//!
//! - [CaDiCaL GitHub Repository](https://!github.com/arminbiere/cadical)
//! - [cxx Rust Bindings](https://!cxx.rs/)
//! - [SAT Solver Overview](https://!en.wikipedia.org/wiki/Boolean_satisfiability_problem)
//!
//! ## Acknowledgments
//!
//! Special thanks to Armin Biere for developing and maintaining CaDiCaL.

pub mod bridge;
pub use bridge::ffi::*;

# cadical-sys

Rust bindings for the CaDiCaL SAT Solver, providing low-level access to one of the most efficient Boolean Satisfiability (SAT) solving libraries.

## Overview

`cadical-sys` offers complete Rust bindings to the CaDiCaL SAT solver using the `cxx` crate, enabling seamless interoperability between Rust and C++ SAT solving capabilities.

### What is a SAT Solver?

A SAT (Boolean Satisfiability) solver is a computational tool that determines whether there exists an assignment of boolean variables that makes a given boolean formula true. SAT solvers are crucial in:
- Formal verification
- Hardware design
- AI planning
- Cryptanalysis
- Constraint solving

### About CaDiCaL

[CaDiCaL](https://github.com/arminbiere/cadical) is a state-of-the-art, modern SAT solver developed by Armin Biere. Known for its:
- High performance
- Extensive features
- Compact implementation
- Advanced conflict-driven clause learning (CDCL) techniques

## Features

- Complete binding of CaDiCaL C++ API
- Safe Rust wrappers using `cxx` (where possible)
- Support for:
  - Adding clauses
  - Solving boolean satisfiability problems
  - Assumption handling
  - Advanced solver configuration
  - Proof tracing
  - Incremental solving

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
cadical-sys = "0.1.0"  # Replace with most recent version
```

## Usage Examples

### Basic SAT solving example
```rust
   use cadical_sys::Status;
   use cadical_sys::CaDiCal;

   // Create a new solver instance
   let mut solver = CaDiCal::new();

   // Add clauses (representing a simple propositional logic problem)
   // For example, (x1 OR x2) AND (NOT x1 OR x3) AND (NOT x2 OR NOT x3)
   solver.clause2(1, 2);    // x1 OR x2
   solver.clause2(-1, 3);   // NOT x1 OR x3
   solver.clause2(-2, -3);  // NOT x2 OR NOT x3

   // Solve the problem
   let status = solver.solve();
   match status {
       Status::SATISFIABLE => {
           // Get variable assignments
           println!("x1: {}", solver.val(1));
           println!("x2: {}", solver.val(2));
           println!("x3: {}", solver.val(3));
       },
       Status::UNSATISFIABLE => println!("No solution exists"),
       Status::UNKNOWN => println!("Solution status unknown")
   }
```

### Advanced example with assumptions and configuration
```rust
   use cadical_sys::Status;
   use cadical_sys::CaDiCal;

   let mut solver = CaDiCal::new();

   // Configure the solver
   solver.configure("plain".to_string());

   // Set some options
   solver.set("verbose".to_string(), 1);

   // Add complex clauses
   solver.clause3(1, 2, 3);  // x1 OR x2 OR x3
   solver.clause3(-1, -2, -3);  // NOT x1 OR NOT x2 OR NOT x3

   // Make assumptions
   solver.assume(1);  // Assume x1 is true

   // Solve with assumptions
   let status = solver.solve();

   // Check solving results
   if status == Status::SATISFIABLE {
       // Interact with solved model
       let num_vars = solver.vars();
       for var in 1..=num_vars {
           println!("Variable {}: {}", var, solver.val(var));
       }
   }
```

### Example of reading DIMACS file and solving
```rust
   use cadical_sys::Status;
   use cadical_sys::CaDiCal;

   let mut solver = CaDiCal::new();
   let mut var_count = 0;

   // Read a DIMACS CNF file
   let result = solver.read_dimacs1(
       "./tests/problem.cnf".to_string(),
       "my_problem".to_string(),
       &mut var_count,
       0
   );

   // Solve the problem from the file
   let status = solver.solve();

   // Write out results or extension
   if status == Status::SATISFIABLE {
       solver.write_extension("/tmp/solution.ext".to_string());
   }
```

### Demonstrating advanced solver interactions
```rust
   use cadical_sys::CaDiCal;

   let mut solver = CaDiCal::new();

   // Reserve variable space
   solver.reserve(1000);

   // Add observed variables for tracking
   solver.add_observed_var(42);

   // Perform simplification
   let simplify_status = solver.simplify(2);

   // Get solver statistics
   solver.statistics();
   solver.resources();
```

## Performance Considerations

- CaDiCaL is highly optimized for complex boolean satisfiability problems
- Recommended for problems with thousands to millions of variables
- Lower overhead compared to many other SAT solvers

## Limitations

- Requires understanding of boolean logic and SAT solving
- Performance depends on problem complexity
- Advanced features require deep knowledge of SAT solving techniques

## Contributing

Contributions are welcome! Please file issues or submit pull requests on the GitHub repository.

## License

CaDiCaL is distributed under the MIT License. Check the original repository for detailed licensing information.

## References

- [CaDiCaL GitHub Repository](https://github.com/arminbiere/cadical)
- [cxx Rust Bindings](https://cxx.rs/)
- [SAT Solver Overview](https://en.wikipedia.org/wiki/Boolean_satisfiability_problem)

## Acknowledgments

Special thanks to Armin Biere for developing and maintaining CaDiCaL.

License: MIT

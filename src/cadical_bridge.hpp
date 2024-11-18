#pragma once
#include "../cadical/src/cadical.hpp"
#include <memory>
#include <vector>
#include <string>

using CaDiCaL::Solver;

// Constructor and basic operations
std::unique_ptr<Solver> create_solver();
void add_literal(std::unique_ptr<Solver>& solver, int literal);
void add_clause_end(std::unique_ptr<Solver>& solver);
int solve_solver(std::unique_ptr<Solver>& solver);

// Advanced clause addition
void add_clause(std::unique_ptr<Solver>& solver, const std::vector<int>& literals);
void add_clause_with_assumption(std::unique_ptr<Solver>& solver, int assumption);

// State checking
bool is_solver_inconsistent(const std::unique_ptr<Solver>& solver);
int get_value(const std::unique_ptr<Solver>& solver, int lit);
bool is_failed(const std::unique_ptr<Solver>& solver, int lit);

// Configuration and limits
bool set_option(std::unique_ptr<Solver>& solver, const std::string& name, int val);
int get_option(const std::unique_ptr<Solver>& solver, const std::string& name);
void optimize(std::unique_ptr<Solver>& solver, int val);
bool set_limit(std::unique_ptr<Solver>& solver, const std::string& name, int val);

// Statistics and information
int get_active_variables(const std::unique_ptr<Solver>& solver);
int64_t get_redundant_clauses(const std::unique_ptr<Solver>& solver);
int64_t get_irredundant_clauses(const std::unique_ptr<Solver>& solver);
void print_statistics(const std::unique_ptr<Solver>& solver);
void print_resources(const std::unique_ptr<Solver>& solver);

// Termination and cleanup
void terminate_solver(std::unique_ptr<Solver>& solver);
void conclude_solver(std::unique_ptr<Solver>& solver);
#include "cadical_bridge.hpp"

std::unique_ptr<Solver> create_solver() {
    return std::unique_ptr<Solver>(new Solver());
}

void add_literal(std::unique_ptr<Solver>& solver, int literal) {
    solver->add(literal);
}

void add_clause_end(std::unique_ptr<Solver>& solver) {
    solver->add(0);
}

int solve_solver(std::unique_ptr<Solver>& solver) {
    return solver->solve();
}

void add_clause(std::unique_ptr<Solver>& solver, const std::vector<int>& literals) {
    solver->clause(literals);
}

void add_clause_with_assumption(std::unique_ptr<Solver>& solver, int assumption) {
    solver->assume(assumption);
}

bool is_solver_inconsistent(const std::unique_ptr<Solver>& solver) {
    return solver->inconsistent();
}

int get_value(const std::unique_ptr<Solver>& solver, int lit) {
    return solver->val(lit);
}

bool is_failed(const std::unique_ptr<Solver>& solver, int lit) {
    return solver->failed(lit);
}

bool set_option(std::unique_ptr<Solver>& solver, const std::string& name, int val) {
    return solver->set(name.c_str(), val);
}

int get_option(const std::unique_ptr<Solver>& solver, const std::string& name) {
    return solver->get(name.c_str());
}

void optimize(std::unique_ptr<Solver>& solver, int val) {
    solver->optimize(val);
}

bool set_limit(std::unique_ptr<Solver>& solver, const std::string& name, int val) {
    return solver->limit(name.c_str(), val);
}

int get_active_variables(const std::unique_ptr<Solver>& solver) {
    return solver->active();
}

int64_t get_redundant_clauses(const std::unique_ptr<Solver>& solver) {
    return solver->redundant();
}

int64_t get_irredundant_clauses(const std::unique_ptr<Solver>& solver) {
    return solver->irredundant();
}

void print_statistics(const std::unique_ptr<Solver>& solver) {
    solver->statistics();
}

void print_resources(const std::unique_ptr<Solver>& solver) {
    solver->resources();
}

void terminate_solver(std::unique_ptr<Solver>& solver) {
    solver->terminate();
}

void conclude_solver(std::unique_ptr<Solver>& solver) {
    solver->conclude();
}
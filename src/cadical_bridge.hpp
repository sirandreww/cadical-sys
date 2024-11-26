// ************************************************************************************************
// imports
// ************************************************************************************************

#pragma once
#include "../cadical/src/cadical.hpp"
#include "../cadical/src/tracer.hpp"
#include <memory>
#include <vector>
#include <string>
#include "rust/cxx.h"

// ************************************************************************************************
// types
// ************************************************************************************************

using CaDiCaL::ClauseIterator;
using CaDiCaL::ExternalPropagator;
using CaDiCaL::FileTracer;
using CaDiCaL::FixedAssignmentListener;
using CaDiCaL::InternalTracer;
using CaDiCaL::Learner;
using CaDiCaL::Solver;
using CaDiCaL::State;
using CaDiCaL::StatTracer;
using CaDiCaL::Status;
using CaDiCaL::Terminator;
using CaDiCaL::Tracer;
using CaDiCaL::WitnessIterator;

// ************************************************************************************************
// typedefs to file
// ************************************************************************************************

// ************************************************************************************************
// helper functions
// ************************************************************************************************

FILE *_read_file(rust::String file, const char *mode)
{
    FILE *fptr = fopen(file.c_str(), mode);
    if (fptr == NULL)
    {
        printf("Failed to open file\n");
        exit(1);
    }
    return fptr;
}

template <typename T>
void _copy_vec_from_cxx_to_rust(const std::vector<T> &source, rust::Vec<T> &destination)
{
    for (auto &i : source)
    {
        destination.push_back(i);
    }
}

// ************************************************************************************************
// API
// ************************************************************************************************

std::unique_ptr<Solver> constructor()
{
    return std::unique_ptr<Solver>(new Solver());
}

rust::String signature()
{
    return Solver::signature();
}

void add(std::unique_ptr<Solver> &solver, int literal)
{
    return solver->add(literal);
}

void clause1(std::unique_ptr<Solver> &solver, int l1)
{
    return solver->clause(l1);
}

void clause2(std::unique_ptr<Solver> &solver, int l1, int l2)
{
    return solver->clause(l1, l2);
}

void clause3(std::unique_ptr<Solver> &solver, int l1, int l2, int l3)
{
    return solver->clause(l1, l2, l3);
}

void clause4(std::unique_ptr<Solver> &solver, int l1, int l2, int l3, int l4)
{
    return solver->clause(l1, l2, l3, l4);
}

void clause5(std::unique_ptr<Solver> &solver, int l1, int l2, int l3, int l4, int l5)
{
    return solver->clause(l1, l2, l3, l4, l5);
}

void clause6(std::unique_ptr<Solver> &solver, const rust::Vec<int> &v)
{
    return solver->clause(v.data(), v.size());
}

void clause7(std::unique_ptr<Solver> &solver, const int *ptr, size_t n)
{
    return solver->clause(ptr, n);
}

bool inconsistent(std::unique_ptr<Solver> &solver)
{
    return solver->inconsistent();
}

void assume(std::unique_ptr<Solver> &solver, int lit)
{
    return solver->assume(lit);
}

int solve(std::unique_ptr<Solver> &solver)
{
    return solver->solve();
}

int val(std::unique_ptr<Solver> &solver, int lit)
{
    return solver->val(lit);
}

bool flip(std::unique_ptr<Solver> &solver, int lit)
{
    return solver->flip(lit);
}

bool flippable(std::unique_ptr<Solver> &solver, int lit)
{
    return solver->flippable(lit);
}

bool failed(std::unique_ptr<Solver> &solver, int lit)
{
    return solver->failed(lit);
}

// ************************************************************************************************
// terminator
// ************************************************************************************************

void connect_terminator(std::unique_ptr<Solver> &solver, std::unique_ptr<Terminator> &terminator)
{
    return solver->connect_terminator(terminator.get());
}

void disconnect_terminator(std::unique_ptr<Solver> &solver)
{
    return solver->disconnect_terminator();
}

// ************************************************************************************************
// learner
// ************************************************************************************************

void connect_learner(std::unique_ptr<Solver> &solver, std::unique_ptr<Learner> &learner)
{
    return solver->connect_learner(learner.get());
}

void disconnect_learner(std::unique_ptr<Solver> &solver)
{
    return solver->disconnect_learner();
}

// ************************************************************************************************
// fixed_listener
// ************************************************************************************************

void connect_fixed_listener(std::unique_ptr<Solver> &solver, std::unique_ptr<FixedAssignmentListener> &fixed_listener)
{
    return solver->connect_fixed_listener(fixed_listener.get());
}

void disconnect_fixed_listener(std::unique_ptr<Solver> &solver)
{
    return solver->disconnect_fixed_listener();
}

// ************************************************************************************************
// external propagator
// ************************************************************************************************

void connect_external_propagator(std::unique_ptr<Solver> &solver, std::unique_ptr<ExternalPropagator> &propagator)
{
    return solver->connect_external_propagator(propagator.get());
}

void disconnect_external_propagator(std::unique_ptr<Solver> &solver)
{
    return solver->disconnect_external_propagator();
}

// ************************************************************************************************
// observed
// ************************************************************************************************

void add_observed_var(std::unique_ptr<Solver> &solver, int var)
{
    return solver->add_observed_var(var);
}

void remove_observed_var(std::unique_ptr<Solver> &solver, int var)
{
    return solver->remove_observed_var(var);
}

void reset_observed_vars(std::unique_ptr<Solver> &solver)
{
    return solver->reset_observed_vars();
}

// ************************************************************************************************
// ? API
// ************************************************************************************************

bool is_decision(std::unique_ptr<Solver> &solver, int lit)
{
    return solver->is_decision(lit);
}

void force_backtrack(std::unique_ptr<Solver> &solver, size_t new_level)
{
    return solver->force_backtrack(new_level);
}

// ************************************************************************************************
// constrain clause
// ************************************************************************************************

void constrain(std::unique_ptr<Solver> &solver, int lit)
{
    return solver->constrain(lit);
}

bool constraint_failed(std::unique_ptr<Solver> &solver)
{
    return solver->constraint_failed();
}

// ************************************************************************************************
// ? API
// ************************************************************************************************

int lookahead(std::unique_ptr<Solver> &solver)
{
    return solver->lookahead();
}

int generate_cubes(std::unique_ptr<Solver> &solver, int x, int min_depth, rust::Vec<int> &result_cubes)
{
    auto r = solver->generate_cubes(x, min_depth);
    for (auto &cube : r.cubes)
    {
        _copy_vec_from_cxx_to_rust(cube, result_cubes);
        result_cubes.push_back(0);
    }
    return r.status;
}

// ************************************************************************************************
// reset
// ************************************************************************************************

void reset_assumptions(std::unique_ptr<Solver> &solver)
{
    return solver->reset_assumptions();
}

void reset_constraint(std::unique_ptr<Solver> &solver)
{
    return solver->reset_constraint();
}

// ************************************************************************************************
// status
// ************************************************************************************************

const State &state(const std::unique_ptr<Solver> &solver)
{
    return solver->state();
}

int status(const std::unique_ptr<Solver> &solver)
{
    return solver->status();
}

// ************************************************************************************************
// version
// ************************************************************************************************

rust::String version()
{
    return Solver::version();
}

// ************************************************************************************************
// copy
// ************************************************************************************************

void copy(const std::unique_ptr<Solver> &source, std::unique_ptr<Solver> &destination)
{
    Solver *dest = destination.get();
    return source->copy(*dest);
}

// ************************************************************************************************
// vars
// ************************************************************************************************

int vars(std::unique_ptr<Solver> &solver)
{
    return solver->vars();
}

void reserve(std::unique_ptr<Solver> &solver, int min_max_var)
{
    return solver->reserve(min_max_var);
}

// void trace_api_calls(std::unique_ptr<Solver> &solver, rust::String file)
// {
//     FILE *fptr = _read_file(file, "w+");
//     solver->trace_api_calls(fptr);
//     fclose(fptr);
// }

bool is_preprocessing_option(rust::String name)
{
    return Solver::is_preprocessing_option(name.c_str());
}

bool is_valid_long_option(rust::String arg)
{
    return Solver::is_valid_long_option(arg.c_str());
}

int get(std::unique_ptr<Solver> &solver, rust::String name)
{
    return solver->get(name.c_str());
}

void prefix(std::unique_ptr<Solver> &solver, rust::String verbose_message_prefix)
{
    return solver->prefix(verbose_message_prefix.c_str());
}

bool set(std::unique_ptr<Solver> &solver, rust::String name, int val)
{
    return solver->set(name.c_str(), val);
}

bool set_long_option(std::unique_ptr<Solver> &solver, rust::String arg)
{
    return solver->set_long_option(arg.c_str());
}

bool is_valid_configuration(rust::String name)
{
    return Solver::is_valid_configuration(name.c_str());
}

bool configure(std::unique_ptr<Solver> &solver, rust::String name)
{
    return solver->configure(name.c_str());
}

void optimize(std::unique_ptr<Solver> &solver, int val)
{
    return solver->optimize(val);
}

bool limit(std::unique_ptr<Solver> &solver, rust::String arg, int val)
{
    return solver->limit(arg.c_str(), val);
}

bool is_valid_limit(std::unique_ptr<Solver> &solver, rust::String arg)
{
    return solver->is_valid_limit(arg.c_str());
}

int active(const std::unique_ptr<Solver> &solver)
{
    return solver->active();
}

int64_t redundant(const std::unique_ptr<Solver> &solver)
{
    return solver->redundant();
}

int64_t irredundant(const std::unique_ptr<Solver> &solver)
{
    return solver->irredundant();
}

int simplify(std::unique_ptr<Solver> &solver, int rounds)
{
    return solver->simplify(rounds);
}

void terminate(std::unique_ptr<Solver> &solver)
{
    return solver->terminate();
}

bool frozen(const std::unique_ptr<Solver> &solver, int lit)
{
    return solver->frozen(lit);
}

void freeze(std::unique_ptr<Solver> &solver, int lit)
{
    return solver->freeze(lit);
}

void melt(std::unique_ptr<Solver> &solver, int lit)
{
    return solver->melt(lit);
}

int fixed(const std::unique_ptr<Solver> &solver, int lit)
{
    return solver->fixed(lit);
}

void phase(std::unique_ptr<Solver> &solver, int lit)
{
    return solver->phase(lit);
}

void unphase(std::unique_ptr<Solver> &solver, int lit)
{
    return solver->unphase(lit);
}

bool trace_proof1(std::unique_ptr<Solver> &solver, rust::String file, rust::String name)
{
    auto fptr = _read_file(file, "w+");
    auto r = solver->trace_proof(fptr, name.c_str());
    fclose(fptr);
    return r;
}

bool trace_proof2(std::unique_ptr<Solver> &solver, rust::String path)
{
    return solver->trace_proof(path.c_str());
}

void flush_proof_trace(std::unique_ptr<Solver> &solver, bool print)
{
    return solver->flush_proof_trace(print);
}

void close_proof_trace(std::unique_ptr<Solver> &solver, bool print)
{
    return solver->close_proof_trace(print);
}

void connect_proof_tracer1(std::unique_ptr<Solver> &solver, std::unique_ptr<Tracer> &tracer, bool antecedents)
{
    return solver->connect_proof_tracer(tracer.get(), antecedents);
}

void connect_proof_tracer2(std::unique_ptr<Solver> &solver, std::unique_ptr<InternalTracer> &tracer, bool antecedents)
{
    return solver->connect_proof_tracer(tracer.get(), antecedents);
}

void connect_proof_tracer3(std::unique_ptr<Solver> &solver, std::unique_ptr<StatTracer> &tracer, bool antecedents)
{
    return solver->connect_proof_tracer(tracer.get(), antecedents);
}

void connect_proof_tracer4(std::unique_ptr<Solver> &solver, std::unique_ptr<FileTracer> &tracer, bool antecedents)
{
    return solver->connect_proof_tracer(tracer.get(), antecedents);
}

void conclude(std::unique_ptr<Solver> &solver)
{
    return solver->conclude();
}

bool disconnect_proof_tracer1(std::unique_ptr<Solver> &solver, std::unique_ptr<Tracer> &tracer)
{
    return solver->disconnect_proof_tracer(tracer.get());
}

bool disconnect_proof_tracer2(std::unique_ptr<Solver> &solver, std::unique_ptr<StatTracer> &tracer)
{
    return solver->disconnect_proof_tracer(tracer.get());
}

bool disconnect_proof_tracer3(std::unique_ptr<Solver> &solver, std::unique_ptr<FileTracer> &tracer)
{
    return solver->disconnect_proof_tracer(tracer.get());
}

void usage()
{
    return Solver::usage();
}

void configurations()
{
    return Solver::configurations();
}

void statistics(std::unique_ptr<Solver> &solver)
{
    return solver->statistics();
}

void resources(std::unique_ptr<Solver> &solver)
{
    return solver->resources();
}

void options(std::unique_ptr<Solver> &solver)
{
    return solver->options();
}

bool traverse_clauses(const std::unique_ptr<Solver> &solver, std::unique_ptr<ClauseIterator> &i)
{
    return solver->traverse_clauses(*i.get());
}

bool traverse_witnesses_backward(const std::unique_ptr<Solver> &solver, std::unique_ptr<WitnessIterator> &i)
{
    return solver->traverse_witnesses_backward(*i.get());
}

bool traverse_witnesses_forward(const std::unique_ptr<Solver> &solver, std::unique_ptr<WitnessIterator> &i)
{
    return solver->traverse_witnesses_forward(*i.get());
}

rust::String read_dimacs1(std::unique_ptr<Solver> &solver, rust::String file, rust::String name, int &vars, int strict)
{
    FILE *fptr = _read_file(file, "r");
    auto r = solver->read_dimacs(fptr, name.c_str(), vars, strict);
    fclose(fptr);
    return r;
}

rust::String read_dimacs2(std::unique_ptr<Solver> &solver, rust::String path, int &vars, int strict)
{
    return solver->read_dimacs(path.c_str(), vars, strict);
}

rust::String read_dimacs3(std::unique_ptr<Solver> &solver, rust::String file, rust::String name, int &vars,
                          int strict, bool &incremental,
                          rust::Vec<int> &cubes)
{
    std::vector<int> internal_cubes;
    FILE *fptr = _read_file(file, "r");
    auto r = solver->read_dimacs(fptr, name.c_str(), vars, strict, incremental, internal_cubes);
    fclose(fptr);
    _copy_vec_from_cxx_to_rust(internal_cubes, cubes);

    return r;
}

rust::String read_dimacs4(std::unique_ptr<Solver> &solver, rust::String path, int &vars, int strict,
                          bool &incremental, rust::Vec<int> &cubes)
{
    std::vector<int> internal_cubes;
    auto r = solver->read_dimacs(path.c_str(), vars, strict, incremental, internal_cubes);
    _copy_vec_from_cxx_to_rust(internal_cubes, cubes);
    return r;
}

rust::String write_dimacs(std::unique_ptr<Solver> &solver, rust::String path, int min_max_var)
{
    return solver->write_dimacs(path.c_str(), min_max_var);
}

rust::String write_extension(std::unique_ptr<Solver> &solver, rust::String path)
{
    return solver->write_extension(path.c_str());
}

void build(rust::String file, rust::String prefix)
{
    FILE *fptr = _read_file(file, "w+");
    Solver::build(fptr, prefix.c_str());
    fclose(fptr);
}

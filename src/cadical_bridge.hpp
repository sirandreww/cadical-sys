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
using CaDiCaL::ConclusionType;

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

rust::String _convert_char_to_rust_string(const char *str)
{
    if (str == NULL)
    {
        return rust::String("Null");
    }
    else
    {
        return rust::String(str);
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
    return _convert_char_to_rust_string(Solver::signature());
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

void clause6(std::unique_ptr<Solver> &solver, const rust::Slice<const int> v)
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

int state(const std::unique_ptr<Solver> &solver)
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

bool is_valid_option(rust::String name)
{
    return Solver::is_valid_option(name.c_str());
}

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
    return _convert_char_to_rust_string(r);
}

rust::String read_dimacs2(std::unique_ptr<Solver> &solver, rust::String path, int &vars, int strict)
{
    return _convert_char_to_rust_string(solver->read_dimacs(path.c_str(), vars, strict));
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

    return _convert_char_to_rust_string(r);
}

rust::String read_dimacs4(std::unique_ptr<Solver> &solver, rust::String path, int &vars, int strict,
                          bool &incremental, rust::Vec<int> &cubes)
{
    std::vector<int> internal_cubes;
    auto r = solver->read_dimacs(path.c_str(), vars, strict, incremental, internal_cubes);
    _copy_vec_from_cxx_to_rust(internal_cubes, cubes);
    return _convert_char_to_rust_string(r);
}

rust::String write_dimacs(std::unique_ptr<Solver> &solver, rust::String path, int min_max_var)
{
    return _convert_char_to_rust_string(solver->write_dimacs(path.c_str(), min_max_var));
}

rust::String write_extension(std::unique_ptr<Solver> &solver, rust::String path)
{
    return _convert_char_to_rust_string(solver->write_extension(path.c_str()));
}

void build(rust::String file, rust::String prefix)
{
    FILE *fptr = _read_file(file, "w+");
    Solver::build(fptr, prefix.c_str());
    fclose(fptr);
}

// ************************************************************************************************
// Objects that pass a state around
// ************************************************************************************************

struct CustomTerminator : public Terminator
{
    uint8_t *s;
    rust::Fn<bool(uint8_t *)> f;

    CustomTerminator(uint8_t *s, rust::Fn<bool(uint8_t *)> f) : s(s), f(f) {}

    bool terminate() override
    {
        return f(s);
    }
};

struct CustomLearner : public Learner
{
    uint8_t *s;
    rust::Fn<bool(uint8_t *, int)> f;
    rust::Fn<void(uint8_t *, int)> h;

    CustomLearner(uint8_t *s, rust::Fn<bool(uint8_t *, int)> f, rust::Fn<void(uint8_t *, int)> h) : s(s), f(f), h(h) {}

    bool learning(int size) override
    {
        return f(s, size);
    }

    void learn(int lit) override
    {
        h(s, lit);
    }
};

struct CustomFixedAssignmentListener : public FixedAssignmentListener
{
    uint8_t *s;
    rust::Fn<void(uint8_t *, int)> f;

    CustomFixedAssignmentListener(uint8_t *s, rust::Fn<void(uint8_t *, int)> f) : s(s), f(f) {}

    void notify_fixed_assignment(int lit) override
    {
        f(s, lit);
    }
};

struct CustomClauseIterator : public ClauseIterator
{
    rust::Fn<bool(uint8_t *, const rust::Slice<const int>)> f;
    uint8_t *s;

    CustomClauseIterator(uint8_t *s, rust::Fn<bool(uint8_t *, const rust::Slice<const int>)> f) : f(f), s(s) {}

    bool clause(const std::vector<int> &clause) override
    {
        rust::Slice<const int> slice{clause.data(), clause.size()};
        return f(s, slice);
    }
};

struct CustomWitnessIterator : public WitnessIterator
{
    rust::Fn<bool(uint8_t *, const rust::Slice<const int>, const rust::Slice<const int>, uint64_t)> f;
    uint8_t *s;

    CustomWitnessIterator(uint8_t *s, rust::Fn<bool(uint8_t *, const rust::Slice<const int>, const rust::Slice<const int>, uint64_t)> f) : f(f), s(s) {}

    bool witness(const std::vector<int> &clause, const std::vector<int> &witness, uint64_t id) override
    {

        rust::Slice<const int> rust_clause{clause.data(), clause.size()};
        rust::Slice<const int> rust_witness{witness.data(), witness.size()};
        return f(s, rust_clause, rust_witness, id);
    }
};

struct CustomExternalPropagator : public ExternalPropagator
{
    uint8_t *s;
    rust::Fn<void(uint8_t *, const rust::Slice<const int32_t>)> rust_notify_assignment;
    rust::Fn<void(uint8_t *)> rust_notify_new_decision_level;
    rust::Fn<void(uint8_t *, size_t)> rust_notify_backtrack;
    rust::Fn<bool(uint8_t *, const rust::Slice<const int32_t>)> rust_cb_check_found_model;
    rust::Fn<int32_t(uint8_t *)> rust_cb_decide;
    rust::Fn<int32_t(uint8_t *)> rust_cb_propagate;
    rust::Fn<int32_t(uint8_t *, int32_t)> rust_cb_add_reason_clause_lit;
    rust::Fn<bool(uint8_t *, bool*)> rust_cb_has_external_clause;
    rust::Fn<int32_t(uint8_t *)> rust_cb_add_external_clause_lit;

    CustomExternalPropagator(
        uint8_t *s,
        bool is_lazy,
        bool are_reasons_forgettable,
        rust::Fn<void(uint8_t *, const rust::Slice<const int32_t>)> rust_notify_assignment,
        rust::Fn<void(uint8_t *)> rust_notify_new_decision_level,
        rust::Fn<void(uint8_t *, size_t)> rust_notify_backtrack,
        rust::Fn<bool(uint8_t *, const rust::Slice<const int32_t>)> rust_cb_check_found_model,
        rust::Fn<int32_t(uint8_t *)> rust_cb_decide,
        rust::Fn<int32_t(uint8_t *)> rust_cb_propagate,
        rust::Fn<int32_t(uint8_t *, int32_t)> rust_cb_add_reason_clause_lit,
        rust::Fn<bool(uint8_t *, bool*)> rust_cb_has_external_clause,
        rust::Fn<int32_t(uint8_t *)> rust_cb_add_external_clause_lit)
        : s(s),
          rust_notify_assignment(rust_notify_assignment),
          rust_notify_new_decision_level(rust_notify_new_decision_level),
          rust_notify_backtrack(rust_notify_backtrack),
          rust_cb_check_found_model(rust_cb_check_found_model),
          rust_cb_decide(rust_cb_decide),
          rust_cb_propagate(rust_cb_propagate),
          rust_cb_add_reason_clause_lit(rust_cb_add_reason_clause_lit),
          rust_cb_has_external_clause(rust_cb_has_external_clause),
          rust_cb_add_external_clause_lit(rust_cb_add_external_clause_lit)
    {
        this->s = s;
        this->is_lazy = is_lazy;
        this->are_reasons_forgettable = are_reasons_forgettable;
    }

    void notify_assignment(const std::vector<int> &lits) override
    {
        rust::Slice<const int32_t> slice{lits.data(), lits.size()};
        rust_notify_assignment(s, slice);
    }

    void notify_new_decision_level() override
    {
        return rust_notify_new_decision_level(s);
    }

    void notify_backtrack(size_t new_level) override
    {
        return rust_notify_backtrack(s, new_level);
    }

    bool cb_check_found_model(const std::vector<int> &model) override
    {
        rust::Slice<const int32_t> slice{model.data(), model.size()};
        return rust_cb_check_found_model(s, slice);
    }

    int32_t cb_decide() override
    {
        return rust_cb_decide(s);
    }

    int32_t cb_propagate() override
    {
        return rust_cb_propagate(s);
    }

    int32_t cb_add_reason_clause_lit(int32_t propagated_lit) override
    {
        return rust_cb_add_reason_clause_lit(s, propagated_lit);
    }

    bool cb_has_external_clause(bool &is_forgettable) override
    {
        return rust_cb_has_external_clause(s, &is_forgettable);
    }

    int32_t cb_add_external_clause_lit() override
    {
        return rust_cb_add_external_clause_lit(s);
    }
};

// Custom tracer for proof tracking
struct CustomTracer : public Tracer
{
    uint8_t *s;
    rust::Fn<void(uint8_t *, uint64_t, bool, const rust::Slice<const int>, bool)> rust_add_original_clause;
    rust::Fn<void(uint8_t *, uint64_t, bool, const rust::Slice<const int>, const rust::Slice<const uint64_t>)> rust_add_derived_clause;
    rust::Fn<void(uint8_t *, uint64_t, bool, const rust::Slice<const int>)> rust_delete_clause;
    rust::Fn<void(uint8_t *, uint64_t, const rust::Slice<const int>)> rust_weaken_minus;
    rust::Fn<void(uint8_t *, uint64_t)> rust_strengthen;
    rust::Fn<void(uint8_t *, uint64_t, const rust::Slice<const int>)> rust_finalize_clause;
    rust::Fn<void(uint8_t *, int)> rust_add_assumption;
    rust::Fn<void(uint8_t *, const rust::Slice<const int>)> rust_add_constraint;
    rust::Fn<void(uint8_t *)> rust_reset_assumptions;
    rust::Fn<void(uint8_t *, uint64_t, const rust::Slice<const int>, const rust::Slice<const uint64_t>)> rust_add_assumption_clause;
    rust::Fn<void(uint8_t *, int, const rust::Slice<const int>)> rust_conclude_sat;
    rust::Fn<void(uint8_t *, int, const rust::Slice<const uint64_t>)> rust_conclude_unsat;
    rust::Fn<void(uint8_t *, const rust::Slice<const int>)> rust_conclude_unknown;

    CustomTracer(
        uint8_t *s,
        rust::Fn<void(uint8_t *, uint64_t, bool, const rust::Slice<const int>, bool)> rust_add_original_clause,
        rust::Fn<void(uint8_t *, uint64_t, bool, const rust::Slice<const int>, const rust::Slice<const uint64_t>)> rust_add_derived_clause,
        rust::Fn<void(uint8_t *, uint64_t, bool, const rust::Slice<const int>)> rust_delete_clause,
        rust::Fn<void(uint8_t *, uint64_t, const rust::Slice<const int>)> rust_weaken_minus,
        rust::Fn<void(uint8_t *, uint64_t)> rust_strengthen,
        rust::Fn<void(uint8_t *, uint64_t, const rust::Slice<const int>)> rust_finalize_clause,
        rust::Fn<void(uint8_t *, int)> rust_add_assumption,
        rust::Fn<void(uint8_t *, const rust::Slice<const int>)> rust_add_constraint,
        rust::Fn<void(uint8_t *)> rust_reset_assumptions,
        rust::Fn<void(uint8_t *, uint64_t, const rust::Slice<const int>, const rust::Slice<const uint64_t>)> rust_add_assumption_clause,
        rust::Fn<void(uint8_t *, int, const rust::Slice<const int>)> rust_conclude_sat,
        rust::Fn<void(uint8_t *, int, const rust::Slice<const uint64_t>)> rust_conclude_unsat,
        rust::Fn<void(uint8_t *, const rust::Slice<const int>)> rust_conclude_unknown)
        : s(s),
          rust_add_original_clause(rust_add_original_clause),
          rust_add_derived_clause(rust_add_derived_clause),
          rust_delete_clause(rust_delete_clause),
          rust_weaken_minus(rust_weaken_minus),
          rust_strengthen(rust_strengthen),
          rust_finalize_clause(rust_finalize_clause),
          rust_add_assumption(rust_add_assumption),
          rust_add_constraint(rust_add_constraint),
          rust_reset_assumptions(rust_reset_assumptions),
          rust_add_assumption_clause(rust_add_assumption_clause),
          rust_conclude_sat(rust_conclude_sat),
          rust_conclude_unsat(rust_conclude_unsat),
          rust_conclude_unknown(rust_conclude_unknown) {}

    void add_original_clause(uint64_t id, bool redundant, const std::vector<int> &clause, bool restored = false) override
    {
        rust::Slice<const int> slice{clause.data(), clause.size()};
        rust_add_original_clause(s, id, redundant, slice, restored);
    }

    void add_derived_clause(uint64_t id, bool redundant, const std::vector<int> &clause, const std::vector<uint64_t> &antecedents) override
    {
        rust::Slice<const int> slice{clause.data(), clause.size()};
        rust::Slice<const uint64_t> antecedents_slice{antecedents.data(), antecedents.size()};
        rust_add_derived_clause(s, id, redundant, slice, antecedents_slice);
    }

    void delete_clause(uint64_t id, bool redundant, const std::vector<int> &clause) override
    {
        rust::Slice<const int> slice{clause.data(), clause.size()};
        rust_delete_clause(s, id, redundant, slice);
    }

    void weaken_minus(uint64_t id, const std::vector<int> &clause) override
    {
        rust::Slice<const int> slice{clause.data(), clause.size()};
        rust_weaken_minus(s, id, slice);
    }

    void strengthen(uint64_t id) override
    {
        rust_strengthen(s, id);
    }

    void finalize_clause(uint64_t id, const std::vector<int> &clause) override
    {
        rust::Slice<const int> slice{clause.data(), clause.size()};
        rust_finalize_clause(s, id, slice);
    }

    void add_assumption(int lit) override
    {
        rust_add_assumption(s, lit);
    }

    void add_constraint(const std::vector<int> &clause) override
    {
        rust::Slice<const int> slice{clause.data(), clause.size()};
        rust_add_constraint(s, slice);
    }

    void reset_assumptions() override
    {
        rust_reset_assumptions(s);
    }

    void add_assumption_clause(uint64_t id, const std::vector<int> &clause, const std::vector<uint64_t> &antecedents) override
    {
        rust::Slice<const int> clause_slice{clause.data(), clause.size()};
        rust::Slice<const uint64_t> antecedents_slice{antecedents.data(), antecedents.size()};
        rust_add_assumption_clause(s, id, clause_slice, antecedents_slice);
    }

    void conclude_sat(const std::vector<int> &model) override
    {
        rust::Slice<const int> slice{model.data(), model.size()};
        rust_conclude_sat(s, 0, slice); // 0 for SAT conclusion type
    }

    void conclude_unsat(ConclusionType conclusion_type, const std::vector<uint64_t> &clause_ids) override
    {
        rust::Slice<const uint64_t> slice{clause_ids.data(), clause_ids.size()};
        rust_conclude_unsat(s, static_cast<int>(conclusion_type), slice);
    }

    void conclude_unknown(const std::vector<int> &trail) override
    {
        rust::Slice<const int> slice{trail.data(), trail.size()};
        rust_conclude_unknown(s, slice);
    }
};

// ************************************************************************************************
// These functions make the objects that can be attached to the solver
// ************************************************************************************************

std::unique_ptr<Terminator> new_terminator(
    uint8_t *initial_state,
    rust::Fn<bool(uint8_t *)> terminate)
{
    return std::unique_ptr<Terminator>(new CustomTerminator(initial_state, terminate));
}

std::unique_ptr<Learner> new_learner(
    uint8_t *initial_state,
    rust::Fn<bool(uint8_t *, int)> learning,
    rust::Fn<void(uint8_t *, int)> learn)
{
    return std::unique_ptr<Learner>(new CustomLearner(initial_state, learning, learn));
}

std::unique_ptr<FixedAssignmentListener> new_fixed_assignment_listener(
    uint8_t *initial_state,
    rust::Fn<void(uint8_t *, int)> notify_fixed_assignment)
{
    return std::unique_ptr<FixedAssignmentListener>(new CustomFixedAssignmentListener(initial_state, notify_fixed_assignment));
}

std::unique_ptr<ClauseIterator> new_clause_iterator(
    uint8_t *initial_state,
    rust::Fn<bool(uint8_t *, const rust::Slice<const int>)> clause)
{
    return std::unique_ptr<ClauseIterator>(new CustomClauseIterator(initial_state, clause));
}

std::unique_ptr<WitnessIterator> new_witness_iterator(
    uint8_t *initial_state,
    rust::Fn<bool(uint8_t *, const rust::Slice<const int>, const rust::Slice<const int>, uint64_t)> witness)
{
    return std::unique_ptr<WitnessIterator>(new CustomWitnessIterator(initial_state, witness));
}

std::unique_ptr<ExternalPropagator> new_external_propagator(
    uint8_t *initial_state,
    bool is_lazy,
    bool are_reasons_forgettable,
    rust::Fn<void(uint8_t *, const rust::Slice<const int32_t>)> notify_assignment,
    rust::Fn<void(uint8_t *)> notify_new_decision_level,
    rust::Fn<void(uint8_t *, size_t)> notify_backtrack,
    rust::Fn<bool(uint8_t *, const rust::Slice<const int32_t>)> cb_check_found_model,
    rust::Fn<int32_t(uint8_t *)> cb_decide,
    rust::Fn<int32_t(uint8_t *)> cb_propagate,
    rust::Fn<int32_t(uint8_t *, int32_t)> cb_add_reason_clause_lit,
    rust::Fn<bool(uint8_t *, bool*)> cb_has_external_clause,
    rust::Fn<int32_t(uint8_t *)> cb_add_external_clause_lit)
{
    return std::unique_ptr<ExternalPropagator>(new CustomExternalPropagator(
        initial_state,
        is_lazy,
        are_reasons_forgettable,
        notify_assignment,
        notify_new_decision_level,
        notify_backtrack,
        cb_check_found_model,
        cb_decide,
        cb_propagate,
        cb_add_reason_clause_lit,
        cb_has_external_clause,
        cb_add_external_clause_lit));
}

std::unique_ptr<Tracer> new_tracer(
    uint8_t *initial_state,
    rust::Fn<void(uint8_t *, uint64_t, bool, const rust::Slice<const int>, bool)> add_original_clause,
    rust::Fn<void(uint8_t *, uint64_t, bool, const rust::Slice<const int>, const rust::Slice<const uint64_t>)> add_derived_clause,
    rust::Fn<void(uint8_t *, uint64_t, bool, const rust::Slice<const int>)> delete_clause,
    rust::Fn<void(uint8_t *, uint64_t, const rust::Slice<const int>)> weaken_minus,
    rust::Fn<void(uint8_t *, uint64_t)> strengthen,
    rust::Fn<void(uint8_t *, uint64_t, const rust::Slice<const int>)> finalize_clause,
    rust::Fn<void(uint8_t *, int)> add_assumption,
    rust::Fn<void(uint8_t *, const rust::Slice<const int>)> add_constraint,
    rust::Fn<void(uint8_t *)> reset_assumptions,
    rust::Fn<void(uint8_t *, uint64_t, const rust::Slice<const int>, const rust::Slice<const uint64_t>)> add_assumption_clause,
    rust::Fn<void(uint8_t *, int, const rust::Slice<const int>)> conclude_sat,
    rust::Fn<void(uint8_t *, int, const rust::Slice<const uint64_t>)> conclude_unsat,
    rust::Fn<void(uint8_t *, const rust::Slice<const int>)> conclude_unknown)
{
    return std::unique_ptr<Tracer>(new CustomTracer(
        initial_state,
        add_original_clause,
        add_derived_clause,
        delete_clause,
        weaken_minus,
        strengthen,
        finalize_clause,
        add_assumption,
        add_constraint,
        reset_assumptions,
        add_assumption_clause,
        conclude_sat,
        conclude_unsat,
        conclude_unknown));
}

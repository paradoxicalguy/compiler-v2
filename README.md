# banana2
a small compiler with explicit architecture

---

banana2 is a deliberately small compiler written in rust that compiles a
simple c-like language into arm64 assembly, assembles it using
aarch64-linux-gnu-gcc, and executes it via qemu.

this project is intentionally designed as a systems-learning compiler:
explicit phases, explicit data structures, and no hidden magic.

the focus is on clear phase boundaries, invariants, and data flow rather
than language features.

---

## features

- regex-based lexer
- recursive descent parser
- explicit ast
- semantic analysis with scopes and static typing
- ast-level optimizer
  - constant folding
  - constant propagation
  - dead code elimination
  - if folding
- arm64 (aarch64) code generation
- cross-architecture execution via qemu
- built-in benchmarking for each compiler phase

---

## compiler pipeline

    source code
       |
       v
    lexing        -> vec<token>
       |
       v
    parsing       -> ast (vec<stmt>)
       |
       v
    semantic      -> validated ast
       |
       v
    optimization  -> optimized ast
       |
       v
    codegen       -> arm64 assembly
       |
       v
    assemble      -> elf binary
       |
       v
    run (qemu)

each stage has a strict input/output contract.
no stage depends on the internals of another stage.

---

## project structure

    src/
     |
     +-- lib.rs              compiler as a library
     +-- main.rs             driver + benchmark
     |
     +-- lexing/
     |    +-- lexer.rs
     |    +-- token.rs
     |
     +-- parsing/
     |    +-- ast.rs
     |    +-- parser.rs
     |
     +-- semantic/
     |    +-- semantic.rs
     |
     +-- optimizer/
     |    +-- optimizer.rs
     |
     +-- codegen/
          +-- arm64.rs

each directory maps to exactly one compiler phase.

---

## language overview

types:
- int
- string
- bool (internal, produced by comparisons)

statements:
    int x = 10;
    print(x + 1);

    if (x > y) {
        print(x);
    } else {
        print(y);
    }

    {
        int x = 1;
        print(x);
    }

expressions:
- integer literals
- string literals
- identifiers
- binary operators: + - < >
- assignment expressions

---

## semantic rules

- variables must be declared before use
- no redeclaration in the same scope
- int variables must be initialized with int expressions
- if conditions must be boolean
- comparison operators produce boolean values

semantic errors are collected and reported together.

---

## optimizer

the optimizer operates purely on the ast and runs multiple passes
until convergence.

optimizations include:
- constant folding
- constant propagation
- algebraic simplification
- dead code elimination
- if condition folding

the optimizer guarantees semantic equivalence.

---

## code generation

target architecture: arm64 (aarch64)

design:
- stack-based variable allocation
- simple temporary register pool
- booleans lowered as 0 / 1
- printf via system abi

the code generator assumes the ast is semantically valid.

---

## benchmarking
main.rs benchmarks:
- lexing
- parsing
- semantic analysis
- optimization
- code generation
- assembly
- runtime execution

## example output:
lexing:        15ms
parsing:       30us
semantic:      400us
optimization:  200us
codegen:       300us
assemble:      350ms
runtime:       25ms


---

## running
requirements:
- rust
- aarch64-linux-gnu-gcc
- qemu-aarch64

run:
    cargo run
    cargo run --release

generated files:
- out.s   arm64 assembly
- out     executable binary

---

## status
this project is feature-complete for its intended scope.

## background and design notes
this project is accompanied by a detailed write-up that documents
the design decisions, tradeoffs, and implementation process:

https://rasmalai123.medium.com/compiler-b9a614f9ef7b

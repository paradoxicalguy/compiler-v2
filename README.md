# quasar v2
a small compiler with explicit architecture

---

quasar is a deliberately small compiler written in rust that compiles a
simple c-like language into arm64 assembly, assembles it using
aarch64-linux-gnu-gcc, and executes it via qemu.

this project is intentionally designed as a systems-learning compiler:
explicit phases, explicit data structures, and no hidden magic.

the focus is on clear phase boundaries, invariants, and data flow rather
than language features.

## project structure
src/
├── lexer/        # regex-based tokenization
├── parser/       # recursive descent parsing
├── ast/          # explicit tree definitions
├── semantics/    # scope tracking and type checking
├── optimizer/    # ast-to-ast transformations
├── codegen/      # arm64 assembly generation
└── main.rs       # driver and phase coordination

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
- chaos mode.

---

## compiler pipeline: 
source code -> lexer -> parser -> semantic analysis -> optimizer -> codegen
each directory maps to exactly one compiler phase.

---

## language overview

types:
- int
- string
- bool (internal, produced by comparisons)
- maybe (probablistic boolean)

statements:
    int x = 10;
    maybe m = 0.5;  // 50% chance of being true

    if (m) {
        print("heads");
    } else {
        print("tails");
    }

    while (x > 0) {
        print(x);
        x = x - 1;
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
- 'maybe' types evaluate to bool on observation (runtime)
- 'maybe' probability must be a float literal between 0.0 and 1.0
- while conditions must be boolean

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
- while loop condition folding

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

## benchmarking example output:
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
  

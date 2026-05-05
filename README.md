# Loom: A Lightweight Rust+Rhai Notebook

Loom is a Rust-native, notebook-style environment for data exploration and prototyping, designed to eliminate the pitfalls of global mutable state in traditional notebooks.

## Key Features

- **Pure-function cells:** Each cell’s output is the only persisted state, making notebooks deterministic and easier to debug.
- **Rhai scripting:** Write data pipelines and analysis in lightweight Rhai syntax — no Python required.
- **Interactive REPL-style execution:** Edit and run cells interactively, like a notebook, while maintaining clear execution order.
- **JSON notebook format:** Save and reload notebooks reliably for reproducible workflows.

## Motivation

Traditional notebooks can become hard to reason about due to hidden state, inconsistent reruns, and mutable global variables. Loom enforces **cell-level purity**, giving you a clear, reproducible, and Rust-native environment for data science and prototyping.

## Example

```rhai
# Cell a
[1, 2, 3, 4, 5]

# Cell b
mean(a)

# Cell c
variance(a)

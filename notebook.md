```rust
let df = read_csv("text.csv");
```

```rust
df.select("col_b");
let a = df.select("col_a");
```

```rust
let b = df.select("col_b");
plot(a, b,"A vs B", "line");
```


# linear-algebra-performance

to perform the tests, you can run 

```bash
perf stat -e task-clock,context-switches,cpu-migrations,page-faults,instructions,cycles,cache-references,cache-misses,branches,branch-misses cargo run --release --bin contiguous_parallel_tiled

```

Just switching between the binary you want to perform against
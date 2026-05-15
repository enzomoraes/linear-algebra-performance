# Linear Algebra Performance: Matrix Multiplication Optimization Study

A comprehensive Rust benchmark project that implements matrix multiplication (C = A × B) across 5 different strategies, demonstrating the impact of **cache optimization** and **parallelization** on computational performance.

This project explores how memory layout, access patterns, and multi-threading affect real-world performance—practical insights into why libraries like BLAS, LAPACK, and modern ML frameworks are designed the way they are.

---

## Project Goals

1. **Demonstrate** how memory layout affects performance (cache locality) at different scales
2. **Compare** different optimization techniques: tiling, parallelism, and memory layouts
3. **Understand** the trade-offs between complexity and performance
4. **Recognize** when optimization is worth it (and when it's not) based on data size and use case
5. **Provide** practical insights for choosing the right approach in systems programming and high-performance computing

## Not project goals

1. **Compete with other libraries for linear algebra performance**
    - The goal here is to show how memory layout and access impacts performance. And we use linear algebra to do so.
---

## The 5 Algorithms

### 1. 🔴 **Naive Fragmented** (Simplicity-First Approach)

**What it does:** Jagged array implementation using `Vec<Vec<f64>>` - each row is independently allocated on the heap.

```rust
// Memory layout: scattered across heap
[ptr → [e00|e01|e02|...]]  // Heap Block A
[ptr → [e10|e11|e12|...]]  // Heap Block B
[ptr → [e20|e21|e22|...]]  // Heap Block C
```

**Performance:** ⚠️ **Poor scaling** (degrades with matrix size)

**How it behaves:**
- **Small matrices (< 100×100):** Actually quite fast—overhead is minimal, cache effects matter less
- **Medium matrices (100-1000):** Performance drops significantly
- **Large matrices (1000+):** Very poor—pointer chasing dominates

**Why it struggles at scale:**
- **Pointer chasing:** Each element access requires dereferencing a pointer to a different heap location
- **Cache misses multiply:** Accessing `B[k][j]` forces 2 pointer dereferences per operation
- CPU cannot predict or prefetch memory locations efficiently
- Cache (L1/L2/L3) becomes ineffective for large datasets

**Use case:**
- **Small data/prototyping:** Quick implementation without worrying about memory layout
- **Educational reference:** Demonstrates why modern systems care about cache
- **Sparse matrices or irregular data:** When contiguous layout isn't natural anyway
- **Development speed over performance:** When simplicity is the priority
- **GPU-like operations:** When data won't fit in cache anyway

**Trade-off:** Simple to implement and understand ↔ Doesn't scale to production sizes

**Theory:** Violates spatial locality (adjacent accesses are nowhere near each other in memory)

---

### 2. 🟡 **Contiguous Strided** (Simple Optimization)

**What it does:** Single contiguous heap allocation with row-major layout, naive loop order (i, j, k).

```rust
// Memory layout: [e00|e01|e02|...|e10|e11|e12|...|e20|e21|...]
// Contiguous blocks, but poor access patterns
```

**Memory access patterns:**
- Matrix A: `A[i][0]`, `A[i][1]`, `A[i][2]`... ✅ **Sequential (great!)**
- Matrix B: `B[0][j]`, `B[1][j]`, `B[2][j]`... ⚠️ **Vertical (same as naive fragmented)**

**Performance:** ~2-3× faster than Naive Fragmented on 1000×1000+ matrices (advantage decreases for smaller matrices where memory layout overhead matters less)

**Why matrix B access is poor:**
Row-major layout stores row elements contiguously, not column elements:
```
Physical memory:  [b00 b01 b02 ... b10 b11 b12 ... b20 b21 b22 ...]
                   ↑               ↑                  ↑
Accesses: B[0][j]  B[1][j]         B[2][j]
Distance: 0        8000 bytes      16000 bytes (for 1000×1000 matrix)
```

**Where does 8000 bytes come from?**
- Each row has 1000 elements (1000 columns)
- Each element is f64 = 8 bytes
- Row stride = 1000 × 8 = **8000 bytes**
- B[1][j] is 8000 bytes away from B[0][j], not 8 bytes adjacent

For a 1000×1000 matrix: 8 KB stride between consecutive accesses means **catastrophic cache misses**:

**What happens in the innermost loop:**
```
k=0: Access B[0][j]
     CPU loads a 64-byte cache line (8 f64 values)
     Only one value is immediately useful

k=1: Access B[1][j]
     Address is 8000 bytes away
     Falls into a completely different cache line
     Very poor spatial locality
     Likely cache miss

k=2: Access B[2][j]
     Another 8000-byte jump
     Another likely cache miss

... repeated across the column
```

**Result:** Out of 1000 accesses to B[k][j], you get nearly 100% miss rate. Each access loads a different cache line that's immediately evicted before it can be reused. The CPU's cache becomes completely useless.

**Use case:**
- Teaching: demonstrates memory layout limitations
- Small matrices where overhead of optimization > benefit
- Single-threaded scenarios where simplicity matters

**Theory:** Good spatial locality (accessing A), poor spatial locality (accessing B by columns)

---

### 3. 🟢 **Contiguous Tiled** (Cache Optimization)

**What it does:** Break the matrix multiplication into cache-friendly blocks (`block_size`).
**Performance:** 5-10× faster than Contiguous Strided on large matrices (1000×1000+), **~10× faster than Naive Fragmented** on the same scale

**Why it's fast:**
1. **Temporal reuse:** Result block `C[ii:ii+block_size][jj:jj+block_size]` stays in L1/L2 cache during entire `kk` loop
2. **Spatial reuse:** B's column segments (`block_size` elements each) fit in cache, reused by all `i` iterations
3. **Problem scaling:** Replaces one large problem (poor cache) with thousands of small problems (excellent cache)

**Cache behavior:**
```
Without tiling:
- Matrix B is repeatedly accessed with large memory strides
- Cache lines are rarely reused effectively
- Many accesses result in cache misses

With tiling (`16×16` blocks):
- The algorithm works on small submatrices that fit in L1/L2 cache
  - A `16×16` block of `f64` values occupies only 2 KB (`16 × 16 × 8 bytes`)
  - Blocks from matrices A, B, and C together require roughly 6 KB
  - This comfortably fits inside a typical 32 KB L1 cache, enabling efficient reuse
- Loaded cache lines are reused many times before eviction
- Memory access becomes much more cache-friendly
- Greatly improves temporal and spatial locality
```

**Use case:**
- **Standard choice for single-threaded matrix multiplication**
- ML frameworks, scientific computing (moderate matrix sizes: 100-10,000)
- CPU-bound computations where cache is bottleneck
- Embedded systems, single-core processors
- Graphics engines, physics simulations

**Theory:** Exploits **temporal locality** (reuse) + **spatial locality** (block fits in cache)

---

### 4. 🟠 **Contiguous Parallel Strided** (Parallelization of Poor Algorithm)

**What it does:** Same as Contiguous Strided, but parallelized with Rayon across rows.

**Performance:** 
- Faster than single-threaded strided multiplication due to parallel execution
- However, speedup is typically sublinear because poor cache locality and memory bandwidth become bottlenecks
- Often scales significantly worse than cache-optimized tiled implementations

**Why limited speedup?**
- Parallelism cannot overcome poor cache locality
- Threads fight for L3 cache with many misses
- Strided memory access causes frequent cache misses
- The workload becomes memory-bound instead of compute-bound
- Adding more threads increases contention for DRAM and L3 cache

**Use case:**
- Systems with many cores but small matrices (might help despite inefficiency)
- **Generally avoid** if better single-threaded version exists
- Demonstrates why parallelizing poor algorithms is counterproductive

---

### 5. 🟢✨ **Contiguous Parallel Tiled** (Optimal Solution)

**What it does:** Combines tiling (cache optimization) with Rayon parallelism across blocks.

**Performance:**
- Tiling improves temporal locality by reusing loaded cache lines multiple times before eviction
- Small blocks can fit into L1/L2 cache, reducing expensive memory accesses
- Parallel execution increases CPU utilization while preserving cache efficiency
- Compared to naive parallel matrix multiplication, this approach scales substantially better because it reduces memory-system bottlenecks

**Why it's optimal:**
1. **Each thread gets independent block:** Minimal cache contention
2. **Cache locality within thread:** Tiling ensures data reuse
3. **Predictable memory access:** CPU can prefetch effectively
4. **Load balancing:** Work distribution is even across cores
5. **No false sharing:** Threads work on separate memory regions

**Typical scaling characteristics:**

- Often achieves significantly better multicore scaling than non-tiled approaches
- Performance is typically limited more by:
  - memory bandwidth
  - cache hierarchy efficiency
  - SIMD/vectorization quality
  - block size selection
  - NUMA architecture
- Real scaling depends heavily on hardware and implementation details

**Use case:**
- Deep learning frameworks: PyTorch, TensorFlow (CPU backend)
- High-performance computing, scientific simulations
- Any compute-intensive linear algebra operation
- **Default choice** for matrix operations on modern multi-core CPUs
- Video processing, image filters, neural networks

**Theory:** Combines **data locality** (tiling) + **task parallelism** (divide work among cores)

---

## Performance Comparison

| Algorithm | Memory Type | Cache Quality | Parallelism | Best For |
|-----------|-------------|---------------|-------------|----------|
| Naive Fragmented | Jagged | Variable | Single | Small data, rapid prototyping |
| Contiguous Strided | Contiguous | Poor | Single | Small-medium matrices, simplicity |
| Contiguous Tiled | Contiguous | Excellent | Single | General CPU use, moderate-large |
| Parallel Strided | Contiguous | Poor | Multi | Legacy code, legacy systems |
| **Parallel Tiled** | Contiguous | Excellent | Multi | **HPC, ML frameworks** |

---

## Building the Project

### Prerequisites
- Rust 1.70+ (with Cargo)
- Linux/macOS (Windows WSL supported)
- perf-tools (for detailed benchmarking)

```bash
# Clone and enter directory
cd linear-algebra-performance

# Build all implementations (release mode)
cargo build --release

# Build specific implementation
cargo build --release -p contiguous_tiled
```

---

## Running Benchmarks

### Quick Test
```bash
./test.sh
```

This runs all 5 implementations with:
- **10 warmup iterations** (cache warming)
- **50 repeated measurements** (statistical confidence)
- **1000×1000 matrices** (default size)
- **Perf metrics:** task-clock, cache-misses, branch-misses, instructions

### Output
Results saved to:
- `test_results-50-1000-264.txt` (raw perf output)
- `test_results-50-1000-264_results.csv` (parsed metrics) -> using analysis package.

---

## Interpreting Results

### Key Metrics

**Task-clock (ms):** Total CPU time used
- Lower is better
- Compare across algorithms for relative performance

**Cache-misses:** Memory access failures

**Branch-misses:** CPU mispredictions on conditional jumps
- Indicates code unpredictability
- Usually lower in optimized loops

**Instructions:** Total CPU instructions executed
- Helps understand instruction throughput

---

## Theoretical Foundations

### 1. **Spatial Locality**
Data elements near each other in memory are likely to be accessed together. 
- **Good:** Reading matrix A row-by-row (consecutive memory addresses)
- **Bad:** Reading matrix B column-by-column (addresses 8KB apart)

### 2. **Temporal Locality**
Data used recently will be used again soon.
- Tiling ensures intermediate results stay in cache while being reused

### 3. **Cache Line Efficiency**
CPU caches fetch 64-byte lines, not individual bytes.
- **Aligned access:** Can load entire useful cache line
- **Misaligned access:** Wastes cache line space

### 5. **Cache Hierarchy**
Modern CPUs have multiple cache levels:
- **L1:** 32 KB per core, 4-cycle latency
- **L2:** 256 KB per core, 12-cycle latency
- **L3:** 8-16 MB shared, 40-75 cycle latency
- **RAM:** 8 GB+, 200+ cycle latency

Tiling ensures working set fits in L1/L2, avoiding expensive RAM fetches.

---

## License

This project is provided as-is for educational purposes.

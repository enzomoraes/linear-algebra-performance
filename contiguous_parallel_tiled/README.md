````markdown
## Contiguous Tiled

Esta estratégia ataca diretamente o problema de performance da versão "naive": o acesso de memória ruim à Matriz B.

---

### 1. O Problema (Revisão)

O método "naive" (com loops `i, j, k`) tem um padrão de acesso péssimo para a Matriz B:

* **Acesso à Matriz A (`A[i][k]`):** `A[i][0]`, `A[i][1]`, `A[i][2]`...
    * ✔ **Bom:** Acesso sequencial (horizontal), ótimo para o cache.
* **Acesso à Matriz B (`B[k][j]`):** `B[0][j]`, `B[1][j]`, `B[2][j]`...
    * 🚨 **Péssimo:** Acesso por coluna (vertical).
    * Se a matriz tem 1000 colunas, cada acesso (`B[0][j]`, `B[1][j]`) salta milhares de bytes na memória, causando um "cache miss" constante. A CPU não consegue prever ou otimizar isso.

---

### Tiling (Divisão em Blocos)

Em vez de processar a matriz inteira, nós a quebramos em **blocos (tiles)** menores, de um tamanho `block_size` (ex: 16x16).



A ideia é que **um bloco inteiro caiba confortavelmente no cache da CPU** (ex: L1 ou L2).

O código implementa essa lógica com 6 loops:

```rust
// Loops de BLOCO (externos)
let n = self.rows;
let m = self.cols;
let p = other.cols;
for ii in (0..n).step_by(block_size) {
    for jj in (0..p).step_by(block_size) {
        for kk in (0..m).step_by(block_size) {
            
            let i_max = (ii + block_size).min(n);
            let j_max = (jj + block_size).min(p);
            let k_max = (kk + block_size).min(m);
            // Loops de ELEMENTO (internos)
            // Processa um micro-bloco
            for i in ii..i_max {
                for j in jj..j_max {
                    let mut sum = result_data[i * p + j]; // Carrega o acumulado
                    for k in kk..k_max {
                        // Acesso é igual ao naive, mas SÓ DENTRO DO BLOCO
                        sum += self.data[i * m + k] * other.data[k * p + j];
                    }
                    result_data[i * p + j] = sum; // Salva o acumulado
                }
            }
        }
    }
}
````

-----

### Por que isso é Rápido?

A mágica está no **reuso de dados** dentro do cache.

#### ✔ Reuso do Bloco de Resultado (`result_data`)

  * O bloco `C[ii..i_max][jj..j_max]` é selecionado pelos loops `ii` e `jj`.
  * Este bloco **permanece no cache** durante todo o loop `kk`.
  * Em vez de ler e escrever `C[i][j]` uma única vez (como no naive), nós o lemos e escrevemos repetidamente (`block_size` vezes).
  * Isso é chamado de **localidade temporal** (reusar dados que acabaram de ser usados).

#### ✔ Reuso do Bloco da Matriz B (`other.data`)

  * Vamos analisar o loop `k` interno (`for k in kk..k_max`).
  * Ele ainda acessa a Matriz B verticalmente: `B[k][j]`.
  * **MAS...** ele só faz isso para `block_size` linhas (ex: 16 linhas), e não `N` linhas (ex: 1000 linhas).
  * A CPU *consegue* carregar esses 16 pequenos pedaços de linhas no cache.
  * Quando o loop `i` interno (`for i in ii..i_max`) executa, ele **reutiliza** esses mesmos 16 pedaços de B que já estão no cache.

### Comparação do Acesso à Matriz B

| Versão | Padrão de Acesso (`B[k][j]`) | Impacto no Cache |
| :--- | :--- | :--- |
| **Naive** | `B[0][j]`, `B[1][j]`, ... `B[1000][j]` | **Desastroso.** O cache não consegue guardar 1000 linhas. |
| **Tiled** | `B[kk+0][j]`, `B[kk+1][j]`, ... `B[kk+15][j]` | **Excelente.** O cache guarda 16 linhas, que são reutilizadas por todos os `i` do bloco. |

**Resumo:** O Tiling força o processador a trabalhar em sub-problemas pequenos o suficiente para caberem no cache. Ele troca um grande problema (com péssimo acesso à memória) por milhares de pequenos problemas (com ótimo acesso à memória).

```
```
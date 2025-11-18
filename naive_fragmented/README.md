````markdown
## Naive Fragmentada

Esta implementação usa a mesma ordem de loop "ingênua" (`i, j, k`), mas muda fundamentalmente a forma como a matriz é armazenada na memória.

* **O que isso significa:** Em vez de um único bloco de memória contíguo, temos um "vetor de ponteiros".
    * O `Vec` externo armazena ponteiros para outros `Vec`s.
    * Cada `Vec` interno (cada linha) é alocado **separadamente** na heap.
---

### O Layout de Memória Fragmentado

Este é o ponto crucial. A memória **não** é contígua.


```text
Memória da Matriz (ex: `self.data`):

 Outer Vec (Ponteiros)
[ ptr_linha_0 ] -> [ elemento_0_0 | elemento_0_1 | elemento_0_2 | ... ] (Heap Bloco A)
[ ptr_linha_1 ] -> [ elemento_1_0 | elemento_1_1 | elemento_1_2 | ... ] (Heap Bloco B)
[ ptr_linha_2 ] -> [ elemento_2_0 | elemento_2_1 | elemento_2_2 | ... ] (Heap Bloco C)
[ ptr_linha_3 ] ...
````

Os blocos A, B e C podem estar em locais completamente diferentes e distantes na RAM.

-----

### O Código (Exemplo)

```rust
// O loop i, j, k é o mesmo
for i in 0..self.rows {
    for j in 0..other.cols {
        let mut sum = 0.0;
        for k in 0..self.cols {
            sum += self.data[i][k] * other.data[k][j];
        }
        result_data[i][j] = sum;
    }
}
```

-----

### Análise do Loop Interno (`k`)

Vamos analisar os acessos dentro do loop `k`, que é o mais crítico.

#### Acesso à Matriz A (`self.data[i][k]`)

  * **Loop interno:** `k`
  * **Como funciona:** Para um `i` fixo (ex: `i = 5`), o acesso `self.data[5]` é feito **uma vez** (fora do loop `k` ou otimizado pelo compilador).
  * Isso encontra o ponteiro para a Linha 5 (ex: "Heap Bloco F").
  * Dentro do loop `k`, o código executa:
    1.  `Bloco_F[0]`
    2.  `Bloco_F[1]`
    3.  `Bloco_F[2]`
    4.  ...

**Conclusão:** Este acesso é **bom!** Uma vez que a linha `i` é localizada (um "salto" de ponteiro), a leitura dessa linha (`A[i][0]`, `A[i][1]`, ...) é perfeitamente sequencial e amigável ao cache.

#### 🚨🚨 Acesso à Matriz B (`other.data[k][j]`)

  * **Loop interno:** `k`
  * **Como funciona:** Para um `j` fixo (ex: `j = 10`), o código precisa acessar `B[0][10]`, `B[1][10]`, `B[2][10]`, ...
  * Vamos ver o que acontece **a cada iteração de `k`**:

> **k = 0:**
>
> 1.  Acessa `other.data[0]` (encontra o **ponteiro** para a linha 0).
> 2.  **Pula** para o local de memória da linha 0.
> 3.  Lê o elemento `[10]` desse local.
>
> **k = 1:**
>
> 1.  Acessa `other.data[1]` (encontra o **ponteiro** para a linha 1).
> 2.  **Pula** para o local de memória da linha 1 (um endereço totalmente diferente\!).
> 3.  Lê o elemento `[10]` desse local.
>
> **k = 2:**
>
> 1.  Acessa `other.data[2]` (encontra o **ponteiro** para a linha 2).
> 2.  **Pula** para o local de memória da linha 2 (outro endereço aleatório\!).
> 3.  Lê o elemento `[10]` desse local.

**Conclusão:** Este é o **pior cenário possível** para a performance. É chamado de *pointer chasing* (caça ao ponteiro).

Cada *única multiplicação* no loop interno requer **dois saltos** de memória imprevisíveis:

1.  Um salto para encontrar o ponteiro da linha `k`.
2.  Um segundo salto para o local daquela linha e ler o dado `[j]`.

Isso destrói completamente o cache da CPU (L1/L2/L3) e causa uma enorme latência de memória a cada passo.

```
```
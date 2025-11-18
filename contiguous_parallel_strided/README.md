````markdown
## Contiguous Parallel Strided

Sua multiplicação faz isso:

```c
sum += self.data[i * self.cols + k] * other.data[k * other.cols + j];
````

Vamos decodificar:

  * `self` = matriz A
  * `other` = matriz B
  * `C[i][j]` = Σ `A[i][k]` \* `B[k][j]`

O acesso à A é:

`A[i][0]`, `A[i][1]`, `A[i][2]`, `A[i][3]`, ...

Isso está OK:

  * ✔ memória contígua
  * ✔ ótimo para cache

-----

### 🚨 Mas agora veja o acesso à B

```text
B[k][j]  =  B[k * cols + j]
               ↑
         índice da linha
```

Para um `j` fixo, quando você incrementa `k`, o que acontece?

Exemplo: matriz 1000×1000

Para `j = 0`:

  * `B[0][0]`
  * `B[1][0]`
  * `B[2][0]`
  * `B[3][0]`
  * ...

Você está caminhando **verticalmente** pela matriz.

-----

## 🚨 Por que isso é ruim?

Uma matriz contígua em Rust é armazenada em **row-major**:

`[row0 | row1 | row2 | row3 | ...]`

Ou seja:

  * Elementos de uma mesma **linha** ficam lado a lado na RAM.
  * Elementos de uma mesma **coluna** ficam muito distantes.

Caminhar pela coluna = saltar pela memória:

```text
B[0][0]   -> endereço 0
B[1][0]   -> endereço 1000
B[2][0]   -> endereço 2000
B[3][0]   -> endereço 3000
```

-----

## Exemplo visual

Como você acessa **A (bom)**:

```text
A: [ a00 a01 a02 a03 a04 a05 ... ]    (acesso sequencial)
```

Como você acessa **B (ruim)**:

```text
B memória física:

[
  b00 b01 b02 b03 ...
  b10 b11 b12 b13 ...
  b20 b21 b22 b23 ...
]
```

Seu acesso percorre assim:

```text
b00
                          b10
                                            b20
                                                            b30
```

Isso é equivalente a:

  * ler um elemento
  * pular 8 KB (ex: 1000 \* 8 bytes)
  * ler outro elemento
  * pular 8 KB
  * ler outro elemento

```
```
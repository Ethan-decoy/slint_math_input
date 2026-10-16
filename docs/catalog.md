# Mathematical input catalog

The **函数** keypad has 303 buttons in 9 directly selectable categories. These actions edit notation; they do not calculate or graph.

Click a category to show its entries; scroll within that category. Switching categories preserves the formula and restores input focus.
Font-formatting controls and redundant function aliases are omitted from the keypad.

The core retains 319 notation actions, including the omitted formatting and aliases, for programmatic input and LaTeX interchange.
Inspect `catalog::entries()` in a Rust host.
`Editor::insert_catalog(id)` performs one undoable insertion. The Slint controller accepts `catalog:<id>` tokens.

The authoritative metadata is [assets/math-catalog.json](../assets/math-catalog.json).
See [supported editing and interchange](support.md) for behavior, LaTeX and platform verification.

## 结构

| Button | Identifier | Description |
| --- | --- | --- |
| a/b | `fraction` | a/b · 可编辑结构 |
| √ | `root` | √ · 可编辑结构 |
| ⁿ√ | `indexed-root` | ⁿ√ · 可编辑结构 |
| aᵇ | `power` | aᵇ · 可编辑结构 |
| aₙ | `subscript` | aₙ · 可编辑结构 |
| a² | `square` | a² · 可编辑结构 |
| (n k) | `binomial` | (n k) · 可编辑结构 |
| \|a\| | `absolute` | \|a\| · 可编辑结构 |
| ‖a‖ | `norm` | ‖a‖ · 可编辑结构 |
| ⌊a⌋ | `floor` | ⌊a⌋ · 可编辑结构 |
| ⌈a⌉ | `ceiling` | ⌈a⌉ · 可编辑结构 |
| ⟨a,b⟩ | `angle-brackets` | ⟨a,b⟩ · 可编辑结构 |
| (a) | `parentheses` | (a) · 可编辑结构 |
| [a] | `brackets` | [a] · 可编辑结构 |
| {a} | `braces` | {a} · 可编辑结构 |
| 文字 | `text` | 直立文字片段 |
| 闭合 ) | `close-0` | 确认右定界符 ) |
| 闭合 ] | `close-1` | 确认右定界符 ] |
| 闭合 } | `close-2` | 确认右定界符 } |
| 闭合 \| | `close-3` | 确认右定界符 \| |
| 闭合 ‖ | `close-4` | 确认右定界符 ‖ |
| 闭合 ⌋ | `close-5` | 确认右定界符 ⌋ |
| 闭合 ⌉ | `close-6` | 确认右定界符 ⌉ |
| 闭合 ⟩ | `close-7` | 确认右定界符 ⟩ |
| â | `hat` | 可编辑数学重音 |
| ā | `bar` | 可编辑数学重音 |
| 向量箭头 | `vector` | 可编辑数学重音 |
| ȧ | `dot` | 可编辑数学重音 |
| ä | `double-dot` | 可编辑数学重音 |
| ã | `tilde` | 可编辑数学重音 |
| 上括注 | `overbrace` | 可编辑数学重音 |
| 下括注 | `underbrace` | 可编辑数学重音 |
| 上方注记 | `overset` | 基式上方的可编辑注记 |
| 下方注记 | `underset` | 基式下方的可编辑注记 |

## 常用

| Button | Identifier | Description |
| --- | --- | --- |
| ln | `fn-ln` | ln · 数学函数名称 |
| log | `fn-log` | log · 数学函数名称 |
| lg | `fn-lg` | lg · 数学函数名称 |
| exp | `fn-exp` | exp · 数学函数名称 |
| abs | `fn-abs` | abs · 数学函数名称 |
| arg | `fn-arg` | arg · 数学函数名称 |
| Re | `fn-Re` | Re · 数学函数名称 |
| Im | `fn-Im` | Im · 数学函数名称 |
| conj | `fn-conj` | conj · 数学函数名称 |
| sgn | `fn-sgn` | sgn · 数学函数名称 |
| floor | `fn-floor` | floor · 数学函数名称 |
| ceil | `fn-ceil` | ceil · 数学函数名称 |
| round | `fn-round` | round · 数学函数名称 |
| trunc | `fn-trunc` | trunc · 数学函数名称 |
| min | `fn-min` | min · 数学函数名称 |
| max | `fn-max` | max · 数学函数名称 |
| gcd | `fn-gcd` | gcd · 数学函数名称 |
| lcm | `fn-lcm` | lcm · 数学函数名称 |
| mod | `fn-mod` | mod · 数学函数名称 |
| nPr | `fn-nPr` | nPr · 数学函数名称 |
| nCr | `fn-nCr` | nCr · 数学函数名称 |
| n! | `factorial-symbol` | 阶乘 |

## 微积分

| Button | Identifier | Description |
| --- | --- | --- |
| lim | `lim` | lim · 下方可编辑条件 |
| limsup | `limsup` | limsup · 下方可编辑条件 |
| liminf | `liminf` | liminf · 下方可编辑条件 |
| sup | `sup` | sup · 下方可编辑条件 |
| inf | `inf` | inf · 下方可编辑条件 |
| logₐ | `log-base` | 带底对数 |
| d/dx | `derivative` | 导数算子 · 可修改变量和阶数 |
| ∂/∂x | `partial-derivative` | 导数算子 · 可修改变量和阶数 |
| d²/dx² | `second-derivative` | 导数算子 · 可修改变量和阶数 |
| ∂²/∂x² | `second-partial` | 导数算子 · 可修改变量和阶数 |
| Σ | `sum` | 可编辑上下限 |
| Π | `product` | 可编辑上下限 |
| ∫ₐᵇ | `integral` | 可编辑上下限 |
| ∫ | `indefinite-integral` | 大算子 |
| ∮ | `contour-integral` | 大算子 |
| ∬ | `double-integral` | 大算子 |
| ∭ | `triple-integral` | 大算子 |
| ⋃ | `big-union` | 大算子 |
| ⋂ | `big-intersection` | 大算子 |
| ∐ | `coproduct` | 大算子 |
| → | `approaches` | → |
| ∞ | `infinity` | ∞ |
| ∂ | `partial-symbol` | ∂ |
| ∇ | `nabla` | ∇ |
| ′ | `prime` | ′ |
| ″ | `double-prime` | ″ |

## 三角

| Button | Identifier | Description |
| --- | --- | --- |
| sin | `fn-sin` | sin · 数学函数名称 |
| cos | `fn-cos` | cos · 数学函数名称 |
| tan | `fn-tan` | tan · 数学函数名称 |
| sec | `fn-sec` | sec · 数学函数名称 |
| csc | `fn-csc` | csc · 数学函数名称 |
| cot | `fn-cot` | cot · 数学函数名称 |
| arcsin | `fn-arcsin` | arcsin · 数学函数名称 |
| arccos | `fn-arccos` | arccos · 数学函数名称 |
| arctan | `fn-arctan` | arctan · 数学函数名称 |
| arcsec | `fn-arcsec` | arcsec · 数学函数名称 |
| arccsc | `fn-arccsc` | arccsc · 数学函数名称 |
| arccot | `fn-arccot` | arccot · 数学函数名称 |
| sinh | `fn-sinh` | sinh · 数学函数名称 |
| cosh | `fn-cosh` | cosh · 数学函数名称 |
| tanh | `fn-tanh` | tanh · 数学函数名称 |
| sech | `fn-sech` | sech · 数学函数名称 |
| csch | `fn-csch` | csch · 数学函数名称 |
| coth | `fn-coth` | coth · 数学函数名称 |
| arcsinh | `fn-arcsinh` | arcsinh · 数学函数名称 |
| arccosh | `fn-arccosh` | arccosh · 数学函数名称 |
| arctanh | `fn-arctanh` | arctanh · 数学函数名称 |
| arcsech | `fn-arcsech` | arcsech · 数学函数名称 |
| arccsch | `fn-arccsch` | arccsch · 数学函数名称 |
| arccoth | `fn-arccoth` | arccoth · 数学函数名称 |

## 统计概率

| Button | Identifier | Description |
| --- | --- | --- |
| mean | `fn-mean` | mean · 数学函数名称 |
| median | `fn-median` | median · 数学函数名称 |
| stdev | `fn-stdev` | stdev · 数学函数名称 |
| stdevp | `fn-stdevp` | stdevp · 数学函数名称 |
| var | `fn-var` | var · 数学函数名称 |
| varp | `fn-varp` | varp · 数学函数名称 |
| mad | `fn-mad` | mad · 数学函数名称 |
| quantile | `fn-quantile` | quantile · 数学函数名称 |
| quartile | `fn-quartile` | quartile · 数学函数名称 |
| cov | `fn-cov` | cov · 数学函数名称 |
| corr | `fn-corr` | corr · 数学函数名称 |
| total | `fn-total` | total · 数学函数名称 |
| length | `fn-length` | length · 数学函数名称 |
| count | `fn-count` | count · 数学函数名称 |
| unique | `fn-unique` | unique · 数学函数名称 |
| sort | `fn-sort` | sort · 数学函数名称 |
| shuffle | `fn-shuffle` | shuffle · 数学函数名称 |
| join | `fn-join` | join · 数学函数名称 |
| append | `fn-append` | append · 数学函数名称 |
| reverse | `fn-reverse` | reverse · 数学函数名称 |
| random | `fn-random` | random · 数学函数名称 |
| normaldist | `fn-normaldist` | normaldist · 数学函数名称 |
| uniformdist | `fn-uniformdist` | uniformdist · 数学函数名称 |
| binomialdist | `fn-binomialdist` | binomialdist · 数学函数名称 |
| poissondist | `fn-poissondist` | poissondist · 数学函数名称 |
| geodist | `fn-geodist` | geodist · 数学函数名称 |
| tdist | `fn-tdist` | tdist · 数学函数名称 |
| chisqdist | `fn-chisqdist` | chisqdist · 数学函数名称 |
| fdist | `fn-fdist` | fdist · 数学函数名称 |
| expdist | `fn-expdist` | expdist · 数学函数名称 |
| discretedist | `fn-discretedist` | discretedist · 数学函数名称 |
| pdf | `fn-pdf` | pdf · 数学函数名称 |
| cdf | `fn-cdf` | cdf · 数学函数名称 |
| inversecdf | `fn-inversecdf` | inversecdf · 数学函数名称 |
| erf | `fn-erf` | erf · 数学函数名称 |
| erfc | `fn-erfc` | erfc · 数学函数名称 |
| gamma | `fn-gamma` | gamma · 数学函数名称 |
| beta | `fn-beta` | beta · 数学函数名称 |
| loggamma | `fn-loggamma` | loggamma · 数学函数名称 |

## 矩阵

| Button | Identifier | Description |
| --- | --- | --- |
| det | `fn-det` | det · 数学函数名称 |
| trace | `fn-trace` | trace · 数学函数名称 |
| rank | `fn-rank` | rank · 数学函数名称 |
| rref | `fn-rref` | rref · 数学函数名称 |
| transpose | `fn-transpose` | transpose · 数学函数名称 |
| inverse | `fn-inverse` | inverse · 数学函数名称 |
| rows | `fn-rows` | rows · 数学函数名称 |
| columns | `fn-columns` | columns · 数学函数名称 |
| dim | `fn-dim` | dim · 数学函数名称 |
| ker | `fn-ker` | ker · 数学函数名称 |
| hom | `fn-hom` | hom · 数学函数名称 |
| [2×2] | `matrix-2` | 矩阵或分段表达式 · 各单元格可编辑 |
| [3×3] | `matrix-3` | 矩阵或分段表达式 · 各单元格可编辑 |
| (2×2) | `pmatrix` | 矩阵或分段表达式 · 各单元格可编辑 |
| \|2×2\| | `determinant` | 矩阵或分段表达式 · 各单元格可编辑 |
| ‖2×2‖ | `norm-matrix` | 矩阵或分段表达式 · 各单元格可编辑 |
| 矩阵 | `plain-matrix` | 矩阵或分段表达式 · 各单元格可编辑 |
| 列向量 | `column-vector` | 矩阵或分段表达式 · 各单元格可编辑 |
| 行向量 | `row-vector` | 矩阵或分段表达式 · 各单元格可编辑 |
| 分段 2 行 | `cases-2` | 矩阵或分段表达式 · 各单元格可编辑 |
| 分段 3 行 | `cases-3` | 矩阵或分段表达式 · 各单元格可编辑 |
| 多行对齐 | `aligned` | 矩阵或分段表达式 · 各单元格可编辑 |
| 增加行 | `matrix-row-add` | 调整光标所在矩阵 · 缩小前须清空被移除单元格 |
| 减少行 | `matrix-row-remove` | 调整光标所在矩阵 · 缩小前须清空被移除单元格 |
| 增加列 | `matrix-column-add` | 调整光标所在矩阵 · 缩小前须清空被移除单元格 |
| 减少列 | `matrix-column-remove` | 调整光标所在矩阵 · 缩小前须清空被移除单元格 |

## 希腊字母

| Button | Identifier | Description |
| --- | --- | --- |
| α | `symbol-0` | α |
| β | `symbol-1` | β |
| γ | `symbol-2` | γ |
| δ | `symbol-3` | δ |
| ε | `symbol-4` | ε |
| ζ | `symbol-5` | ζ |
| η | `symbol-6` | η |
| θ | `symbol-7` | θ |
| ι | `symbol-8` | ι |
| κ | `symbol-9` | κ |
| λ | `symbol-10` | λ |
| μ | `symbol-11` | μ |
| ν | `symbol-12` | ν |
| ξ | `symbol-13` | ξ |
| ο | `symbol-14` | ο |
| π | `symbol-15` | π |
| ρ | `symbol-16` | ρ |
| σ | `symbol-17` | σ |
| τ | `symbol-18` | τ |
| υ | `symbol-19` | υ |
| φ | `symbol-20` | φ |
| χ | `symbol-21` | χ |
| ψ | `symbol-22` | ψ |
| ω | `symbol-23` | ω |
| ϵ | `symbol-24` | ϵ |
| ϑ | `symbol-25` | ϑ |
| ϰ | `symbol-26` | ϰ |
| ϖ | `symbol-27` | ϖ |
| ϱ | `symbol-28` | ϱ |
| ς | `symbol-29` | ς |
| ϕ | `symbol-30` | ϕ |
| Γ | `symbol-31` | Γ |
| Δ | `symbol-32` | Δ |
| Θ | `symbol-33` | Θ |
| Λ | `symbol-34` | Λ |
| Ξ | `symbol-35` | Ξ |
| Π | `symbol-36` | Π |
| Σ | `symbol-37` | Σ |
| Υ | `symbol-38` | Υ |
| Φ | `symbol-39` | Φ |
| Ψ | `symbol-40` | Ψ |
| Ω | `symbol-41` | Ω |

## 关系集合

| Button | Identifier | Description |
| --- | --- | --- |
| = | `symbol-42` | = |
| ≠ | `symbol-43` | ≠ |
| < | `symbol-44` | < |
| > | `symbol-45` | > |
| ≤ | `symbol-46` | ≤ |
| ≥ | `symbol-47` | ≥ |
| ≈ | `symbol-48` | ≈ |
| ≃ | `symbol-49` | ≃ |
| ≅ | `symbol-50` | ≅ |
| ≡ | `symbol-51` | ≡ |
| ∝ | `symbol-52` | ∝ |
| ≪ | `symbol-53` | ≪ |
| ≫ | `symbol-54` | ≫ |
| ∼ | `symbol-55` | ∼ |
| ∈ | `symbol-56` | ∈ |
| ∉ | `symbol-57` | ∉ |
| ∋ | `symbol-58` | ∋ |
| ⊂ | `symbol-59` | ⊂ |
| ⊃ | `symbol-60` | ⊃ |
| ⊆ | `symbol-61` | ⊆ |
| ⊇ | `symbol-62` | ⊇ |
| ⊊ | `symbol-63` | ⊊ |
| ⊋ | `symbol-64` | ⊋ |
| ∪ | `symbol-65` | ∪ |
| ∩ | `symbol-66` | ∩ |
| ∖ | `symbol-67` | ∖ |
| ∅ | `symbol-68` | ∅ |
| ∀ | `symbol-69` | ∀ |
| ∃ | `symbol-70` | ∃ |
| ∄ | `symbol-71` | ∄ |
| ∧ | `symbol-72` | ∧ |
| ∨ | `symbol-73` | ∨ |
| ¬ | `symbol-74` | ¬ |
| ⊕ | `symbol-75` | ⊕ |
| ⊗ | `symbol-76` | ⊗ |
| ⊙ | `symbol-77` | ⊙ |
| ⊥ | `symbol-78` | ⊥ |
| ∥ | `symbol-79` | ∥ |
| ∠ | `symbol-80` | ∠ |

## 符号常数

| Button | Identifier | Description |
| --- | --- | --- |
| → | `symbol-81` | → |
| ← | `symbol-82` | ← |
| ↔ | `symbol-83` | ↔ |
| ⇒ | `symbol-84` | ⇒ |
| ⇐ | `symbol-85` | ⇐ |
| ⇔ | `symbol-86` | ⇔ |
| ↦ | `symbol-87` | ↦ |
| ↑ | `symbol-88` | ↑ |
| ↓ | `symbol-89` | ↓ |
| ↗ | `symbol-90` | ↗ |
| ↘ | `symbol-91` | ↘ |
| ± | `symbol-92` | ± |
| ∓ | `symbol-93` | ∓ |
| × | `symbol-94` | × |
| · | `symbol-95` | · |
| ÷ | `symbol-96` | ÷ |
| ∘ | `symbol-97` | ∘ |
| ∗ | `symbol-98` | ∗ |
| ⋆ | `symbol-99` | ⋆ |
| • | `symbol-100` | • |
| ⊎ | `symbol-101` | ⊎ |
| ∴ | `symbol-102` | ∴ |
| ∵ | `symbol-103` | ∵ |
| ≔ | `symbol-104` | ≔ |
| … | `symbol-105` | … |
| ⋯ | `symbol-106` | ⋯ |
| ⋮ | `symbol-107` | ⋮ |
| ⋱ | `symbol-108` | ⋱ |
| ! | `symbol-109` | ! |
| % | `symbol-110` | % |
| ′ | `symbol-111` | ′ |
| ″ | `symbol-112` | ″ |
| π | `symbol-113` | π |
| τ | `symbol-114` | τ |
| e | `symbol-115` | e |
| i | `symbol-116` | i |
| ∞ | `symbol-117` | ∞ |
| ℝ | `symbol-118` | ℝ |
| ℕ | `symbol-119` | ℕ |
| ℤ | `symbol-120` | ℤ |
| ℚ | `symbol-121` | ℚ |
| ℂ | `symbol-122` | ℂ |
| ℙ | `symbol-123` | ℙ |
| ℍ | `symbol-124` | ℍ |
| ℵ | `symbol-125` | ℵ |
| ℏ | `symbol-126` | ℏ |
| ℓ | `symbol-127` | ℓ |
| ℜ | `symbol-128` | ℜ |
| ℑ | `symbol-129` | ℑ |
| ∅ | `symbol-130` | ∅ |
| ° | `symbol-131` | ° |

## Core-only formatting and aliases

These entries are available through the core API, not the mouse keypad.

| Notation | Identifier | Description |
| --- | --- | --- |
| asinh | `fn-asinh` | asinh · 数学函数名称 |
| acosh | `fn-acosh` | acosh · 数学函数名称 |
| atanh | `fn-atanh` | atanh · 数学函数名称 |
| asech | `fn-asech` | asech · 数学函数名称 |
| acsch | `fn-acsch` | acsch · 数学函数名称 |
| acoth | `fn-acoth` | acoth · 数学函数名称 |
| sign | `fn-sign` | sign · 数学函数名称 |
| factorial | `fn-factorial` | factorial · 数学函数名称 |
| 下划线 | `underline` | 可编辑数学重音 |
| 粗体 | `bold` | 粗体 · 可编辑数学字母 |
| 黑板体 | `blackboard` | 黑板体 · 可编辑数学字母 |
| 花体 | `calligraphic` | 花体 · 可编辑数学字母 |
| 哥特体 | `fraktur` | 哥特体 · 可编辑数学字母 |
| 斜体 | `italic` | 斜体 · 可编辑数学字母 |
| 无衬线 | `sans-serif` | 无衬线 · 可编辑数学字母 |
| 等宽 | `monospace` | 等宽 · 可编辑数学字母 |

# Structure of Programming Languages — Reviewer

Token-dense compilation of Axiomatic Semantics, Program Correctness, and Denotational Semantics. Optimized for agent retrieval during examinations.

---

## 1. Three Frameworks for Defining Semantics

1. **Operational Semantics**
2. **Axiomatic Semantics**
3. **Denotational Semantics**

A programming language is well-defined when its **syntax**, **type system**, and **semantics** are defined (Tucker 2007). Semantics = precise definition of the meaning of any correct program in the language. Meaning of a program = what happens when it is translated by a compiler and executed by a machine.

---

## 2. Axiomatic Semantics

### Definition
- Method of describing semantics of a program using **logical assertions**
- Meaning is given by a statement (assertion) showing the value(s) of variable(s) involved

### Assertion
- A logical statement expressing the value(s) of the program variables at a point
- Written in braces `{...}`
- Two uses:
  1. Specify the meaning of a program
  2. **Prove the correctness of a program**

### Example assertions
```
{j=3 ∧ k=4}     ← initial assertion (precondition)
  j = j+k
{j=7 ∧ k=4}     ← final assertion (postcondition)
```

```
{x=A ∧ y=B}
  z = x
  x = y
  y = z
{x=B ∧ y=A}     ← variable swap proved
```

---

## 3. Assessing Program Quality

Three questions:
1. **Is the program well written?** Style, clarity, ease of modification. Subjective, no formal method.
2. **Is the program efficient?** Cost of execution: storage + execution time. Handled by **algorithm analysis**: Big O `O(n)`, Big Theta `Θ(n)`, Big Omega `Ω(n)`.
3. **Does the program do what it is supposed to do?** This is **program correctness**.

### Program errors
- **Syntactic error**: violates the definition of a well-formed program. Detected by the language translator (compiler / interpreter); usually easy to correct.
- **Logical error**: program runs but produces the wrong result. Detected via **program verification**.

### Program Verification
Technique used to establish program correctness by testing for errors in logic. Sometimes called **partial correctness** (does not prove termination, only that if it terminates, the result is correct).

---

## 4. Correct Program Segment (Definition)

A program segment **P** is correct with respect to an **Initial Assertion AI** and **Final Assertion AF** iff:
- whenever AI is true of the program variables before execution of P, AND
- P terminates,
- THEN AF will be true of the program variables after the execution of P.

### Notation
- `{AI} P {AF}`  or  `AI / P / AF`  (Hoare triple)

### Assertion types
- **Initial Assertion (AI / precondition)**: what is known / assumed about variables BEFORE execution. If no assumption, AI = `TRUE`.
- **Final Assertion (AF / postcondition)**: what is true about variables AFTER program terminates normally.
- **Intermediate Assertion**: what is true after a non-final statement.

---

## 5. Program Correctness via Backward Axiom of Assignment

### Core idea
A program is correct if the **Initial Assertion can be "computed" from the given Final Assertion** by working backward through the program statements ("backward tracing").

### Backward Axiom of Assignment (formal)
Let `x := E` be an assignment statement and `Q` be the final assertion.
Then the initial assertion is `P = Q[x→E]` (Q with all instances of x replaced by E).

> If logical statement Q is true after `x := E` executes, then the statement that is true before is derived from Q by replacing x by E.

### General form
```
P: {A(x1, x2, …, xi-1, ε(x1,…,xn), xi+1, …, xn)}
S:  xi := ε(x1, x2, …, xi-1, xi, xi+1, …, xn)
Q: {A(x1, x2, …, xi-1, xi, xi+1, …, xn)}
```

### Step-by-step procedure
1. Formulate AI + AF that characterize the task.
2. Divide program into segments; for each, formulate AI and AF. If S2 follows S1, then AF of S1 ⇒ AI of S2.
3. Apply Backward Axiom of Assignment to each segment.
4. Conclude correctness.

The first-statement Initial Assertion (the **precondition of the program**) should be the **weakest precondition**.

---

## 6. Worked Example — Backward Tracing

Show that the swap is correct:

```
AI: {x=1 ∧ y=2}
   t := x
   x := y
   y := t
AF: {x=2 ∧ y=1}
```

Step 1 — Compute IA of last statement (`y := t`) using AF:
Replace `y` with `t` in `{x=2 ∧ y=1}` → `{x=2 ∧ t=1}`

Step 2 — Compute IA of `x := y` using `{x=2 ∧ t=1}`:
Replace `x` with `y` → `{y=2 ∧ t=1}`

Step 3 — Compute IA of `t := x` using `{y=2 ∧ t=1}`:
Replace `t` with `x` → `{y=2 ∧ x=1}` which equals `{x=1 ∧ y=2}` (commutativity of ∧).

This matches the given AI → **program is correct**.

Logical flow direction: assertions are checked **top to bottom**; trace is computed **bottom to top**.

---

## 7. Simple Backward-Axiom Examples

```
{? }           →   {2*x+1 > 1}   →   {x > 0}  (since 2x>0 ⇒ 2x+1>1)
  sum = 2*x+1
{sum > 1}
```

Common exercises:
- `{?} x = x+3 {x=5}` → IA: `{x = 2}`
- `{x=? ∧ y=?} x = x+y {x=7 ∧ y=4}` → IA: `{x=3 ∧ y=4}`
- `{?} x = y+1 {x=6 ∧ y=5}` → IA: `{y=5}`
- `{2=2} x:=3 {x=3}` → `2=2` is trivially true, so valid
- `{x+y+z=5} x:=x+y+z {x=5}` → directly valid
- `{x=y ∧ x<0} x:=-x {y=-x ∧ x>0}` → valid

---

## 8. Rules of Inference for Axiomatic Semantics (5 rules)

### Rule 1 — Rule of Composition
```
A1 {S1} A2
A2 {S2} A3
─────────────
∴ A1 {S1; S2} A3
```

Example: if `{True} x:=2 {x=2}` and `{x=2} y:=x+z {y=z+2}` are both correct, then `{True} x:=2; y:=x+z {y=z+2}` is correct.

### Rule 2 — Rule of Consequence

**First form** (strengthen precondition):
```
A1 ⇒ A2
A2 {S} A3
─────────
∴ A1 {S} A3
```

**Second form** (weaken postcondition):
```
A1 {S} A2
A2 ⇒ A3
─────────
∴ A1 {S} A3
```

Example: if `{x<0} y:=-x {y>0}` is correct and `¬(x≥0) ⇒ x<0`, then `{¬(x≥0)} y:=-x {y>0}` is correct.

### Rule 3 — If-Then Rule
```
(A1 ∧ Condition) {S} A2
(A1 ∧ ¬Condition) ⇒ A2
─────────────────────
∴ A1 {if Condition then S} A2
```

Must show:
1. When Condition true: executing S from A1 ∧ Condition yields A2.
2. When Condition false: A1 ∧ ¬Condition already implies A2 (S is skipped).

Example: `{TRUE} if x>y then y:=x {y ≥ x}` proven by:
- (i) `{TRUE ∧ x>y} y:=x {y ≥ x}` — true since y=x ⇒ y≥x
- (ii) `TRUE ∧ ¬(x>y) ⇒ y ≥ x` — true since ¬(x>y) ⇔ x≤y ⇔ y≥x

### Rule 4 — If-Then-Else Rule
```
(Q1 ∧ Condition) {S1} Q2
(Q1 ∧ ¬Condition) {S2} Q2
─────────────────────────
∴ Q1 {if Condition then S1 else S2} Q2
```

Example: prove `{true} if x<0 then r:=-x else r:=x {r=|x|}`:
- (i) `{true ∧ x<0} r:=-x {r=|x|}` — since x<0 ⇒ -x=|x|
- (ii) `{true ∧ ¬(x<0)} r:=x {r=|x|}` — since x≥0 ⇒ x=|x|

### Rule 5 — Rule of Iteration (while-loop)
```
(Q ∧ Condition) {S} Q
────────────────────────────────────
∴ Q {while Condition do S} (Q ∧ ¬Condition)
```

**Q is the loop invariant.**

---

## 9. Loop Invariant

- **Invariant**: a condition that does not change.
- **Loop invariant**: a condition that is true before the loop AND remains true after each execution of the loop body.
- The loop invariant is the **weakened postcondition** that is also a precondition for the loop.
- Must be:
  - **Weak enough** to be satisfied before the loop begins
  - **Strong enough** that, combined with the exit condition (¬B), it forces the truth of the desired postcondition

### Finding the loop invariant
Similar to inductive hypothesis in mathematical induction:
1. Compute the relationship for a few iterations
2. Start with the postcondition; determine the precondition when the loop body isn't executed
3. Determine the precondition after 1st, 2nd, 3rd iteration
4. Recognize the pattern

### Example
```
{?}
while (s > 1)
   s = s/2
{s = 1}
```
- 0 iter: s=1
- 1 iter: s/2=1 → s=2
- 2 iter: s/2=2 → s=4
- 3 iter: s/2=4 → s=8
- Pattern: s = powers of 2, s ≥ 1 → **loop invariant: s ≥ 1** (and s is a power of 2)

### Example 2
```
while (y != x)
   y = y+1
{y = x}
```
- 0 iter: y=x
- 1 iter: y+1=x → y=x-1
- 2 iter: y=x-2
- Pattern: y ≤ x → **loop invariant: y ≤ x**

---

## 10. Proving Loop Correctness — Full Procedure

Given precondition P, loop invariant I, loop body S, loop condition B:

1. **Show P ⇒ I** (weakest precondition guarantees the loop invariant)
2. **Show `{I ∧ B} S {I}` is correct** (body preserves invariant)
3. **Show `(I ∧ ¬B) ⇒ Q`** (invariant + exit condition implies desired postcondition)

### Worked example — factorial program

```
{n > 0}
i := 1
factorial := 1
                            ← Loop Invariant: factorial = i! ∧ i ≤ n
while i < n do
   i := i+1
   factorial := factorial * i
end
{factorial = n!}
```

**Step 1** — Show `{n > 0}` ⇒ `{factorial=1 ∧ i=1 ∧ n>0}` after `i:=1; factorial:=1` (trivial; 1 = 1!, 1 ≤ n).

**Step 2** — Show body preserves invariant:
- Start: `{factorial = i! ∧ i ≤ n ∧ i < n}` → `{factorial = i! ∧ i ≤ n-1}` → `{factorial · (i+1) = (i+1)! ∧ (i+1) ≤ n}`
- After `i := i+1`: `{factorial · i = i! ∧ i ≤ n}`
- After `factorial := factorial * i`: `{factorial = i! ∧ i ≤ n}` ✓

**Step 3** — Exit: `{factorial = i! ∧ i ≤ n ∧ ¬(i < n)}` = `{factorial = i! ∧ i = n}` = `{factorial = n!}` ✓

---

## 11. Quick exam template — Loop Correctness

| Symbol | Meaning |
|---|---|
| P | precondition (given) |
| I | loop invariant (guess + verify) |
| B | loop condition |
| S | loop body |
| Q | postcondition (given) |

Three obligations:
1. `P ⇒ I`
2. `{I ∧ B} S {I}`
3. `(I ∧ ¬B) ⇒ Q`

---

# Part B — Denotational Semantics

---

## 12. Denotational Semantics — Core Idea

**Keywords: State, Function**

- Expresses the **meaning of a program as a collection of functions** operating on the **state** of the program.
- Defines the meaning of each program statement via a **state-transforming mathematical function**.
- Maps abstract language elements onto **state-transforming functions**.

### Requirements
- Mathematical entities (integers, real numbers, characters, booleans) and their properties must be defined.
- A precise **model of state** is needed.
- **Partial functions** describing state transformations are established.
- Constraints on real computers (e.g. 8-bit integer range = -128 to 127) temper the math.

### Standard signatures (Tucker 2007)
- `M: Program → State`
- `M: Statement × State → State`
- `M: Expression × State → Value`

---

## 13. Components of Denotational Semantics

1. **Production Rules** (grammar)
2. **Syntactic Domain**
3. **Semantic Domain**
4. **Semantic Function**
5. **Auxiliary Function**
6. **Semantic Equation**

---

## 14. State, Partial Function

- **State**: set of (variable, current value) pairs.
  `State = {(var1, val1), (var2, val2), …, (varn, valn)}`
- **Partial Function**: function not defined on all possible inputs.

---

## 15. Semantic Domains

A **semantic domain** is a set of values whose properties and operations are independently well-understood. Used as the basis/bases of semantic rules.

### Domain types
- **Primitive Domain**: elements are primitive values (not composed of simpler values)
  - **Character**: from a character set
  - **Integer**: `{…,-2,-1,0,1,2,…}`
  - **Natural**: nonnegative integers
  - **Boolean**: `{true, false}`
- **Function Domain** `A → B`: each element is a function mapping A to B
  - Example: `Integer → Boolean` = functions like `isOdd`, `isEven`
  - `Store: Variable → Integer` (binding variable names to values)
- **Cartesian Product Domain** `A × B`: elements are ordered pairs `{(x, y) | x ∈ A, y ∈ B}`
  - General: `D1 × D2 × … × Dn = {(x1, x2, …, xn)}`
- **Disjoint Union Domain** `A + B`: elements chosen from either component
- **Sequence Domain** `D*`: finite sequences of zero or more elements of D
  - Example: `String = Character*`

### Bounded domains
`N⁰` (with ° superscript) means the integer set has a **greatest lower bound** and **least upper bound**.

---

## 16. Semantic Function

Maps syntactic objects to objects in semantic domains. Each semantic domain typically has one semantic function.

Examples:
- `Value: Expression → Integer`
- `Dig: Digit → Integer`
- `Program: Program → (Input → Output)`

---

## 17. Notation `[[X]]`

`[[ ]]` is a **metasymbol** called **semantic braces** or **interpretation function**.

- `[[X]] = v` means "the semantics of program fragment X is mathematical object v".

---

## 18. Semantic Equation

Specifies how a semantic function acts on each construct of the language. **One semantic equation per production rule.**

### Simple example
- Production: `Digit ::= '0' | '1' | … | '9'`
- Semantic function: `Dig: Digit → Integer`
- Equations:
  ```
  Dig[['0']] = 0
  Dig[['1']] = 1
  …
  Dig[['9']] = 9
  ```

### Number example
- Production: `Number ::= Number Digit | Digit`
- Semantic function: `Num: Number → Integer`
- Equations:
  ```
  Num[[Number Digit]] = 10 * Num[[Number]] + Num[[Digit]]
  Num[[Digit]] = Dig[[Digit]]
  ```

---

## 19. Auxiliary Function

Predefined mathematical operations used inside semantic functions (the "plumbing").

Common ones:
- `plus: N × N → N` (e.g. (1,2) → 3)
- `minus: N × N → N` (e.g. (5,3) → 2)
- `times: N × N → N` (e.g. (2,3) → 6)
- `divide: N × N → N` (e.g. (9,3) → 3; uses floor `⌊q⌋`)

---

## 20. Canonical Example — Integer Arithmetic Expressions

### Production rules
```
E ::= E+E | E-E | E*E | E/E | (E) | C
C ::= C D | D
D ::= 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9
```

### Syntactic domains
- E : Expression
- C : Constant
- D : Digits

### Semantic domain
- `N = {…, -3, -2, 0, 1, 2, …}⁰`

### Semantic functions
- `ε : Expression → N`
- `τ : Constant → N`
- `δ : Digits → N`

### Auxiliary functions
- `plus, minus, times, divide : N × N → N`

### Semantic equations
```
a.  ε[[E+E]]  = plus(  ε[[E]], ε[[E]] )
b.  ε[[E-E]]  = minus( ε[[E]], ε[[E]] )
c.  ε[[E*E]]  = times( ε[[E]], ε[[E]] )
d.  ε[[E/E]]  = divide(ε[[E]], ε[[E]] )
e.  ε[[(E)]]  = ε[[E]]
f.  ε[[E]]    = τ[[C]]              (when E is a constant)
g.  τ[[C]]    = τ[[CD]]
h.  τ[[C]]    = δ[[D]]              (when C is a single digit)
i.  τ[[CD]]   = plus(times(10, τ[[C]]), δ[[D]])
j-s. δ[[0]] = 0 … δ[[9]] = 9
```

---

## 21. Worked Derivation — `[[25 * 3]]`

```
ε[[25*3]] 
  =c    times(ε[[25]], ε[[3]])
  =f    times(τ[[25]], τ[[3]])
  =i&h  times(plus(times(10, τ[[2]]), δ[[5]]), δ[[3]])
  =h,o,m times(plus(times(10, δ[[2]]), 5), 3)
  =l    times(plus(times(10, 2), 5), 3)
  =times times(plus(20, 5), 3)
  =plus times(25, 3)
  =times 75
```

**Result: ε[[25 * 3]] = 75**

Convention notation:
- `e1 =a e2` — applied semantic equation a
- `e1 =a&b e2` — applied both a and b
- Depth-first application; evaluate subexpressions as early as possible.

---

## 22. Worked Derivation — `[[38 + (3 * 5)]]`

```
ε[[38+(3*5)]] 
  =a     plus(ε[[38]], ε[[(3*5)]])
  =f,c   plus(τ[[38]], times(ε[[3]], ε[[5]]))
  =i,f   plus(plus(times(10, τ[[3]]), δ[[8]]), times(τ[[3]], τ[[5]]))
  =h,r   plus(plus(times(10, δ[[3]]), 8), times(δ[[3]], δ[[5]]))
  =m,o   plus(plus(times(10, 3), 8), times(3, 5))
  =times plus(plus(30, 8), 15)
  =plus  plus(38, 15)
  =plus  53
```

**Result: 53**

---

## 23. Denotational Semantics with Programs (Bigger Picture)

### Statement type syntax
```
Statement ::= Skip | Block | Assignment | Conditional | Loop
Skip       ::= (empty)
Block      ::= Statement*
Assignment ::= Variable target ; Expression source
Conditional ::= Expression test ; Statement thenBranch, elseBranch
Loop       ::= Expression test ; Statement body
```

### Meaning of a statement
```
M : Statement × State → State

M(s, state) =
   M(Skip s, state)        if s is Skip
   M(Assignment s, state)  if s is Assignment
   M(Conditional s, state) if s is Conditional
   M(Loop s, state)        if s is Loop
   M(Block s, state)       if s is Block
```

### Full mini-language syntax
```
P ::= S
S ::= V := E | read(V) | write(V) | while C do S od | S; S
C ::= E1 < E2 | E1 > E2 | E1 = E2 | E1 ≤ E2 | E1 ≥ E2 | E1 ≠ E2
E ::= T | E+T | E-T
T ::= F | T*F | T/F
F ::= (E) | cons | V
```

### Syntactic domains
P (Program), S (Statement), C (Condition), E (Expression), T (Term), F (Factor)

### Semantic domains
- `τ: B = {true, false}⁰` (booleans)
- `φ: Fi = N*` (file as sequence of numbers)
- `γ: CF = St × Fi × Fi` (configuration: state × input × output)
- `σ: St = V → N` (state binds variables to numbers)
- `ν: N = {…, -2, -1, 0, 1, 2, …}⁰`

### Semantic function domains
- `M: Program → Fi → Fi`
- `Σ: Statement → CF → CF`
- `ς: Condition → St → B`
- `ε: Expression → St → N`
- `α: Term → St → N`
- `β: Factor → St → N`

### Sample semantic equation (with state)
```
Evaluate[[E1 + E2]] Store = plus(Evaluate[[E1]] Store, Evaluate[[E2]])
```
The value of `E1 + E2` is the sum of the values of its components. Both depend on the current variable bindings represented by **Store**.

---

## 24. Conventions for Evaluating Semantic Equations

1. **Depth-first** application of semantic equations (not breadth-first).
2. Integer expressions and subexpressions are **evaluated at the earliest possible point** in the derivation.

---

## 25. Cheat sheet — quick recall

### Axiomatic
| Concept | Form |
|---|---|
| Hoare triple | `{P} S {Q}` |
| Backward assignment | `{Q[x→E]} x := E {Q}` |
| Composition rule | `{A1}S1{A2}, {A2}S2{A3} ⇒ {A1}S1;S2{A3}` |
| Consequence (1) | `A1⇒A2, {A2}S{A3} ⇒ {A1}S{A3}` |
| Consequence (2) | `{A1}S{A2}, A2⇒A3 ⇒ {A1}S{A3}` |
| If-then | needs branch + skip case |
| If-then-else | needs both branch cases |
| While loop | `{I∧B}S{I} ⇒ {I} while B do S {I∧¬B}` |
| Loop correctness | (1) P⇒I (2) `{I∧B}S{I}` (3) `(I∧¬B)⇒Q` |
| Recommended IA | **weakest precondition** |

### Denotational
| Concept | Notation |
|---|---|
| Semantic braces | `[[X]]` |
| Meaning of X is v | `[[X]] = v` |
| Program meaning | `M: Program → State` |
| Statement meaning | `M: Statement × State → State` |
| Expression meaning | `M: Expression × State → Value` |
| Annotation | `e1 =a e2` (applied rule a) |
| Multi-rule | `e1 =a&b e2` |
| Six components | Production rules, Syntactic Domain, Semantic Domain, Semantic Function, Auxiliary Function, Semantic Equation |

### Common semantic-equation patterns
```
ε[[E+E]] = plus(ε[[E]], ε[[E]])
τ[[CD]] = plus(times(10, τ[[C]]), δ[[D]])
δ[[d]] = d  (for digit d)
```

---

## 26. Reference

Tucker, Allan and Robert Noonan. (2007). *Programming Languages Principles and Paradigms*. 2nd Edition. McGraw-Hill Education.

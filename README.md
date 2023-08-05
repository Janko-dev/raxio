# Raxio

The domain specific language (DSL) `raxio` is a symbolic pattern matching tool, in which variables and functors can be constructed and symbolically manipulated with rules. The core features of the language include the ability to express algebraic symbols with arbitrary arguments (functors) and atomic symbols (variables), and pattern match against them by defining either predefined rule definitions or in-line rules. Currently, the project is capable of assisting in simple mathematical proofs. However, it is not meant as a serious proof assistant, but rather, as a simple command line tool, which I created for recreational and educational purposes.

The application is written in Rust and used via the command line interface. The components of `raxio` follow the typical interpreter pipeline, i.e., lexing the input text to obtain a stream of tokens, parsing the tokens into data structures that express statements, and interpreting the statements to manipulate the state in the environment.  

## Usage
To build the project, install [Rust/Cargo](https://www.rust-lang.org/tools/install) and run the following in the terminal. 

```console
$ git clone https://github.com/Janko-dev/raxio
$ cd raxio
$ cargo build --release 
``` 
go to `target/release/` and run the `raxio` executable. Either pass a filename as the argument to interpret the entire file or pass no arguments to enter the REPL (Read-Evaluate-Print-Loop) environment. 
```console
$ ./raxio [FILE_NAME] 
```

## Examples

### Simple algebraic swap of symbols

To illustrate the syntax of symbolic manipulation in `raxio`, consider the simple example of the symbols `pair(A, B)`. We can define a rule to swap the values of `A` and `B`.
```console
def swap as pair(x, y) => pair(y, x) 
```
The rule is defined by the identifier `swap` and will try to match the left-hand-side (lhs) to the expression (in this case `pair(A, B)`). If matching is successful, then the corresponding right-hand-side (rhs) will be produced with the correct symbols. Therefore, we get the following in the REPL:
```console
> def swap as pair(x, y) => pair(y, x)
> pair(A, B) 
Start matching: pair(A, B)
    ~ swap 0
    pair(B, A)
    ~ end
Result: pair(B, A)
>
``` 
After defining `swap`, we can start pattern matching by stating an expression, i.e., `pair(A, B)`. This will start an environment (prefixed by the tilde ~), which allows us to apply either defined rules (such as `swap`) or in-line rules. In the above, we apply swap and further specify a number, 0, which indicates the depth to match for patterns. Depth 0 indicates that we wish to match on the entire expression. 

### Power rule of calculus

As another example, consider the power rule of calculus 
$$f(x) = x^n \rightarrow f'(x) = nx^{n-1}$$
We can define the power rule as the following using functors, albeit as of yet readability is still a bit lacking. 
```console
def diff_power_rule as pow(x, n) => mul(n, pow(x, sub(n, 1)))
```
With this rule we can symbolically show that the derivative of $y^2$ is equal to $2y$.
```console
> def diff_power_rule as pow(x, n) => mul(n, pow(x, sub(n, 1)))
> pow(y, 2) 
Start matching: pow(y, 2)
    ~ diff_power_rule 0
    mul(2, pow(y, sub(2, 1)))
    ~ sub(2, 1) => 1, 2
    mul(2, pow(y, 1))
    ~ pow(x, 1) => x, 1
    mul(2, y)
    ~ end
Result: mul(2, y)
>
```
Note that we simplified further by using the in-line rules, i.e., `sub(2, 1) => 1, 2` and `pow(x, 1) => x, 1`. Trivially, `sub(2, 1) => 1, 2` states that $2-1 \implies 1$. The `2` after the comma indicates the depth to pattern match. Similarly, `pow(x, 1) => x, 1` denotes $x^1 \implies x$. 

A final remark on the syntax, when specifying depth, use comma during in-line pattern matching and without the comma when calling a defined rule. 

## Grammar
Consider the following grammar that describes the syntax of `raxio` in a variant of [EBNF](https://en.wikipedia.org/wiki/Extended_Backus%E2%80%93Naur_form).

```ebnf
Stmt         := Rule | 
                Define | 
                Expr |
                End ; 

Rule         := Expr "=>" Expr "," Number;
Define       := "def" Identifier "as" Expr "=>" Expr ;
Expr         := FunctorExpr | 
                VariableExpr ;
End          := "end" ;  

FunctorExpr  := Identifier "(" (Expr ",")* ")" ;
VariableExpr := Identifier (Number)?;

Number       := ("0"-"9") ("0"-"9")* ;
Identifier   := ("a"-"z" | "A"-"Z" | "_" ) ("a"-"z" | "A"-"Z" | "_" | Number)* ;
```
# Raxio

`raxio` is a domain specific language (DSL) that allows for symbolic pattern matching. At its core lies the idea of representing some expression that consists of algebraic symbols with arbitrary arguments (functors) and atomic symbols (variables), and thereafter apply rules to manipulate the symbolic expression. In other words match against patterns of a rule and then produce the corresponding result of the transformation. This can be done by either defining rules definitions that are bound to an identifier or applying in-line rules directly without first defining an identifier. See the the [how it works](#how-it-works) section, the [examples](#examples) and the [grammar](#grammar) for more details on the syntax.

Currently, the project is capable of assisting in simple mathematical proofs. However, it is not meant as a serious proof assistant, but rather, as a simple command line tool, which I created for recreational and educational purposes.

The application is written in Rust and used via the command line interface. The components of `raxio` follow the typical interpreter pipeline, i.e., lexing the input text to obtain a stream of tokens, parsing the tokens into data structures that express statements, and interpreting the statements to manipulate the state in the runtime environment.

## Todo's
- [x] Add in-fix math operators `+`, `-`, `*`, and `/`, during lexing and parsing that get translated to `add()`, `sub()`, `mul()`, and `div()`, respectively. Will help with readability. 
- [ ] Maintain history of manipulated expressions with the corresponding applied rule, so that the history can be written to a file. 
- [ ] Add string literal after end-statement indicating the path to write the derivation of the expression, i.e., to write the history. For example, the syntax would look like `end "path/to/file"`. 
- [ ] Review and evaluate the usability of the grammar.

## Usage
To build the project, install [Rust/Cargo](https://www.rust-lang.org/tools/install) and run the following in the terminal. 

```bash
$ git clone https://github.com/Janko-dev/raxio
$ cd raxio
$ cargo build --release 
``` 
go to `target/release/` and run the `raxio` executable. Either pass a filename as the argument to interpret the entire file or pass no arguments to enter the REPL (Read-Evaluate-Print-Loop) environment. 
```bash
$ ./raxio [FILE_NAME] 
```
```bash
$ ./raxio
Welcome to the REPL environment of raxio.
Enter "quit" to stop the REPL environment.
>
```

## How it works

We can create an expression by either entering the name of an identifier, which will act as a variable, and thus start the pattern matching environment, or we can enter an identifier and subsequently provide a comma separated list of other expressions enclosed in parentheses. The latter is denoted as a functor. For example, the expression `f(x)` is a functor with identifier `f` and a single argument `x`. This form of expression is akin to the idea of an operator which performs some operation on the operands, i.e., on the arguments. 

This can be arbitrarily complex. For example, to denote the trivial expression $x^2$, we can choose to denote exponentiation as `pow(a, b)` where `a` is the base and `b` is the exponent. 

The other, arguably more interesting part of this application, is the ability to pattern matchthe expressions that you create. The syntax to define a **rule** is as follows
```bash
def RULE_NAME as LEFT_HAND_SIDE_EXPR => RIGHT_HAND_SIDE_EXPR
```
the `LEFT_HAND_SIDE_EXPR` will first be matched against the expression in question. If this succeeds, then the `RIGHT_HAND_SIDE_EXPR` will be used to produce the correct expression. As an example consider the simple the symbols `pair(A, B)`. We can define a rule to swap the values of `A` and `B`.
```bash
def swap as pair(x, y) => pair(y, x) 
```
The rule is defined by the identifier `swap`. In the REPL, the following is thus derived. 
```bash
> def swap as pair(x, y) => pair(y, x)
> pair(A, B) 
Start matching: pair(A, B)
    ~> swap 0
    pair(B, A)
    ~> pair(x, y) => pair(y, x), 0
    pair(A, B)
    ~> end
Result: pair(B, A)
>
```

After defining `swap`, we can start pattern matching by stating an expression, i.e., `pair(A, B)`. This will start an environment (prefixed by `~> `), which allows us to apply either defined rules (such as `swap`) or in-line rules. In the above, we apply swap and further specify a number, 0, which indicates the depth to match for patterns. Depth 0 indicates that we wish to match on the entire expression. The second application of a rule is an in-line rule, analogous to anonymous functions/lambda's in that these are executed in-line directly. The important note is that the depth is specified differentlty for predefined rules and in-line rules. In the former, simply provide some whitespace, and in the latter, it is necessary to provide a comma to seperate the right hand side expression from the depth value.

## Examples

### Power rule of calculus

Consider the power rule of calculus 
$$f(x) = x^n \rightarrow f'(x) = nx^{n-1}$$
We can define the power rule as the following using functors.
```bash
def diff_power_rule as pow(x, n) => mul(n, pow(x, sub(n, 1)))
```
With this rule we can symbolically show that the derivative of $y^2$ is equal to $2y$.
```bash
Welcome to the REPL environment of raxio.
Enter "quit" to stop the REPL environment.
> def diff_power_rule as pow(x, n) => mul(n, pow(x, sub(n, 1)))
> def rule_2_minus_1 as sub(2, 1) => 1
> def rule_pow_1 as pow(x, 1) => x
> pow(y, 2)
Start matching: pow(y, 2)
    ~> diff_power_rule 0
    mul(2, pow(y, sub(2, 1)))
    ~> rule_2_minus_1 2
    mul(2, pow(y, 1))
    ~> rule_pow_1 1
    mul(2, y)
    ~> end
Result: mul(2, y)
>
```

### Proof of power rule

In the above example, the power rule is defined as a formula to transform an expression of exponentiation into the derivative of that expression. We can also use `raxio` to show the proof of the power rule by using the limit definition of the derivative

$$
    \lim_{h \to 0} \frac{f(x + h) - f(x)}{h}
$$

If we assume $f(x) = x^2$, then we can work this out to equal $f'(x) = 2x$. Using `raxio`, we can achieve the same systematically. Albeit, readability for this example is lacking.  

```bash
> lim(h, 0, div(sub(f(add(x, h)), f(x)), h))
Start matching: lim(h, 0, div(sub(f(add(x, h)), f(x)), h))
    ~> f(x) => pow(x, 2), 3
    lim(h, 0, div(sub(pow(add(x, h), 2), pow(x, 2)), h))
    ~> pow(add(a, b), 2) => add(pow(a, 2), mul(2, a, b), pow(b, 2)), 3 
    lim(h, 0, div(sub(add(pow(x, 2), mul(2, x, h), pow(h, 2)), pow(x, 2)), h))
    ~> sub(add(a, b, c), d) => add(a, b, c, neg(d)), 2
    lim(h, 0, div(add(pow(x, 2), mul(2, x, h), pow(h, 2), neg(pow(x, 2))), h))
    ~> add(a, b, c, neg(a)) => add(b, c), 2
    lim(h, 0, div(add(mul(2, x, h), pow(h, 2)), h))
    ~> div(add(a, b), c) => add(div(a, c), div(b, c)), 1
    lim(h, 0, add(div(mul(2, x, h), h), div(pow(h, 2), h)))
    ~> div(pow(a, 2), a) => a, 2
    lim(h, 0, add(div(mul(2, x, h), h), h))
    ~> div(mul(2, b, a), a) => mul(2, b), 2
    lim(h, 0, add(mul(2, x), h))
    ~> lim(t, 0, add(a, t)) => a, 0
    mul(2, x)
    ~> end
Result: mul(2, x)
>
```

## Grammar
Consider the following grammar that describes the syntax of `raxio` in a variant of [EBNF](https://en.wikipedia.org/wiki/Extended_Backus%E2%80%93Naur_form).

```ebnf
Stmt         := Rule   | 
                Define | 
                Expr   |
                End    |
                "quit" |
                "undo" |
                "help" ; 

Rule         := Expr "=>" Expr "," Number;
Define       := "def" Identifier "as" Expr "=>" Expr ;
Expr         := FunctorExpr | 
                VariableExpr ;
End          := "end" Path ;

Path         := "\"" ("/")? String ("/" String)* "\"" ;

FunctorExpr  := Identifier "(" (Expr ",")* ")" ;
VariableExpr := Identifier (Number)?;

Number       := ("0"-"9") ("0"-"9")* ;
Char         := ("a"-"z" | "A"-"Z" | "_" )
String       := Char*
Identifier   := Char (Char | Number)* ;
```

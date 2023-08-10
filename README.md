# Raxio

`raxio` is a domain specific language (DSL) that allows for symbolic pattern matching. At its core lies the idea of representing some expression consisting of algebraic symbols either by an identifier accompanied by a set of arbitrary sub-expression arguments (i.e., functors) or by an identifier acting as an atomic symbol (variables). Furthermore, rules can be defined that map a pattern to another pattern. Using these rules, a given expression can be transformed and manipulated with the goal of deriving some target expression or proving some mathematical form. Rules can either be bound to an identifier and called at a later point during the pattern matching sequence, or they can be defined directly on a given expression, which are denoted as in-line rules. Consider reading up on the [syntax](#syntax) and reviewing the [examples](#examples).

I wrote this DSL purely for educational and recreational purposes. Even though `raxio` is capable of assisting in some mathematical proofs, as shown in the examples section, it is not meant as a serious tool with capabilities similar [Coq](https://github.com/coq/coq) or [Isabelle](https://isabelle.in.tum.de/). Nevertheless, `raxio` requires not much setup and its REPL environment provide the user with simple hands-on pattern matching. 

The application is written in Rust and requires a command line interface. The components of `raxio` follow the typical interpreter pipeline, i.e., lexing the input text to obtain a stream of tokens, parsing the tokens into data structures that express statements, and interpreting the statements to manipulate the state in the runtime environment.

## Usage
To build the project, install [Rust/Cargo](https://www.rust-lang.org/tools/install) and run the following in the terminal. 

```bash
$ git clone https://github.com/Janko-dev/raxio
$ cd raxio
$ cargo build --release 
``` 
go to `target/release/` and run the `raxio` executable. Either pass a filename (with the convention of having the `.rx` extension) as the argument to interpret the entire file or pass no arguments to enter the REPL (Read-Evaluate-Print-Loop) environment.
```bash
$ ./raxio [FILE_NAME] 
```
```bash
$ ./raxio
Welcome to the REPL environment of raxio.
Enter "quit" to stop the REPL environment.
Enter "help" to see an overview of raxio syntax.
Enter "undo" during mattern patching to undo the current expression.
>
```

## Syntax

### Expressions
Expressions are either symbolic words (variables), such as `foo_42`, `x`, `this_is_a_SYMBOL`, or they are symbolic words followed by a list of comma-separated expressions that are between parentheses (functors), such as `f(x)`, `foo(bar, baz(y))`, `print(x, y, z)`. The recursive nature of this definition allows for arbitrary complex expressions. The meaning of these symbols can thus also be arbitrary. `f(x)` could be a mathematical function that performs some set of operations on `x` to produce a value. `print(hello_world)` could be a procedure that manipulates the internal state by printing the contents of `hello_world` to the terminal. The aforementioned semantics are irrelevant in `raxio`. Instead, the focus is on the formal symbolic form of the expression. 

### Rules
Being able to only define expressions is not that useful. Therefore, the syntax extends to be able to manipulate a given expression within a pattern matching context. This is denoted if an expression is entered in the REPL.
```bash
$ ./raxio
Welcome to the REPL environment of raxio.
Enter "quit" to stop the REPL environment.
Enter "help" to see an overview of raxio syntax.
Enter "undo" during mattern patching to undo the current expression.
> f(x) 
Start matching on: f(x)
    ~>
```
The prompt (`~>`) is asking for either a predefined rule at some depth or an in-line rule at some depth, so that the expression `f(x)` can be matched and possibly transformed. Predefined rules can (surprisingly) also be defined inside a pattern matching context. For example, to define a rule `foo` that matches on th expession `f(a)` and transforms the expression into `g(a, a)`, enter the following in the REPL.
```bash
def foo as f(a) => g(a, a)
```
To see the rule in action, apply it inside a pattern matching context accompanied with the depth level of the expression to match.
```bash
> def foo as f(a) => g(a, a)
> f(x) 
Start matching on: f(x)
    ~> apply foo at 0                   // defined rule at depth 0
    g(x, x)
    ~>
```
Here, the `at 0` indicates to match on the zero'th depth, which is the entire expression. In-line rules follow similar syntax without the binding to an identifier, such as `foo`. Consider the following to transform `g(x, x)` into `g(f(x), h(x))`.
 ```bash
> def foo as f(a) => g(a, a)
> f(x) 
Start matching on: f(x)
    ~> apply foo at 0
    g(x, x)
    ~> g(a, a) => g(f(a), h(a)) at 0    // inline rule
    g(f(x), h(x))
    ~> apply foo at 1                   // defined rule at depth 1
    g(g(x, x), h(x))
    ~>
```
To end the pattern matching procedure, enter `end` followed by an optional string literal denoting a path, `"path/to/file.txt"`. The former just ends the pattern matching context, discarding the history of derivations, whereas the latter writes the derivation history to a specified file before discarding the history of derivations. 

Furthermore, in the above example, the `foo` rule is again applied on the result of the in-line rule, `g(f(x), h(x))`, at depth 1 to obtain `g(g(x, x), h(x))`. This might clear up how the depth works, as the functors are depth-first searched for the provided depth value and only then mathed upon. 

### Binary arithmetic operators
To provide more readability for nested functor expressions, the standard math operators, `+`, `-`, `*`, and `/` are also parsed as infix functor operators with corresponding identifier names, `add()`, `sub()`, `mul()`, and `div()`, respectively. Furthermore, the parentheses `()` can be used to group expressions together, which have a higher precedence during parsing (just like typical arithmetic precedence). In other words, writing an expression `c * (a + b)` gets translated into the binary functor `mul(c, add(a, b))`. 

```bash
> def distributive_law as a * (b + c) => a * b + a * c
> f(x) * (g(y) + h(z))
Start matching on: f(x) * (g(y) + h(z))
                   As functor: mul(f(x), group(add(g(y), h(z))))
    ~> apply distributive_law at 0
    f(x) * g(y) + f(x) * h(z)
    As functor: add(mul(f(x), g(y)), mul(f(x), h(z)))
    ~> end
Result: f(x) * g(y) + f(x) * h(z)
        As functor: add(mul(f(x), g(y)), mul(f(x), h(z)))
>
```

## Todo's
- [ ] add more control to pattern matching, not only at some depth but also some index of argument to match on, e.g., `x => y at 0, 2` where `2` indicates the second index at depth `0`.
- [ ] add wildcard to match anything and everything at all depths, e.g., `x => y at *` or just permit the `at DEPTH` with `x => y` to match on all depths. 
- [ ] write as latex math output when file extension of path in end-statement has `.tex`. 

## Examples

### Power rule of calculus

Consider the power rule of calculus 
$$f(x) = x^n \implies f'(x) = nx^{n-1}$$
We can define the power rule as the following rule.
```bash
def diff_power_rule as pow(x, n) => n * pow(x, n-1)
```
With this rule we can symbolically show that the derivative of $y^2$ is equal to $2y$.
```bash
Welcome to the REPL environment of raxio.
Enter "quit" to stop the REPL environment.
Enter "help" to see an overview of raxio syntax.
Enter "undo" during mattern patching to undo the current expression.
> def diff_power_rule as pow(x, n) => n * pow(x, n-1)
> pow(y, 2)
Start matching on: pow(y, 2)
    ~> apply diff_power_rule at 0
    2 * pow(y, 2 - 1)
    As functor: mul(2, pow(y, sub(2, 1)))
    ~> 2-1 => 1 at 2
    2 * pow(y, 1)
    As functor: mul(2, pow(y, 1))
    ~> pow(x, 1) => x at 1
    2 * y
    As functor: mul(2, y)
    ~> end 
Result: 2 * y
        As functor: mul(2, y)
>
```

### Proof of power rule

In the above example, the power rule is defined as a formula to transform an expression of exponentiation into the derivative of that expression. We can also use `raxio` to show the proof of the power rule by using the limit definition of the derivative

$$
    \lim_{h \to 0} \frac{f(x + h) - f(x)}{h}
$$

If we assume $f(x) = x^2$, then we can work this out to equal $f'(x) = 2x$. Using `raxio`, we can achieve the same systematically.  

```bash
> lim(h, 0, (f(x + h) - f(x)) / h)
Start matching on: lim(h, 0, (f(x + h) - f(x)) / h)
                   As functor: lim(h, 0, div(group(sub(f(add(x, h)), f(x))), h))
    ~> f(a) => pow(a, 2) at 4
    lim(h, 0, (pow(x + h, 2) - pow(x, 2)) / h)
    As functor: lim(h, 0, div(group(sub(pow(add(x, h), 2), pow(x, 2))), h))
    ~> pow(a + b, 2) => pow(a, 2) + 2 * a * b + pow(b, 2) at 4
    lim(h, 0, (pow(x, 2) + 2 * x * h + pow(h, 2) - pow(x, 2)) / h)
    As functor: lim(h, 0, div(group(sub(add(add(pow(x, 2), mul(mul(2, x), h)), pow(h, 2)), pow(x, 2))), h))
    ~> a + b + c - a => b + c at 3
    lim(h, 0, (2 * x * h + pow(h, 2)) / h)
    As functor: lim(h, 0, div(group(add(mul(mul(2, x), h), pow(h, 2))), h))
    ~> (a + b) / c => a/c + b/c at 1
    lim(h, 0, 2 * x * h / h + pow(h, 2) / h)
    As functor: lim(h, 0, add(div(mul(mul(2, x), h), h), div(pow(h, 2), h)))        
    ~> a * b * c / c => a * b at 2   
    lim(h, 0, 2 * x + pow(h, 2) / h)
    As functor: lim(h, 0, add(mul(2, x), div(pow(h, 2), h)))
    ~> pow(a, 2) / a => a at 2  
    lim(h, 0, 2 * x + h)
    As functor: lim(h, 0, add(mul(2, x), h))
    ~> lim(t, 0, a + t) => a at 0
    2 * x
    As functor: mul(2, x)
    ~> end
Result: 2 * x
        As functor: mul(2, x)
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

Rule         := Expr "=>" Expr "at" Number;
Define       := "def" Identifier "as" Expr "=>" Expr ;
Expr         := FunctorExpr | 
                VariableExpr ;
End          := "end" Path ;

Path         := "\"" ("/")? String ("/" String)* "\"" ;

FunctorExpr  := Identifier "(" (Expr ",")* ")" ;
VariableExpr := Identifier ("at" Number)?;

Number       := ("0"-"9") ("0"-"9")* ;
Char         := ("a"-"z" | "A"-"Z" | "_" )
String       := Char*
Identifier   := Char (Char | Number)* ;
```

# Raxio

Manipulation of symbols in the form of either constants (32 bit floats), variables, or functors, useful for pattern matching and simple proofs. The application is written in Rust and used via the terminal. Consider the following grammar for the domain specific language of Raxio. 

```ebnf
Stmt         := Rule | 
                Define | 
                Expr ; 

Rule         := Expr "=>" Expr ;
Define       := "def" Identifier "as" Rule ;
Expr         := FunctorExpr | 
                VariableExpr | 
                ConstantExpr ;
                
FunctorExpr  := Identifier "(" (Expr ",")* ")" ;
VariableExpr := Identifier ;
ConstantExpr := (0-9)*("." (0-9)*)? ;

Identifier   := (a-z | A-Z)* ;
```

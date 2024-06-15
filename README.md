# Ferrite

Ferrite is a small, expressive language meant for creating applications.

Visit [examples/tour.fr](examples/tour.fr) for a quick rundown of the syntax, constructs and other details of the language.

## Usage

* `ferrite`: Print version information and help text.
  * Options:
    * `-h`: Print help text.
    * `-v`: Print version.
* `ferrite new <name>`: Create a new project.
* `ferrite build [program]`: Build the current project, or a specific project. 
* `ferrite run [program]`: Build and run the current project, or a specific project.
* `ferrite test`: Execute tests in the current project.
  * Options:
    * `-u, --unit`: Run only unit tests. 
    * `-b, --bench`: Run only benchmarks.
* `ferrite check`: Verify and lint the current project.
  * Options:
    * `-f, --format`: Format the current project.
    * `--lsp`: Start the built-in language server.

## Overview

### Goals

These may or may not already be in the language.

* Simple, preferably word-based syntax.
* Batteries-included core library.
* Immutability by default.
* Expressiveness:
  * Pattern matching/destructuring
  * Lambdas
* Acceptable performance.
* Fast (sub-second for smaller projects) build times.
* Small binaries.
* Easy foreign function interfaces (FFI) - enabling library code in faster languages, especially C.
* WebAssembly as a tier 1 compilation target.

### Obstacles

These are non-goals - things that will not be added to the language.

* Null - use option/either/result instead.
* Manual memory management.
* Ownership/lifetime rules and references.
* Lifetimes.
* Separate syntax for generics.
  * Function parameters and record fields are instead expressed as either a single concrete type or possibly one or more traits.  

## Syntax

**Comments** are series of characters which are ignored by the compiler, delimited by the pound symbol and ended by a newline.  
```shell
comment -> "#" .* \n
```

**Spaces** are any characters which fall under general category of 'whitespace'. These are unilaterally ignored by the compiler, except during formatting.
```shell
space -> " " | "\r" | "\n" | "\t" 
```

**Identifiers** define the names of entities. Valid identifiers are defined in terms of the [Unicode XID](https://www.unicode.org/reports/tr31/tr31-39.html#D1) start and continue sets. Identifiers are allowed to start with an underscore.  
```shell
id -> (xid_start | "_") xid_continue*
```

**Keywords** are a subset of identifiers which specify program behavior.  
```shell
keyword -> "var" | "do" | "end" | "match" | "if" | "then" | "else" | "loop" | "while" | "for" | "in" | "to" | "function" | "return" | "type" | "record" | "trait"
```

**Paths** are used for grouping elements.  
```shell
path_segment -> id | super | self | package
path -> path_segment ("." path_segment)*
```

**Operators** apply operations to their left and/or right operands. If multiple operator characters exist in a row, they are combined into one operator (such as in the case of "=="). Not all combinations of valid operator characters in a row are valid operators.  
```shell
op -> ("+" | "-" | "*" | "/" | "%" | "^" | "<" | ">" | "=" | "." | ":" | "!" | "?")*
```

**Numbers** are integer or floating point literals.  
```shell
digit -> [0-9]
number -> digit+ ("." digit+)?
```

**Text** is any amount of unicode characters surrounded by unescaped quotation marks.
These literals may be prefixed with a single-character text modifier which specifies how the contained text should be interpreted.  
```shell
text_modifier -> ("f" | "r" | "c" | "b")
text -> text_modifier? '"' .* '"'
```  

**Bools** are the entire set of literals allowed in boolean logic.  
```shell
bool -> "true" | "false"
```

**Blocks** are a sequence of expressions which are evaluated in order. Blocks evaluate to the final expression in their body, otherwise the unit type. Blocks shadow their containing scope, meaning that a variable defined in a block will not leak to the outer scope.  
```shell
block -> "do" expression* "end"
```

**Types** are a separate namespace to identifiers which are used to verify program correctness before runtime. The language contains a few built-in types which form the basis for all other types.  
```shell
primitive_type -> "bool" | "number" | "text" | "unit"
type -> primitive_type | id
```

**Expressions** evaluate to a value and always have a type.  
```shell
expression -> ?
```

**Patterns** are used to destructure and bind values from structures.  
```shell
pattern -> basic_pattern ("or" basic_pattern)*
basic_pattern -> 
    literal_pattern 
  | identifier_pattern 
  | tuple_pattern
  | list_pattern
  | table_pattern
  | datatype_pattern
  | record_pattern 

literal_pattern -> bool | number | text | "unit"
identifier_pattern -> id
tuple_pattern -> "(" basic_pattern ("," basic_pattern)* ")"
list_pattern -> "[" basic_pattern ("," basic_pattern)* "]"
tuple_pattern -> "(" basic_pattern ("," basic_pattern)* ")"
table_pattern -> "{" basic_pattern ("," basic_pattern)* "}"
datatype_pattern -> "{" basic_pattern ("," basic_pattern)* "}"
record_pattern -> path "{" basic_pattern ("," basic_pattern)* "}"
```

**Assignment** associate an identifier with an expression, which may optionally have a type definition alongside it. Variables are defined by including the "var" keyword before the identifier.  
```shell
assignment -> pattern "=" expression
variable -> "var"? pattern "=" expression
```

**Lists** are growable homogenous ordered arrays of expressions. Lists may only contain elements of the same type.   
```shell
list -> "[" expression ("," expression)* "]"
```

**Sequences** are finite heterogenous ordered lists. Sequences may contain elements of differing types, but may not dynamically grow in size.  
```shell
sequence -> "(" expression ("," expression)* ")"
```

**Tables** are growable, heterogenous associative arrays over expressions.  
```shell
table_element -> expression = expression
table -> "[" table_element ("," table_element)* "]"
```

**Datatypes** are tagged unions (sum types).  
```shell
datatype -> "type" id type (, type)* "end"
```

**Records** are a heterogenous collection of fields.  
```shell
field -> id ":" type
record -> "record" id field ("," field)* "end"
```

**Traits** define the functionality of a particular type.  
```shell
trait -> "trait"
```

## Associativity & Precedence

Operators/Expressions higher in the table have stronger binding.  
Operators with the same precedence share a row.

| Operator/Expression        | Associativity       | Examples         |
| -------------------------- | ------------------- | ---------------- |
| Grouping                   |                     | `(a + b) * c`    |
| Indexing                   | Left to right       | `a.b`            |
| Unary negation/logical not | Right to left       | `-a`, `!a`       |
| Multiplication/division    | Left to right       | `a * b`, `a / b` |
| Addition/subtraction       | Left to right       | `a + b`, `a - b  |
| Equality                   | Require parentheses | `a == b`         |
| Logical and                | Left to right       | `a and b`        |
| Logical exclusive or (xor) | Left to right       | `a xor b`        |
| Logical or                 | Left to right       | `a or b`         |
| Ranges                     | Require parentheses | `a to b`         |
| Assignment                 | Right to left       | `a = b`          |

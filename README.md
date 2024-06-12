# Ferrite

## Syntax

**Comments** are series of characters which are ignored by the compiler, delimited by the pound symbol and ended by a newline.  
`comment -> '#' .* \n`

**Identifiers** define the names of entities. Valid identifiers are defined in terms of the Unicode XID start and continue sets. Identifiers are allowed to start with an underscore.  
`id -> (xid_start | '_') xid_continue*`

**Keywords** are a subset of identifiers which specify program behavior.  
`keyword -> "var" | "do" | "end" | "if" | "then" | "else" | "loop" | "while" | "for" | "in"`

**Operators** apply operations to their left and/or right operands. If multiple operator characters exist in a row, they are combined into one operator (such as in the case of '=='). Not all combinations of valid operator characters in a row are valid operators.  
`op -> ('+' | '-' | '*' | '/' | '%' | '^' | '<' | '>' | '=' | '.' | ':' | '!' | '?')*`

**Numbers** are integer or floating point literals.  
`digit -> [0-9]`  
`number -> digit+ ('.' digit+)?`

**Text** is any amount of unicode characters surrounded by unescaped quotation marks.
These literals may be prefixed with a single-character text modifier which specifies how the contained text should be interpreted.  
`text_modifier -> ('f' | 'r' | 'c' | 'b')`  
`text -> text_modifier? '"' .* '"'`  

**Bools** are the entire set of literals allowed in boolean logic.  
`bool -> "true" | "false"`

**Blocks** are a sequence of expressions which are evaluated in order. Blocks evaluate to the final expression in their body, otherwise the unit type. Blocks shadow their containing scope, meaning that a variable defined in a block will not leak to the outer scope.  
`block -> "do" expression* "end"`

**Types** are a separate namespace to identifiers which are used to verify program correctness before runtime. The language contains a few built-in types which form the basis for all other types.  
`primitive -> "bool" | "number" | "text" | "unit"`
`type -> primitive | id`

**Expressions** evaluate to a value and always have a type.  
`expression -> ?`

**Declarations** associate an identifier with an expression, which may optionally have a type definition alongside it. Variables are defined by including the "var" keyword before the identifier.  
`declaration -> "var"? id '=' expression`

**Lists** are growable homogenous ordered arrays of expressions. Lists may only contain elements of the same type.   
`list -> '[' expression (',' expression)* ']'`

**Sequences** are finite heterogenous ordered lists. Sequences may contain elements of differing types, but may not dynamically grow in size.  
`sequence -> '(' expression (',' expression)* ')'`

**Tables** are growable, heterogenous associative arrays over expressions.  
`table_element -> expression = expression`  
`table -> '[' table_element (',' table_element)* ']'`

**Type definitions** are tagged unions (sum types).  
`type_definition -> "type" id type (, type)* "end"`

**Records** are a heterogenous collection of fields.  
`field -> id ':' type`
`record -> "record" id field (',' field)* "end"`

**Traits** define the functionality of a particular type.  
`trait -> trait `

## Associativity & Precedence

Operators/Expressions higher in the table have stronger binding.  
Operators with the same precedence share a row.

| Operator/Expression        | Associativity       | Examples         |
| -------------------------- | ------------------- | ---------------- |
| Grouping                   |                     | `(a + b) * c`    |
| Indexing                   | Left to right       | `a.b`            |
| Unary negation/logical not |                     | `-a`, `!a`       |
| Multiplication/division    | Left to right       | `a * b`, `a / b` |
| Addition/subtraction       | Left to right       | `a + b`, `a - b  |
| Equality                   | Require parentheses | `a == b`         |
| Logical and                | Left to right       | `a and b`        |
| Logical exclusive or (xor) | Left to right       | `a xor b`        |
| Logical or                 | Left to right       | `a or b`         |
| Ranges                     | Require parentheses | `a to b`         |
| Assignment                 | Right to left       | `a = b`          |

## Command Line Tools

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


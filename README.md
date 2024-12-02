# `inet-types`

Types for interaction nets. Documentation not written yet. Read [EXPLANATION.md], though.

## Syntax

```ebnf
var_name = /[a-z][a-zA-Z0-9]+/
ctr_name = /[!A-Z0-9][a-zA-Z0-9]+/
macro = ctr_name "[" /.*/ "]"
tree = macro tree? | ctr_name | ctr_name "(" tree* ")" | var_name
item = macro
     | tree "~" tree # Rule
     | tree "=" tree # Redex
     | "{" book "}"
book = item*
```


### Basic syntax

Trees are either variables, which start with lowercase letters, or cells, which start with uppercase letters and then contain a space-separated list of trees in the auxiliary ports, enclosed by parnetheses.

### Scopes

`{}` introduces a scope. A scope allows defining local agents and local variables.


### Macros

`Name[ ... ]`  is a macro usage, which is actually more like C's preprocessor directives. An example of one is `#[ ... ]`, which creates a comment.

## Special agents

### Duality agent

`~` is the duality agent. `a = ~(b)` makes `b` the cocell of `a`. Each symbol `T` has an cosymbol `~T`. `~T` is the type of all cells that can interact with cells of type `T`. In other words, it is the supertype of the types of cells that can interact with cells of type `T`. The `~` operator is involutive, so `~~T = T` . 

The `~` agent commutes through auxiliary ports in addition to changing a cell's symbol.

### Annotation agent

`:` is the annotation agent type. The first port contains the value, and the second port contains the type.

### Annotator agent

`::` is the annotator agent type. It can annotate whole trees with their types. `::(a) ~ b` makes `a` the annotated version of the tree `b`.



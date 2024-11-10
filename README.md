# `inet-types`

Types for interaction nets. Documentation not written yet

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

Trees are either variables, which start with lowercase letters, or agents, which start with uppercase 

### Scopes

`{}` introduces a scope. A scope allows defining local agents and local variables.

## Special agents

### Inverse agent

`~` is the inverse agent. `a = ~(b)` makes `a` the inverse agent of `b`. Each agent type `T` has an inverse agent type `~T`. `~T` is the type of all agents that can interact with agents of type `T`. In other words, it is the supertype of the types of agents that can interact with agents of type `T`. The `~` operator is involutive, so `~~T = T` . 

### Annotation agent

`:` is the annotation agent type. The first port contains the value, and the second port contains the type.

### Annotator agent

`::` is the annotator agent type. It can annotate whole trees with their types. `::(a) ~ b` makes `a` the annotated version of the tree `b`.



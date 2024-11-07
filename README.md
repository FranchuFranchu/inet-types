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

Trees are either variables, which start with lowercase letters, or agents

### Scopes

`{}` introduces a scope. A scope allows defining new interactions and local variables.

## Typing

Typing works using two agents: `:`, the annotation agent, and `::` which is the annotator agent.
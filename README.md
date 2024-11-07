## Syntax

```ebnf
var_name = /[a-z][a-zA-Z0-9]+/
ctr_name = /[!A-Z0-9][a-zA-Z0-9]+/
macro = ctr_name "[" /.*/ "]"
tree = macro tree? | ctr_name | ctr_name "(" tree* ")" | var_name
redex = tree "=" tree | macro
item = macro
     | tree "=" tree ("{" redex* "}")?
book = item*
```
# Interaction Net Types

An _interaction system_ is defined by:
- A set of _symbols_, each with an _arity_
- A set of _interaction rules_ between each pair of _symbols_. 

A pair of symbols can have no rule assigned to them. In this case, we say the interaction between those two symbols is _undefined_. We'll say that an interaction between two cells with those symbols is _stuck_, because it can never reduce.

A cell is a node in an interaction net. A cell has a _symbol_, a single principal port, and an amount of auxiliary ports which correspond to its symbol's arity. Unlike graph theory nodes, a cell has structure; each port is different.

## Syntax

This document uses textual syntax to talk about interaction nets. 

A _tree_ is either a cell or a variable, which represents an aux-aux connection. Variable names start with a lowercase character.

A cell looks like this:
```
SymbolName(a0 a1 ... an)
```
Where `a{n}` are the trees plugged in to its auxiliary ports. Symbol names start with anything that is not a lowercase character or is in the string `" ()[]{}="`

An interaction net can be represented by a set of pairs of trees which form active pairs. Active pairs use the syntax `<tree> = <tree>`

For example, the following is a net with two active pairs:
```
Mult(a1 Add(a1 out)) = S(0)
Dup(a0 a1) = S(S(0))
```
Notation abuse is allowed; we can pretend connections to auxiliary ports are active pairs too.
```
Mult(a1 other) = S(0)
Dup(a0 a1) = S(S(0))
other = Add(a1 out)
```
The original net can be recovered with a simple substitution.

Variables that only appear once are free ports. Other variables must appear exactly twice.

A interaction rule is described by a syntax similar to the one used by Lafont in his 1991 paper, where the `~` character takes the role of `><`:
```
A(a0 a1 a2 ... an) ~ B(b0 b1 b2 ... bm)
```
Here, a{n} represents what will get plugged in to the nth port after the reduction. In other words, the rule syntax encodes the following reduction rule (of course, using fresh variable names):
```
A(a0' a1' a2' ... an') = B(b0' b1' b2' ... bm')
----------------------------------------
a0 = a0'
a1 = a1'
....
an = an'
b0 = b0'
b1 = b1'
....
bm = bm'
```

We'll sometimes use the syntax `A ~ B` to talk about the interaction rule between symbols `A` and `B`.

The `A(...) ~ B(...)` syntax is enough to faithfully represent all possible reduction rules. However, it's very hard to read and use when the trees inside the auxiliary ports are very large. This is why we'll allow rule definitions to be followed by a net delimited by curly braces. The net can access variables that are used in the rule definition.

So, for example, 
```
Mult(Dup(b c) a) ~ S(Times(b Add(c a)))
```
is equivalent to:
```
Mult(argument result) ~ S(pred) {
	#[ 
		This is an implementation of
		x * S(y) = x * y + x

		x is argument, y is pred.
	] 
	argument = Dup(a0 a1)
	pred = Times(a1 Add(a0 result))
}
```
This is nice because rules can be self-documenting.

To be clear, `~` defines interaction rules, while `=` is used to represent active pairs inside a net. Usually, `~` will be in top-level definitions, while `=` will appear inside rules to encode which active pairs will appear as the result of a reduction.

## Type-checking

To talk about a program being well-typed, we first need to define what the equivalent of a runtime "type error" is. The equivalent we'll use is "undefined interactions". Undefined interactions are like typing errors. For example, if we define `True ~ Bool.not`, `False ~ Bool.not`, then `Zero ~ Bool.not` is a type error, because it is an undefined interaction.

A type checker ensures that type errors never happen at runtime by categorizing terms into _types_. Even if we don't know the terms, if there are no type errors in the type-level, then there will be no type errors in the term-level. 

This is very easy to translate to interaction nets. We can categorize symbols into types, which are symbols too. For example, `Bool.true` is an instance of `Bool`, and so is `Bool.false`. We'll also have to define a type for everything that "consumes" a `Bool`, like `Bool.and`, `Bool.not`, `Bool.match`. We'll call this type `~Bool`. `~Bool` is the type of everything that can interact with `Bool`.

We'll introduce a condition, called the *completeness condition*. It says that if two types `TA` and `TB` interact, then for all `Va: TA`, `Vb: TB`, `Va ~ Vb`. 

For example, if we define `Bool ~ ~Bool`, then everything that is a `Bool` must be able to interact with everything that is a `~Bool`.

Thanks to the completeness condition, if we substitute each cell by its type, and the resulting net reduces just fine without any undefined interactions, then we know that if we reduce the value-level cells, no undefined interactions will happen.

A type error at the type level happens when the interaction between two types is undefined. So, for example, the interaction `Nat ~ ~Bool` will not be defined, but the interaction `Bool ~ ~Bool` will.

### Auxiliary ports

Under this scheme, auxiliary ports will have to be replaced by something that can interact with whatever is behind the auxiliary port. So for example, the successor constructor `a = Succ(b)` would get substituted by `a = Nat; b = ~Nat`.

## Cotypes

Let's look into more detail into `~T`. `~T` is a special primitive; it's unlike other symbols. It's the type of all types of cells which interact with things of type `T`. In other words, it's the supertype of all types which interact with `T`. It's the _weakest_ and _most general_ type which can interact with `T`

It turns out that if we look at it closely, we realize that `~` is an involutive operator. `~~T` is the supertype of everything that can interact with `~T`. The only thing that is guaranteed to interact with `~T` is `T`, so `~~T = T`

`~` is like linear logic's `⊥` operator, which takes the dual of a proposition.

> Every type A has a dual A^⊥, and returning a re-
sult of type A is equivalent to consuming an argument of type A^⊥.
Further, dualization is involutive: (A^⊥)^⊥ = A. [1]

To encode the definition of the `~` operator into the system, we'll introduce the **subtyping property**. If `A ~ B` and `~B ~ C`, then `A ~ C`.

In English, it means "If A interacts with B, and everything that interacts with B also interacts with C, then A must interact with B"

This is enough to encode `~`'s properties.

### Subtyping

The duality operator is closely related to subtyping. If anything that can consume a T can also consume an U, then U is effectively a subtype of T. In this system, this looks like `~T ~ U`, which can be used to define `U <= T`.

If `T <= U`, then `~U <= ~T`

## Annotations

The notion of "substituing" a cell with its type to type-checker can be useful in simply typed cases. However, for more complex type-systems, we sometimes need to "recover" the value to type-check something. This is the case, for example, in dependent type checking.

This is why, from now on, instead of substituting cells by their type, we'll _annotate_ then with their type by using a new symbol `:`. Roughly, a cell `Value` gets annotated as `:(Value Type)`.

The `:` symbol annihilates with itself. So, as a consequence, annotation will do what substitution did before; if two values interact, then their types will interact in the annotated net too.

We can implement the annotation transformation using interaction nets, by introducing a 1-ary `::` agent which is the annotator agent. `::` replaces a cell with its annotated form, and recurses through auxiliary ports, so it can be used to annotate whole trees. Here are some examples:

```
Bool.true ~ ::(:(Bool.true Bool))
Bool.false ~ ::(:(Bool.false Bool))

Bool.not(:(out Bool)) ~ ::(:(Bool.not(out) ~Bool))
Bool.and(:(arg ~Bool) :(out Bool)) ~ ::(:(Bool.or(arg out) ~Bool))

Nat.zero ~ ::(:(Nat.zero Nat))
Nat.succ(:(arg ~Nat)) ~ ::(:(Nat.succ(arg) Nat))
```

`::` annihilates with itself.

### Checking nets

This is enough to check all nets. For example, we can check that if take the logical AND of Bool.true and a boolean value, we get a boolean value as a result:

```
# Original net:
Bool.true = Bool.and(input output)
# Annotated.
a = b
::(a) = Bool.true
::(b) = Bool.and(:(input Bool) :(output ~Bool))
-----------------------------------------------
::(a) = Bool.true
::(a) = Bool.and(:(input Bool) :(output ~Bool))
-----------------------------------------------
::(:(Bool.true Bool)) = Bool.and(:(input Bool) :(output ~Bool))
-----------------------------------------------
:(Bool.true Bool) = :(Bool.and(i o) ~Bool)
:(input Bool) ~ :(i ~Bool)
:(output ~Bool) ~ :(o Bool)
----------------------------------------------- *
Bool.true = Bool.and(input output)
Bool = ~Bool
Bool = ~Bool
~Bool = Bool
----------------------------------------------- *
input = output
# No undefined interactions, so it type checks.
```

The reason this checks types is because of the completness condition. If two values that can't interact were to interact, then their types would have to interact too, and the completness condition means that values with undefined interactions have types with undefined interactions.

## Checking rules

We've mentioned already that nets can be checked by annotating all cells. However, right now, we can write ridiculous rules that don't care about how to type the symbols that are involved in the interaction. For example, right now, this is allowed:

```
::(:(Foo(x) Bool)) ~ Foo(::(:(x !Bool)))
::(:(Bar(x) !Empty)) ~ Bar(::(:(x Empty)))

Foo(x) ~ Bar(x)
```

The interaction can be used to convert a boolean value to an instance of the empty type.

There's a special net we can construct to check types. To check a rule between symbols `A` and `B`, it looks like this:

```
::(v) = A(::(a0) ::(a1) ::(a2) ... ::(an))
::(v) = B(::(b0) ::(b1) ::(b2) ... ::(bn))
A(::(~(a0)) ::(~(a1)) ::(~(a2)) ... ::(~(an))) = B(::(~(b0)) ::(~(b1)) ::(~(b2)) ... ::(~(bm)))
```

This uses the `~` agent, which turns cells it interacts with into the respective cocells, and commutes through auxiliary ports.

For example, let's type-check `Foo ~ Bar`:

```
::(v) = Foo(::(a0))
::(v) = Bar(::(b0))
Foo(::(~(a0))) = Bar(::(~(b0)))
--------------------------------- :: ~ Foo, :: ~ Bar, :: ~ ::, :: ~ ::
v = :(Foo(x) Bool)
a0 = :(x !Bool)
v = :(Bar(y) !Empty)
b0 = :(y Empty)
Foo(::(~(a0))) = Bar(::(~(b0)))
--------------------------------- subst
:(Foo(x) Bool) = :(Bar(y) !Empty)
Foo(::(~(:(x !Bool)))) = Bar(::(~(:(y Empty))))
--------------------------------- : ~ :
Bool = !Empty
Foo(x) ~ Bar(y)
Foo(::(~(:(x !Bool)))) = Bar(::(~(:(y Empty))))
```

`Bool ~ !Empty` is undefined, so we've already lost.

The carefully placed `~` agents are necessary for things that should check to check. Their placement is related to how subtyping should apply when checking rules. I'm still not quite sure why it works, or how to justify it. More research is needed here. TODO: Add example of checking a rule that is well-typed.

If no undefined interactions appear, then the rule is well-typed.

## Examples

Under this system, there is more than one way to implement ideas from other type systems and programming languages. It's not always clear what the best way to do these things is. I'll give a few examples here.

### Traits

In traditional type theories, duplication and erasure are usually implicit. However, when using interaction nets, duplication has to be defined explicitly. Fortunately, under this system, with everything we've mentioned so far, we can define constructs that look a bit similar to interfaces or traits in other languages.

It's not necessary for a constructor type T to interact only with its destructor type ~T . In fact, we can make it interact with many different destructors. This feature is like Java's interfaces or Rust's traits

If we define `Era: ~Erasure`, and `~Erasure ~ Bool`, then all booleans must be able to interact with `Era`. If we later define `~Erasure ~ Nat`, then `Era` will be able to erase both natural numbers and booleans.

`Erasure` is a supertype of both `Bool` and `Nat`, and, as a consequence, `~Erasure` is a subtype of both `~Bool` and `~Nat` (the only inhabitant being `Era`)

```
#[ 
	Define the ~Erasure "trait" or "interface"
	Era consumes erasable things, so it's of type ~Erasure

]
::(:(Era ~Erasure)) ~ Era

#[
	Bool implements Erasure
]
~Erasure ~ Bool
Era ~ Bool.true
Era ~ Bool.false


#[ Using Era ]

::(:(Bool.and(x y) ~Bool)) ~ Bool.and(::(:(x ~Bool)) ::(:(y Bool)))
Bool.and(x x) ~ Bool.true
Bool.and(Era Bool.false) ~ Bool.false
```

### Dependent typing

Annotation is powerful enough to encode dependent types. For example, here's a DepTest symbol, which returns either a boolean or a natural number depending on what's passed to it.

```

#[ This determines what the return type of DepTest is ]
::(:(DepTest.aux(x) ~Bool)) ~ DepTest.aux(::(:(x Type)))
DepTest.aux(Bool) ~ Bool.true
DepTest.aux(Nat) ~ Bool.false

::(:(v ~Bool)) ~ DepTest(::(:(r rt))) {
	Dup(v0 v1) = v
	v0 = DepTest(r)
	v1 = DepTest.aux(rt)
}

DepTest(Bool.false) ~ Bool.true
DepTest(Nat.zero) ~ Bool.false
```

### Top and bottom

The top type `Top` is defined by having no destructors (the type `~Top` is uninhabited). This means that you can always write `A: Top`, and you will never have to implement anything.

The bottom type `Bot` is defined by having no constructors (the type `Bot` is uninhabited). This means that you can always write `A: ~Bot`, and you will never have to implement anything. This models the principle of explosion. 

In fact, `~Bot == Top; ~Top == Bot`

### TODO

There's a lot of ideas that I'm not sure how to implement yet. Mainly, higher-order stuff like functions, and how to duplicate functions.

Also, how can I add T6's lifetimes to this? 

## Conclusion

This system is very cool, and it's the closest I've gotten so far to an interaction type theory capable of working as the foundation of a proof checker.

Please share any comments, questions, or ideas with me.

[1] https://citeseerx.ist.psu.edu/document?repid=rep1&type=pdf&doi=19b1cdead55c69b77425c9688da6ba2dae5aa9be
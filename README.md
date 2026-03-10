# Mini-Imp

Mini-Imp is a simple imperative language, used for exercise in the
compilation technique course.

## Grammar

```
<prog> ::= def main with input <var> output <var> as <cmd>

<cmd> ::= (<cmd>) | skip | <var> := <exp> | <cmd> ; <cmd>
  | if <b> then <cmd> else <cmd> | while <b> do <cmd> | print <exp>
  | skip

<exp> ::=  <exp> + <term> | <exp> - <term> | <term>

<term> ::= <term> * <factor> | <factor>

<factor> ::= <var> | <int> | (<exp>)

<b> ::= true | false | <b> and <b> | <b> or <b> | not <b> |
  | <exp> < <exp> | <exp> > <exp>

<var> Is the set of letters and numbers starting with a letter.
<int> are integers numbers

```

## Usage

To run simple mini-imp programs:

```bash

mini-imp <program> <input>

```

Some simple programs are in the `./examples/` directory

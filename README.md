# Mini-Imp

Mini-Imp is a simple imperative language, used for exercise in the
compilation technique course.

## Grammar

```
<prog> ::= def main with input <var> output <var> as <cmd>

<cmd> ::= <atom_cmd> ; <cmd> | <atom_cmd>

<atom_cmd> ::= (<cmd>) | skip | <var> := <exp>
  | if <b> then <atom_cmd> else <atom_cmd> | while <b> do <atom_cmd> | print <exp>

<exp> ::=  <exp> + <term> | <exp> - <term> | <term>

<term> ::= <term> * <factor> | <factor>

<factor> ::= <var> | <int> | (<exp>)

<bexp> ::= <bexp> and <atom> | <bexp> or <atom> | not <bexp>
  | <exp> < <exp> | <exp> > <exp> | <atom>

<atom> ::= true | false | (<bexp>)

<var> Is the set of letters and numbers starting with a letter.
<int> are integers numbers

```

## Usage

To run simple mini-imp programs:

```bash

mini-imp <program> <input>

```

This also create a dot file describing the Control Flow Graph of the code.

Some simple programs are in the `./examples/` directory.

# Mini-Imp

Mini-Imp is a simple imperative language, used for exercise in the
compilation technique course.

## Grammar

```
<prog> ::= def main with input <var> output <var> as <cmd>

<cmd> ::= (<cmd>) | <var> := <e> | <cmd> ; <cmd>
  | if <b> then <cmd> else <cmd> | while <b> do <cmd> | print <e>

<e> ::= <var> | <int> | <e> + <e> | <e> - <e> | <e> * <e>

<b> ::= true | false | <e> and <e> | <e> or <e> | not <e> |
  | <e> < <e> | <e> > <e>

<var> Is the set of letters and numbers starting with a letter.
<int> are integers numbers

```

## Usage

To run simple mini-imp programs:

```bash

mini-imp <program> <input>

```

Some simple programs are in the `./examples/` directory

# VEAC Compile-Time Grammar

This file owns the grammar for the static programming layer. Names and literals use the shared
[authoring lexical rules](grammar.md#lexical-rules).

```ebnf
entry       = { import | declaration } , project ;
module-file = "module" , "{" , { [ "export" ] , declaration } , "}" ;
import      = "import" , relative-string , "as" , identifier , ";" ;
declaration = typed-const | typed-preset | sequence-component | sequence-instance ;
typed-const = "const" , value-type , identifier , "=" , expression , ";" ;
typed-preset = "preset" , preset-kind , identifier , block ;
sequence-instance = "instance sequence" , identifier , "from" , qualified-identifier
                  , instance-block ;
sequence-component = "component sequence" , identifier , "{"
                   , { parameter | source-slot | local-sequence-instance }
                   , "body" , block , "}" ;
local-sequence-instance = "instance sequence" , "@" , identifier
                        , "from" , qualified-identifier , instance-block ;
instance-block = "{" , { bind | fill } , "}" ;
parameter = "param" , value-type , identifier , [ "default" , expression ] , ";" ;
source-slot = "slot" , slot-kind , identifier , ";" ;
slot-kind = "video" | "audio" | "visual" | "text" | "caption" | "sequence" ;
bind = "bind" , identifier , expression , ";" ;
fill = "fill" , identifier , block ;
value-type = "scalar" | "time" | "length" | "percent" | "angle"
           | "text" | "color" | "bool" | "identifier" ;
expression = literal | qualified-identifier | unary | binary | call
           | "(" , expression , ")" ;
```

The `module {}` marker has no self-name. Its canonical identity is its loader-provided source ID,
and each importer chooses a lexical alias. The nine `value-type` spellings above are exhaustive.

Constants allow forward references and exact pure expressions. Both `//` and `/* ... */` comments
are legal between expression tokens. Preset kinds, parameter defaults, source slots,
`${expression}` injection, and local `@id` hygiene are statically checked.

Local sequence instances are compile-time component members. Expansion emits every descendant as a
top-level core sequence, rewrites the parent's `@id` reference to the same hygienic ID, and rejects
composition cycles, excessive depth, excessive nodes, or expanded output before core parsing.

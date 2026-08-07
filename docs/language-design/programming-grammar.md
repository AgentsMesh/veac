# VEAC 可执行编程语法

本文件定义 executable Surface grammar。name 与 literal 复用
[lexical rules](../language-reference/vocabulary.md#词法与标识符)。

```ebnf
entry       = { import | input-declaration | declaration | temporal-declaration } ;
module-file = "module" , "{" , { import | [ "export" ] , declaration } , "}" ;
import      = "import" , relative-string , "as" , identifier , ";" ;
input-declaration = "input" , input-role , identifier , ":" , value-type , ";" ;
input-role = "parameter" | "asset_metadata" | "analysis" ;
declaration = typed-const | function | struct-declaration | enum-declaration
            | implementation ;

typed-const = "const" , value-type , identifier , "=" , expression , ";" ;
function    = "fn" , identifier , "(" , [ function-parameter
            , { "," , function-parameter } ] , ")"
            , "->" , value-type , function-body ;
function-parameter = identifier , ":" , value-type ;
function-body = "{" , { block-statement } , expression , "}" ;
block-statement = let-statement | var-statement | set-statement ;
let-statement = "let" , identifier , [ ":" , value-type ]
              , "=" , expression , ";" ;
var-statement = "var" , identifier , [ ":" , value-type ]
              , "=" , expression , ";" ;
set-statement = "set" , identifier , "=" , expression , ";" ;
struct-declaration = "struct" , identifier , "{" , { field-declaration } , "}" ;
enum-declaration = "enum" , identifier , "{" , enum-variant
                 , { "," , enum-variant } , [ "," ] , "}" ;
field-declaration = identifier , ":" , value-type , "," ;
enum-variant = identifier , [ "{" , { field-declaration } , "}" ] ;
implementation = "impl" , qualified-identifier , local-id , "{" , { method } , "}" ;
method = [ "export" ] , "fn" , identifier , "(" , "self"
       , { "," , function-parameter } , ")" , "->" , value-type , function-body ;

temporal-declaration = "animate" , temporal-property , "on" , temporal-target
                     , [ "using" , "resource" , resource-path ] , function-body ;
temporal-target = "clip" , item-path
                | "text" , item-path
                | "clip-mask" , clip-mask-path
                | "clip-effect" , clip-effect-path
                | "apply" , apply-path
                | "apply-mask" , apply-mask-path
                | "apply-effect" , apply-effect-path ;
item-path = "(" , item-segments , ")" ;
clip-mask-path = "(" , item-segments , "," , mask-index , ")" ;
clip-effect-path = "(" , item-segments , "," , local-id , "," , local-id , ")" ;
apply-path = "(" , apply-segments , ")" ;
apply-mask-path = "(" , apply-segments , "," , mask-index , ")" ;
apply-effect-path = "(" , apply-segments , "," , local-id , "," , local-id
                  , "," , local-id , ")" ;
item-segments = local-id , "," , local-id , "," , local-id , "," , local-id ;
apply-segments = local-id , "," , local-id , "," , local-id ;
resource-path = "(" , local-id , "," , local-id , ")" ;
local-id = "@" , identifier ;
temporal-property = "visual-position" | "visual-scale" | "visual-rotation"
                  | "visual-crop" | "visual-opacity" | "audio-gain" | "audio-pan"
                  | "mask-position" | "mask-scale" | "mask-rotation"
                  | "mask-feather" | "mask-expansion"
                  | "text-position" | "text-scale" | "text-rotation"
                  | "text-reveal" | "text-highlight-progress" | "text-opacity"
                  | "effect-parameter" | "apply-opacity" ;
mask-index = unsigned-decimal-integer ;

value-type = primitive-type | qualified-identifier
           | list-type | range-type | map-type | tuple-type | function-type ;
primitive-type = "int" | "scalar" | "time" | "length" | "percent" | "angle"
               | "text" | "color" | "bool" | "identifier" ;
list-type = "list" , "<" , value-type , ">" ;
range-type = "range" , "<" , "int" , ">" ;
map-type = "map" , "<" , ( "text" | "identifier" ) , "," , value-type , ">" ;
tuple-type = "(" , value-type , "," , value-type , { "," , value-type } , ")" ;
function-type = "fn" , "(" , [ value-type , { "," , value-type } ] , ")"
              , "->" , value-type , function-effect ;
function-effect = "effect" , ( "pure" | "local" | "emit" | "any" ) ;
expression = unary | binary | range-expression | postfix-expression ;
postfix-expression = primary-expression , { call-suffix | member-suffix } ;
call-suffix = "(" , [ expression , { "," , expression } ] , ")" ;
member-suffix = "." , identifier , [ call-suffix ] ;
primary-expression = literal | qualified-identifier | closure | for-expression
                   | if-expression | function-body
                   | match-expression | nominal-constructor
                   | list-literal | map-literal | tuple-literal | "(" , expression , ")" ;
nominal-constructor = qualified-identifier , "{" , [ named-field
                    , { "," , named-field } , [ "," ] ] , "}" ;
named-field = identifier , ":" , expression ;
match-expression = "match" , expression , "{" , match-arm
                 , { "," , match-arm } , [ "," ] , "}" ;
match-arm = ( variant-pattern | "_" ) , "=>" , expression ;
variant-pattern = qualified-identifier , [ "{" , [ pattern-field
                , { "," , pattern-field } , [ "," ] ] , "}" ] ;
pattern-field = identifier , [ ":" , identifier ] ;
closure = "fn" , "(" , [ function-parameter , { "," , function-parameter } ] , ")"
        , "->" , value-type , function-effect , function-body ;
range-expression = additive-expression , ".." , additive-expression
                 , [ "by" , additive-expression ] ;
list-literal = "[" , [ expression , { "," , expression } ] , "]" ;
map-literal = "#{" , [ map-entry , { "," , map-entry } ] , "}" ;
map-entry = expression , ":" , expression ;
tuple-literal = "(" , expression , "," , expression , { "," , expression } , ")" ;
if-expression = "if" , expression , function-body , "else" , function-body ;
for-expression = "for" , identifier , "in" , expression , function-body ;
```

`module {}` 没有 self-name；canonical identity 是 loader 提供的 source ID，importer 选择 lexical
alias。十种 `primitive-type` spelling 是完整闭集；`list`、`map`、tuple 和 `fn` 构造递归
semantic type。`fn` 分别是 declaration introducer 和结构类型 constructor；signature
punctuation `:` 与 `->` 不是 keyword 或 vocabulary entry。

`range<int>` 是唯一范围类型。`start .. end` 与 `start .. end by step` 都是半开范围，三个操作数
必须恰好为 `int`；省略步长在执行时解释为 `1`。`..` 非结合，优先级低于 additive、高于
ordering。零步长及计数或算术溢出在执行时失败，方向背离终点产生空范围。

函数只有 typed block body，不接受旧的 `= expression;` 形式。parameter 和 `let` binding 均为
immutable；`var` 声明 lexical mutable local，`set` 只能按 exact type 更新已声明的 `var`。三种
statement 按 source order 执行，最后一个无分号 expression 是 block 的类型和值。`let`/`var` 的
可选 annotation 提供 expected type，不是 cast。declared return type 必须与 block tail 的 inferred
type 相同。body 可读取自己的 parameter 与 lexical local，并调用 builtin、同 module function 或
qualified imported function。mutable local 不能逃逸或被 closure capture。
module 通过 `export` 显式发布 const、fn、struct 与 enum；impl block 本身不导出，method 在 block 内
逐个使用 `export fn` 发布。exported declaration 可以依赖合法的 private sibling declaration，resolver
会保留其闭合依赖但不会把 private name 暴露给 importer。

root source 可声明 `input parameter`、`input asset_metadata` 与 `input analysis`。input 是 host 在
build 边界显式提供的 typed source-of-truth，不是 ambient filesystem、environment、clock 或 network
读取；module file 不能自行声明 input。

closure 使用完整显式签名 `fn(name: T, ...) -> R effect E { ... }`，并通过 postfix grammar 调用，因此
local、parameter、返回值、closure literal 与连续调用共用一种机制。closure 可捕获外层 immutable
local、parameter 或 capture，按第一次解析出现顺序逐层建立环境；ambient external、function value、
初始化自身和超过 64 项的 capture 均被拒绝。函数值不支持 equality 或 ordering。

`map(iterable, callback)`、`filter(iterable, predicate)` 和
`fold(iterable, initial, callback)` 是 closed generic operation，不是用户可覆盖的函数。
iterable 只接受 `list<T>`、`range<int>` 或 `map<K, V>`；map 的元素类型是 `(K, V)`。
`for value in iterable { body }` 是返回 `list<R>` 的立即 lexical iteration，lower 为 verifier 约束的
non-escaping callback。它可调用或捕获外层函数 local，但 callback 值不能逃逸到普通表达式。
function type 与 closure 都必须显式携带 effect contract。aggregate 可传递嵌套 function value，
callable metadata 沿 collection、tuple 与 nominal projection 保留，并由 Core verifier 重算。

同 module function 通过完整 call graph 解析，因此支持 forward call，并在 build execution 前拒绝
direct/indirect recursion、unknown call、arity/type mismatch 以及同模块调用链的 depth overflow。
跨 module 的调用目标已经编译为确定的 function，但链路会在 imported function graph 之间继续；
bounded evaluator 因此还会在执行时对整条跨模块 call chain 应用同一个 call-depth guard。即使
function 未使用，也必须完成 parse、name resolution 与 type check。

`temporal-target` 与 `temporal-property` 枚举完整 Surface 闭合集；允许的 target/property 配对、selector
类型和结果类型由 [Temporal 动态叶矩阵](../language-reference/executable-temporal-sinks.md) 定义。
`mask-index` 是不带符号的十进制 `u32`，超出范围会在 parser 边界失败。语法接受 optional resource
binding，后续 typed sink resolution 仍会拒绝 owner、selector、property 或 clock 不匹配。

constant 允许 forward reference 和 exact pure expression。expression token 之间允许 `//` 与
`/* ... */` comment。组件复用由 module、typed function、nominal value 和 method 表达，不存在
`${expression}` source injection 或生成源码后重解析。

当前 grammar 已包含 block control flow、immutable list/map/tuple value、有限范围、typed closure、
有界 iteration、nominal struct/enum、exhaustive match 与静态 method。`animate` body 可以调用 Pure
函数，并从 verified Core residualize 为 random-access Temporal program。路线见
[Executable VEAC Language RFC](../rfcs/executable-veac-language.md)。

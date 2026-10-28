# Lama language stack interpreter

## Installation

1. Install `rust` and `make`
2. Build
```shell
cargo build --release
```
3. Cargo usually places binary in `target/release/`.

`lama-intr` requires `x86_64` architecture with `linux` os

## Testing

### Interpreter

If you have `lamac` 1.3 installed, you can verify bytecode interpreter against
`lamac` stack machine code interpreter (`lamac -s`):

```shell
# build debug version
cargo build 

cd test
bash test.sh
```

### Analyzer
If you have `lamac` 1.3 installed, ou can analyze `Sort.lama` file from Lama preformance suite with the following command:
```shell
lamac -b Lama/perfomance/Sort.lama
cargo run --release -- --file Sort.bc --analyze
```

## Usage

### Interpretation
Interpret Lama bytecode (`*.bc` files) with following command
```shell
cargo run --release --  --file foo.bc
```

### Analysis
```shell
cargo run --release -- --file boo.bc --analyze
```
This command will print

## Performance
This interpreter is really slow, compared to `lamac -s` implementation.
Measured for `Sort.lama` file from Lama performance suite, with array size=500.
```shell
$ time -f "%E" lamac -s Lama/performance/Sort.lama </dev/null
0:00.89
$ time -f "%E" ./target/release/lama-intr --file Sort.bc
0:18.87
```

## Notes

This implementation uses patched version of lama runtime. Namely this assertions were patched out:
```diff
--- a/runtime/runtime.c
+++ b/runtime/runtime.c
@@ -13,10 +13,8 @@ extern size_t __gc_stack_top, __gc_stack_bottom;
   flag      = __gc_stack_top == 0;                                                                 \
   if (flag) { __gc_stack_top = (size_t)__builtin_frame_address(0); }                               \
   assert(__gc_stack_top != 0);                                                                     \
-  assert((__gc_stack_top & 0xF) == 0);                                                             \
-  assert(__builtin_frame_address(0) <= (void *)__gc_stack_top);
+  assert((__gc_stack_top & 0xF) == 0);
 #define POST_GC()                                                                                  \
-  assert(__builtin_frame_address(0) <= (void *)__gc_stack_top);                                    \
   if (flag) { __gc_stack_top = 0; }
 #else
 #define PRE_GC() (void)0
```

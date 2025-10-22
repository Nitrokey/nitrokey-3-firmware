<!-- Copyright (C) 2023 Nitrokey GmbH -->
<!-- SPDX-License-Identifier: LGPL-3.0-only -->

Array Tuple Concat
==================

Concatenate a tuple of arrays (or references to an array) into a single array
The [`concat_arrays`](https://docs.rs/array-tuple-concat/latest/array_tuple_concat/fn.concat_arrays.html) function allows you to concatenate an arbitrary (up to 10) static arrays into one final array.
```rust
use function_concat_array_tuple::concat_arrays;
assert_eq!(concat_arrays(([1u8], [2])), [1u8, 2]);
assert_eq!(concat_arrays((&[1u8], &[2])), [1u8, 2]);
```
The `concat_arrays` function takes a `const N: usize` parameter.
This parameter will often be infered from context.
The `N` const parameter **must** be equal to the sum of the length of the arrays.
For safety, there is a compile-time check that this is correct.
The following would fail to compile:
```rust,compile_fail
use function_concat_array_tuple::concat_arrays;
let _: [u8; 3] = concat_arrays(([1], [2]));
```
```rust,compile_fail
use function_concat_array_tuple::concat_arrays;
let _: [u8; 3] = concat_arrays((&[1], &[2]));
```
Due to `const` functions in traits not being possible, this does not work in a `const` context. It is however possible

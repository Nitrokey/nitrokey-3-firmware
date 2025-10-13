#![cfg_attr(not(test), no_std)]
// Copyright (C) 2025 Nitrokey GmbH
// SPDX-License-Identifier: LGPL-3.0-only

//! Concatenate a tuple of arrays (or references to an array) into a single array
//!
//! The [`concat_arrays`][] function allows you to concatenate an arbitrary (up to 10) static arrays into one final array.
//!
//! ```rust
//!# use array_tuple_concat::concat_arrays;
//! assert_eq!(concat_arrays(([1u8], [2])), [1u8, 2]);
//! assert_eq!(concat_arrays((&[1u8], &[2])), [1u8, 2]);
//! ```
//!
//! The `concat_arrays` function takes a `const N: usize` parameter.
//! This parameter will often be infered from context.
//! The `N` const parameter **must** be equal to the sum of the length of the arrays.
//! If the `N` const parameter cannot be infered from the context, compilation will fail:
//!
//! ```rust,compile_fail
//!# use array_tuple_concat::concat_arrays;
//! let a = concat_arrays(([1], [2]));
//! ```
//!
//! For safety, there is a compile-time check that the `N` parameter inferred or given explicitely
//! is correct. The following would fail to compile:
//!
//! ```rust,compile_fail
//!# use array_tuple_concat::concat_arrays;
//! let _: [u8; 3] = concat_arrays(([1], [2]));
//! ```
//!
//! ```rust,compile_fail
//!# use array_tuple_concat::concat_arrays;
//! let _: [u8; 3] = concat_arrays((&[1], &[2]));
//! ```
//!
//! Due to `const` functions in traits not being possible, this does not work in a `const` context. It is however possible to use the [`const_array_concat`]() macro or the numbered functions ([`concat_arrays_1`](), [`concat_arrays_2`]()) in const contexts

use core::mem::MaybeUninit;
use core::ptr::{self, copy_nonoverlapping};

mod sealed {
    pub trait Sealed<const N: usize> {
        const REAL_SIZE: usize;
        type Item;
        fn to_array(self) -> [Self::Item; N];
    }
}
pub trait ConcatArray<const N: usize>: sealed::Sealed<N> {}

/// Concatenate a tuple of arrays (or references to an array) into a single array
///
/// This function allows you to concatenate an arbitrary (up to 10) static arrays into one final array.
///
/// ```rust
///# use array_tuple_concat::concat_arrays;
/// assert_eq!(concat_arrays(([1u8], [2])), [1u8, 2]);
/// assert_eq!(concat_arrays((&[1u8], &[2])), [1u8, 2]);
/// ```
///
/// The `concat_arrays` function takes a `const N: usize` parameter.
/// This parameter will often be infered from context.
/// The `N` const parameter **must** be equal to the sum of the length of the arrays.
/// If the `N` const parameter cannot be infered from the context, compilation will fail:
///
/// ```rust,compile_fail
///# use array_tuple_concat::concat_arrays;
/// let a = concat_arrays(([1], [2]));
/// ```
///
/// For safety, there is a compile-time check that this is correct.
/// The following would fail to compile:
///
/// ```rust,compile_fail
///# use array_tuple_concat::concat_arrays;
/// let _: [u8; 3] = concat_arrays(([1], [2]));
/// ```
///
/// ```rust,compile_fail
///# use array_tuple_concat::concat_arrays;
/// let _: [u8; 3] = concat_arrays((&[1], &[2]));
/// ```
///
/// Due to `const` functions in traits not being possible, this does not work in a `const` context. It is however possible
pub fn concat_arrays<const N: usize, T: ConcatArray<N>>(arrays: T) -> [T::Item; N] {
    arrays.to_array()
}

/// Get the total length of a tuple of arrays.
///
/// This function can work in a `const` context
pub const fn concatenated_length<T: ConcatArray<0>>(_arrays: &T) -> usize {
    T::REAL_SIZE
}

macro_rules! impl_concat_array {
    (
        $final_ident:ident, $final_array_name:ident,
        $function_name:ident, $function_name_ref:ident,
        $function_name_ensure_same_type:ident,
        $($const_ident:ident, $array_name:ident, $function_name_inner:ident, $function_name_ref_inner:ident, $function_name_ensure_same_type_inner:ident, )*) => {

        /// Ensure all arrays passed as arguments to the function have the same type and returns the first array
        ///
        /// Used to improve type inference in [`const_array_concat`]
        #[doc(hidden)]
        pub const fn $function_name_ensure_same_type<T, $(const $const_ident: usize,)* const $final_ident: usize>(
             first_arg: [T; $final_ident],
             $(_: &[T; $const_ident],)*
        ) -> [T; $final_ident] {
            first_arg
        }

        /// Concatenates a number of arrays
        ///
        /// [`concat_arrays`][] is probably a simpler choice.
        /// Use this function if you need it in a `const` context
        ///
        /// The [`const_array_concat`] macro can also reduce issues with regards to inference of the size of the final array in a const context
        #[allow(clippy::too_many_arguments)]
        pub const fn $function_name<T, $(const $const_ident: usize,)* const $final_ident: usize, const RESULT_SIZE: usize>
            ($($array_name: [T; $const_ident],)* $final_array_name: [T; $final_ident])
        -> [T; RESULT_SIZE] {
                let _ = const {assert!(RESULT_SIZE == $($const_ident+)* $final_ident)};
                $(
                    let $array_name: MaybeUninit<[T; $const_ident]> = MaybeUninit::new($array_name);
                    let $array_name = $array_name.as_ptr() as *mut T;
                )*

                let $final_array_name: MaybeUninit<[T; $final_ident]> = MaybeUninit::new($final_array_name);
                let $final_array_name = $final_array_name.as_ptr() as *mut T;

                let mut res: MaybeUninit<[T; RESULT_SIZE]> = MaybeUninit::uninit();
                let res_ptr = res.as_mut_ptr() as *mut T;
                let idx = 0;
                $(
                    // Safety: const size check means that the values have not yet been written and are in range
                    unsafe {
                        copy_nonoverlapping($array_name, res_ptr.add(idx), $const_ident);
                    }
                    let idx = idx + $const_ident;
                )*

                unsafe {
                    copy_nonoverlapping($final_array_name, res_ptr.add(idx), $final_ident);
                }

                unsafe {
                    assert!(idx + $final_ident ==  RESULT_SIZE);
                    // Safety: all elements where written
                    res.assume_init()
                }
        }

        // Not public since it's not useful because `clone` is not const
        /// Concatenates a number of array references
        #[allow(clippy::too_many_arguments)]
        fn $function_name_ref<T: Clone, $(const $const_ident: usize,)* const $final_ident: usize, const RESULT_SIZE: usize>
            ($($array_name: &[T; $const_ident],)* $final_array_name: &[T; $final_ident])
        -> [T; RESULT_SIZE] {
                let _ = const {assert!(RESULT_SIZE == $($const_ident+)* $final_ident)};

                let mut res: MaybeUninit<[T; RESULT_SIZE]> = MaybeUninit::uninit();
                let res_ptr = res.as_mut_ptr() as *mut T;
                let idx = 0;
                $(
                    let mut current_offset = 0;
                    while current_offset < $const_ident {
                        // Safety: const size check means that the values have not yet been written and are in range
                        unsafe {
                            ptr::write(res_ptr.add(idx + current_offset), $array_name[current_offset].clone());
                        }
                        current_offset += 1;
                    }
                    let idx = idx + $const_ident;
                )*

                let mut current_offset = 0;
                while current_offset < $final_ident {
                    // Safety: const size check means that the values have not yet been written and are in range
                    unsafe {
                        ptr::write(res_ptr.add(idx + current_offset), $final_array_name[current_offset].clone());
                    }
                    current_offset += 1;
                }

                unsafe {
                    assert!(idx + $final_ident  ==  RESULT_SIZE);
                    // Safety: all elements where written
                    res.assume_init()
                }
        }

        impl<T,$(const $const_ident: usize,)* const $final_ident: usize, const RESULT_SIZE:usize> ConcatArray<RESULT_SIZE> for ($([T; $const_ident],)* [T; $final_ident],) {}

        #[allow(unused_assignments)]
        impl<T, $(const $const_ident: usize,)* const $final_ident: usize, const RESULT_SIZE: usize> sealed::Sealed<RESULT_SIZE> for ($([T; $const_ident],)* [T; $final_ident],) {
            const REAL_SIZE: usize = $($const_ident+)* $final_ident;
            type Item = T;
            fn to_array(self) -> [T; RESULT_SIZE] {
                let ($($array_name,)* $final_array_name,) = self;
                $function_name($($array_name,)* $final_array_name,)
            }
        }

        impl<T: Clone, $(const $const_ident: usize,)* const $final_ident: usize, const RESULT_SIZE: usize> ConcatArray<RESULT_SIZE> for ($(&[T; $const_ident],)* &[T; $final_ident],) {}

        #[allow(unused_assignments)]
        impl<T: Clone, $(const $const_ident: usize,)* const $final_ident: usize, const RESULT_SIZE: usize> sealed::Sealed<RESULT_SIZE> for ($(&[T; $const_ident],)* &[T; $final_ident],) {
            const REAL_SIZE: usize = $($const_ident+)* $final_ident;
            type Item = T;
            fn to_array(self) -> [T; RESULT_SIZE] {
                let ($($array_name,)* $final_array_name,) = self;
                $function_name_ref($($array_name,)* $final_array_name,)
            }
        }
        impl_concat_array!($(
            $const_ident,
            $array_name,
            $function_name_inner,
            $function_name_ref_inner,
            $function_name_ensure_same_type_inner,
        )*);
    };
    ()=>{}
}

impl_concat_array!(
    A,
    a,
    concat_arrays_10,
    concat_arrays_ref_10,
    ensure_same_type_10,
    B,
    b,
    concat_arrays_9,
    concat_arrays_ref_9,
    ensure_same_type_9,
    C,
    c,
    concat_arrays_8,
    concat_arrays_ref_8,
    ensure_same_type_8,
    D,
    d,
    concat_arrays_7,
    concat_arrays_ref_7,
    ensure_same_type_7,
    E,
    e,
    concat_arrays_6,
    concat_arrays_ref_6,
    ensure_same_type_6,
    F,
    f,
    concat_arrays_5,
    concat_arrays_ref_5,
    ensure_same_type_5,
    G,
    g,
    concat_arrays_4,
    concat_arrays_ref_4,
    ensure_same_type_4,
    H,
    h,
    concat_arrays_3,
    concat_arrays_ref_3,
    ensure_same_type_3,
    I,
    i,
    concat_arrays_2,
    concat_arrays_ref_2,
    ensure_same_type_2,
    J,
    j,
    concat_arrays_1,
    concat_arrays_ref_1,
    ensure_same_type_1,
);

impl<T, const N: usize> sealed::Sealed<N> for [T; N] {
    const REAL_SIZE: usize = N;
    type Item = T;
    fn to_array(self) -> [T; N] {
        self
    }
}

impl<T: Clone, const N: usize> sealed::Sealed<N> for &[T; N] {
    const REAL_SIZE: usize = N;
    type Item = T;

    fn to_array(self) -> [T; N] {
        self.clone()
    }
}

/// Macro to concatenate arrays in a const context
///
/// Unlike [`concat_arrays`](), this macro works in const context and does not need to be able to infer the
/// length of the resulting array from the context.
/// 
/// ```rust
///# use array_tuple_concat::const_array_concat;
/// let array = const { const_array_concat!(const { [1] }, const { [] }, const { [] }) };
/// assert_eq!(array, [1].as_slice());
/// ```
///
/// However, it has some restrictions. All array arguments need to be evaluatable in a const context,
/// and at least one argument need to have its type defined, otherwise inference will break
///
/// ```compile_fail
///# use array_tuple_concat::const_array_concat;
/// let array = const_array_concat!(const { [] }, const { [] });
/// ```
///
/// Instead, you need to specify the type of at least one array, either by having a value, or through the full array syntax:
///
/// ```
///# use array_tuple_concat::const_array_concat;
/// let array = const_array_concat!(const { [0u8; 0] }, const { [] });
/// ```
///
/// This macro works with up to ten arrays
#[rustfmt::skip]
#[macro_export]
macro_rules! const_array_concat {
    (
        const { $a0:expr }
    ) => {{
        $crate::concat_arrays_1::<
            _,
            { $crate::ensure_same_type_1($a0).len() },
            { $crate::concatenated_length(&($a0,)) },
        >($crate::ensure_same_type_1($a0))
    }};
    (
        const { $a0:expr },
        const { $a1:expr } $(,)?
    ) => {{
        $crate::concat_arrays_2::<
            _,
            { $crate::ensure_same_type_2($a0, &$a1).len() },
            { $crate::ensure_same_type_2($a1, &$a0).len() },
            { $crate::concatenated_length(&($a0, $a1)) },
        >(
            $crate::ensure_same_type_2($a0, &$a1),
            $crate::ensure_same_type_2($a1, &$a0),
        )
    }};
    (
        const { $a0:expr },
        const { $a1:expr },
        const { $a2:expr } $(,)?
    ) => {{
        $crate::concat_arrays_3::<
            _,
            { $crate::ensure_same_type_3($a0, &$a1, &$a2).len() },
            { $crate::ensure_same_type_3($a1, &$a0, &$a2).len() },
            { $crate::ensure_same_type_3($a2, &$a0, &$a1).len() },
            { $crate::concatenated_length(&($a0, $a1, $a2)) },
        >(
            $crate::ensure_same_type_3($a0, &$a1, &$a2),
            $crate::ensure_same_type_3($a1, &$a0, &$a2),
            $crate::ensure_same_type_3($a2, &$a0, &$a1),
        )
    }};
    (
        const { $a0:expr },
        const { $a1:expr },
        const { $a2:expr },
        const { $a3:expr } $(,)?
    ) => {{
        $crate::concat_arrays_4::<
            _,
            { $crate::ensure_same_type_4($a0, &$a1, &$a2, &$a3).len() },
            { $crate::ensure_same_type_4($a1, &$a0, &$a2, &$a3).len() },
            { $crate::ensure_same_type_4($a2, &$a0, &$a1, &$a3).len() },
            { $crate::ensure_same_type_4($a3, &$a0, &$a1, &$a2).len() },
            { $crate::concatenated_length(&($a0, $a1, $a2, $a3)) },
        >(
            $crate::ensure_same_type_4($a0, &$a1, &$a2, &$a3),
            $crate::ensure_same_type_4($a1, &$a0, &$a2, &$a3),
            $crate::ensure_same_type_4($a2, &$a0, &$a1, &$a3),
            $crate::ensure_same_type_4($a3, &$a0, &$a1, &$a2),
        )
    }};
    (
        const { $a0:expr },
        const { $a1:expr },
        const { $a2:expr },
        const { $a3:expr },
        const { $a4:expr } $(,)?
    ) => {{
        $crate::concat_arrays_5::<
            _,
            { $crate::ensure_same_type_5($a0, &$a1, &$a2, &$a3, &$a4).len() },
            { $crate::ensure_same_type_5($a1, &$a0, &$a2, &$a3, &$a4).len() },
            { $crate::ensure_same_type_5($a2, &$a0, &$a1, &$a3, &$a4).len() },
            { $crate::ensure_same_type_5($a3, &$a0, &$a1, &$a2, &$a4).len() },
            { $crate::ensure_same_type_5($a4, &$a0, &$a1, &$a2, &$a3).len() },
            { $crate::concatenated_length(&($a0, $a1, $a2, $a3, $a4)) },
        >(
            $crate::ensure_same_type_5($a0, &$a1, &$a2, &$a3, &$a4),
            $crate::ensure_same_type_5($a1, &$a0, &$a2, &$a3, &$a4),
            $crate::ensure_same_type_5($a2, &$a0, &$a1, &$a3, &$a4),
            $crate::ensure_same_type_5($a3, &$a0, &$a1, &$a2, &$a4),
            $crate::ensure_same_type_5($a4, &$a0, &$a1, &$a2, &$a3),
        )
    }};
    (
        const { $a0:expr },
        const { $a1:expr },
        const { $a2:expr },
        const { $a3:expr },
        const { $a4:expr },
        const { $a5:expr } $(,)?
    ) => {{
        $crate::concat_arrays_6::<
            _,
            { $crate::ensure_same_type_6($a0, &$a1, &$a2, &$a3, &$a4, &$a5).len() },
            { $crate::ensure_same_type_6($a1, &$a0, &$a2, &$a3, &$a4, &$a5).len() },
            { $crate::ensure_same_type_6($a2, &$a0, &$a1, &$a3, &$a4, &$a5).len() },
            { $crate::ensure_same_type_6($a3, &$a0, &$a1, &$a2, &$a4, &$a5).len() },
            { $crate::ensure_same_type_6($a4, &$a0, &$a1, &$a2, &$a3, &$a5).len() },
            { $crate::ensure_same_type_6($a5, &$a0, &$a1, &$a2, &$a3, &$a4).len() },
            { $crate::concatenated_length(&($a0, $a1, $a2, $a3, $a4, $a5)) },
        >(
            $crate::ensure_same_type_6($a0, &$a1, &$a2, &$a3, &$a4, &$a5),
            $crate::ensure_same_type_6($a1, &$a0, &$a2, &$a3, &$a4, &$a5),
            $crate::ensure_same_type_6($a2, &$a0, &$a1, &$a3, &$a4, &$a5),
            $crate::ensure_same_type_6($a3, &$a0, &$a1, &$a2, &$a4, &$a5),
            $crate::ensure_same_type_6($a4, &$a0, &$a1, &$a2, &$a3, &$a5),
            $crate::ensure_same_type_6($a5, &$a0, &$a1, &$a2, &$a3, &$a4),
        )
    }};
    (
        const { $a0:expr },
        const { $a1:expr },
        const { $a2:expr },
        const { $a3:expr },
        const { $a4:expr },
        const { $a5:expr },
        const { $a6:expr } $(,)?
    ) => {{
        $crate::concat_arrays_7::<
            _,
            { $crate::ensure_same_type_7($a0, &$a1, &$a2, &$a3, &$a4, &$a5, &$a6).len() },
            { $crate::ensure_same_type_7($a1, &$a0, &$a2, &$a3, &$a4, &$a5, &$a6).len() },
            { $crate::ensure_same_type_7($a2, &$a0, &$a1, &$a3, &$a4, &$a5, &$a6).len() },
            { $crate::ensure_same_type_7($a3, &$a0, &$a1, &$a2, &$a4, &$a5, &$a6).len() },
            { $crate::ensure_same_type_7($a4, &$a0, &$a1, &$a2, &$a3, &$a5, &$a6).len() },
            { $crate::ensure_same_type_7($a5, &$a0, &$a1, &$a2, &$a3, &$a4, &$a6).len() },
            { $crate::ensure_same_type_7($a6, &$a0, &$a1, &$a2, &$a3, &$a4, &$a5).len() },
            { $crate::concatenated_length(&($a0, $a1, $a2, $a3, $a4, $a5, $a6)) },
        >(
            $crate::ensure_same_type_7($a0, &$a1, &$a2, &$a3, &$a4, &$a5, &$a6),
            $crate::ensure_same_type_7($a1, &$a0, &$a2, &$a3, &$a4, &$a5, &$a6),
            $crate::ensure_same_type_7($a2, &$a0, &$a1, &$a3, &$a4, &$a5, &$a6),
            $crate::ensure_same_type_7($a3, &$a0, &$a1, &$a2, &$a4, &$a5, &$a6),
            $crate::ensure_same_type_7($a4, &$a0, &$a1, &$a2, &$a3, &$a5, &$a6),
            $crate::ensure_same_type_7($a5, &$a0, &$a1, &$a2, &$a3, &$a4, &$a6),
            $crate::ensure_same_type_7($a6, &$a0, &$a1, &$a2, &$a3, &$a4, &$a5),
        )
    }};
    (
        const { $a0:expr },
        const { $a1:expr },
        const { $a2:expr },
        const { $a3:expr },
        const { $a4:expr },
        const { $a5:expr },
        const { $a6:expr },
        const { $a7:expr } $(,)?
    ) => {{
        $crate::concat_arrays_8::<
            _,
            { $crate::ensure_same_type_8($a0, &$a1, &$a2, &$a3, &$a4, &$a5, &$a6, &$a7).len() },
            { $crate::ensure_same_type_8($a1, &$a0, &$a2, &$a3, &$a4, &$a5, &$a6, &$a7).len() },
            { $crate::ensure_same_type_8($a2, &$a0, &$a1, &$a3, &$a4, &$a5, &$a6, &$a7).len() },
            { $crate::ensure_same_type_8($a3, &$a0, &$a1, &$a2, &$a4, &$a5, &$a6, &$a7).len() },
            { $crate::ensure_same_type_8($a4, &$a0, &$a1, &$a2, &$a3, &$a5, &$a6, &$a7).len() },
            { $crate::ensure_same_type_8($a5, &$a0, &$a1, &$a2, &$a3, &$a4, &$a6, &$a7).len() },
            { $crate::ensure_same_type_8($a6, &$a0, &$a1, &$a2, &$a3, &$a4, &$a5, &$a7).len() },
            { $crate::ensure_same_type_8($a7, &$a0, &$a1, &$a2, &$a3, &$a4, &$a5, &$a6).len() },
            { $crate::concatenated_length(&($a0, $a1, $a2, $a3, $a4, $a5, $a6, $a7)) },
        >(
            $crate::ensure_same_type_8($a0, &$a1, &$a2, &$a3, &$a4, &$a5, &$a6, &$a7),
            $crate::ensure_same_type_8($a1, &$a0, &$a2, &$a3, &$a4, &$a5, &$a6, &$a7),
            $crate::ensure_same_type_8($a2, &$a0, &$a1, &$a3, &$a4, &$a5, &$a6, &$a7),
            $crate::ensure_same_type_8($a3, &$a0, &$a1, &$a2, &$a4, &$a5, &$a6, &$a7),
            $crate::ensure_same_type_8($a4, &$a0, &$a1, &$a2, &$a3, &$a5, &$a6, &$a7),
            $crate::ensure_same_type_8($a5, &$a0, &$a1, &$a2, &$a3, &$a4, &$a6, &$a7),
            $crate::ensure_same_type_8($a6, &$a0, &$a1, &$a2, &$a3, &$a4, &$a5, &$a7),
            $crate::ensure_same_type_8($a7, &$a0, &$a1, &$a2, &$a3, &$a4, &$a5, &$a6),
        )
    }};
    (
        const { $a0:expr },
        const { $a1:expr },
        const { $a2:expr },
        const { $a3:expr },
        const { $a4:expr },
        const { $a5:expr },
        const { $a6:expr },
        const { $a7:expr },
        const { $a8:expr } $(,)?
    ) => {{
        $crate::concat_arrays_9::<
            _,
            { $crate::ensure_same_type_9($a0, &$a1, &$a2, &$a3, &$a4, &$a5, &$a6, &$a7, &$a8).len() },
            { $crate::ensure_same_type_9($a1, &$a0, &$a2, &$a3, &$a4, &$a5, &$a6, &$a7, &$a8).len() },
            { $crate::ensure_same_type_9($a2, &$a0, &$a1, &$a3, &$a4, &$a5, &$a6, &$a7, &$a8).len() },
            { $crate::ensure_same_type_9($a3, &$a0, &$a1, &$a2, &$a4, &$a5, &$a6, &$a7, &$a8).len() },
            { $crate::ensure_same_type_9($a4, &$a0, &$a1, &$a2, &$a3, &$a5, &$a6, &$a7, &$a8).len() },
            { $crate::ensure_same_type_9($a5, &$a0, &$a1, &$a2, &$a3, &$a4, &$a6, &$a7, &$a8).len() },
            { $crate::ensure_same_type_9($a6, &$a0, &$a1, &$a2, &$a3, &$a4, &$a5, &$a7, &$a8).len() },
            { $crate::ensure_same_type_9($a7, &$a0, &$a1, &$a2, &$a3, &$a4, &$a5, &$a6, &$a8).len() },
            { $crate::ensure_same_type_9($a8, &$a0, &$a1, &$a2, &$a3, &$a4, &$a5, &$a6, &$a7).len() },
            { $crate::concatenated_length(&($a0, $a1, $a2, $a3, $a4, $a5, $a6, $a7, $a8)) },
        >(
            $crate::ensure_same_type_9($a0, &$a1, &$a2, &$a3, &$a4, &$a5, &$a6, &$a7, &$a8),
            $crate::ensure_same_type_9($a1, &$a0, &$a2, &$a3, &$a4, &$a5, &$a6, &$a7, &$a8),
            $crate::ensure_same_type_9($a2, &$a0, &$a1, &$a3, &$a4, &$a5, &$a6, &$a7, &$a8),
            $crate::ensure_same_type_9($a3, &$a0, &$a1, &$a2, &$a4, &$a5, &$a6, &$a7, &$a8),
            $crate::ensure_same_type_9($a4, &$a0, &$a1, &$a2, &$a3, &$a5, &$a6, &$a7, &$a8),
            $crate::ensure_same_type_9($a5, &$a0, &$a1, &$a2, &$a3, &$a4, &$a6, &$a7, &$a8),
            $crate::ensure_same_type_9($a6, &$a0, &$a1, &$a2, &$a3, &$a4, &$a5, &$a7, &$a8),
            $crate::ensure_same_type_9($a7, &$a0, &$a1, &$a2, &$a3, &$a4, &$a5, &$a6, &$a8),
            $crate::ensure_same_type_9($a8, &$a0, &$a1, &$a2, &$a3, &$a4, &$a5, &$a6, &$a7),
        )
    }};
    (
        const { $a0:expr },
        const { $a1:expr },
        const { $a2:expr },
        const { $a3:expr },
        const { $a4:expr },
        const { $a5:expr },
        const { $a6:expr },
        const { $a7:expr },
        const { $a8:expr },
        const { $a9:expr } $(,)?
    ) => {{
        $crate::concat_arrays_10::<
            _,
            { $crate::ensure_same_type_10($a0, &$a1, &$a2, &$a3, &$a4, &$a5, &$a6, &$a7, &$a8, &$a9).len() },
            { $crate::ensure_same_type_10($a1, &$a0, &$a2, &$a3, &$a4, &$a5, &$a6, &$a7, &$a8, &$a9).len() },
            { $crate::ensure_same_type_10($a2, &$a0, &$a1, &$a3, &$a4, &$a5, &$a6, &$a7, &$a8, &$a9).len() },
            { $crate::ensure_same_type_10($a3, &$a0, &$a1, &$a2, &$a4, &$a5, &$a6, &$a7, &$a8, &$a9).len() },
            { $crate::ensure_same_type_10($a4, &$a0, &$a1, &$a2, &$a3, &$a5, &$a6, &$a7, &$a8, &$a9).len() },
            { $crate::ensure_same_type_10($a5, &$a0, &$a1, &$a2, &$a3, &$a4, &$a6, &$a7, &$a8, &$a9).len() },
            { $crate::ensure_same_type_10($a6, &$a0, &$a1, &$a2, &$a3, &$a4, &$a5, &$a7, &$a8, &$a9).len() },
            { $crate::ensure_same_type_10($a7, &$a0, &$a1, &$a2, &$a3, &$a4, &$a5, &$a6, &$a8, &$a9).len() },
            { $crate::ensure_same_type_10($a8, &$a0, &$a1, &$a2, &$a3, &$a4, &$a5, &$a6, &$a7, &$a9).len() },
            { $crate::ensure_same_type_10($a9, &$a0, &$a1, &$a2, &$a3, &$a4, &$a5, &$a6, &$a7, &$a8).len() },
            { $crate::concatenated_length(&($a0, $a1, $a2, $a3, $a4, $a5, $a6, $a7, $a8, $a9)) },
        >(
            $crate::ensure_same_type_10($a0, &$a1, &$a2, &$a3, &$a4, &$a5, &$a6, &$a7, &$a8, &$a9),
            $crate::ensure_same_type_10($a1, &$a0, &$a2, &$a3, &$a4, &$a5, &$a6, &$a7, &$a8, &$a9),
            $crate::ensure_same_type_10($a2, &$a0, &$a1, &$a3, &$a4, &$a5, &$a6, &$a7, &$a8, &$a9),
            $crate::ensure_same_type_10($a3, &$a0, &$a1, &$a2, &$a4, &$a5, &$a6, &$a7, &$a8, &$a9),
            $crate::ensure_same_type_10($a4, &$a0, &$a1, &$a2, &$a3, &$a5, &$a6, &$a7, &$a8, &$a9),
            $crate::ensure_same_type_10($a5, &$a0, &$a1, &$a2, &$a3, &$a4, &$a6, &$a7, &$a8, &$a9),
            $crate::ensure_same_type_10($a6, &$a0, &$a1, &$a2, &$a3, &$a4, &$a5, &$a7, &$a8, &$a9),
            $crate::ensure_same_type_10($a7, &$a0, &$a1, &$a2, &$a3, &$a4, &$a5, &$a6, &$a8, &$a9),
            $crate::ensure_same_type_10($a8, &$a0, &$a1, &$a2, &$a3, &$a4, &$a5, &$a6, &$a7, &$a9),
            $crate::ensure_same_type_10($a9, &$a0, &$a1, &$a2, &$a3, &$a4, &$a5, &$a6, &$a7, &$a8),
        )
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tests_values() {
        assert_eq!(concat_arrays(([1u8], [2])), [1u8, 2]);
        assert_eq!(concat_arrays(([1u8, 2, 3],)), [1u8, 2, 3]);
        assert_eq!(concat_arrays(([], [1, 2, 3])), [1u8, 2, 3]);
        assert_eq!(concat_arrays(([1], [2], [3], [4])), [1u8, 2, 3, 4]);
        assert_eq!(
            concat_arrays(([1], [2], [3], [4, 5, 6])),
            [1u8, 2, 3, 4, 5, 6]
        );
        assert_eq!(concat_arrays((&[1u8], &[2])), [1u8, 2]);
        assert_eq!(concat_arrays((&[1u8, 2, 3],)), [1u8, 2, 3]);
        assert_eq!(concat_arrays((&[], &[1, 2, 3])), [1u8, 2, 3]);
        assert_eq!(concat_arrays((&[1], &[2], &[3], &[4])), [1u8, 2, 3, 4]);
        assert_eq!(
            concat_arrays((&[1], &[2], &[3], &[4, 5, 6])),
            [1u8, 2, 3, 4, 5, 6]
        );
        assert_eq!(concat_arrays_2([1u8], [2]), [1u8, 2]);
        assert_eq!(concat_arrays_1([1u8, 2, 3]), [1u8, 2, 3]);
        assert_eq!(concat_arrays_2([], [1, 2, 3]), [1u8, 2, 3]);
        assert_eq!(concat_arrays_4([1], [2], [3], [4]), [1u8, 2, 3, 4]);
        assert_eq!(
            concat_arrays_4([1], [2], [3], [4, 5, 6]),
            [1u8, 2, 3, 4, 5, 6]
        );
        assert_eq!(concat_arrays_ref_2(&[1u8], &[2]), [1u8, 2]);
        assert_eq!(concat_arrays_ref_1(&[1u8, 2, 3]), [1u8, 2, 3]);
        assert_eq!(concat_arrays_ref_2(&[], &[1, 2, 3]), [1u8, 2, 3]);
        assert_eq!(concat_arrays_ref_4(&[1], &[2], &[3], &[4]), [1u8, 2, 3, 4]);
        assert_eq!(
            concat_arrays_ref_4(&[1], &[2], &[3], &[4, 5, 6]),
            [1u8, 2, 3, 4, 5, 6]
        );

        // Non-copy concatenations
        assert_eq!(
            concat_arrays((["1".to_string()], ["2".to_string()])),
            ["1".to_string(), "2".to_string()]
        );
        assert_eq!(
            concat_arrays((["1".to_string(), "2".to_string(), "3".to_string()],)),
            ["1".to_string(), "2".to_string(), "3".to_string()]
        );
        assert_eq!(
            concat_arrays(([], ["1".to_string(), "2".to_string(), "3".to_string()])),
            ["1".to_string(), "2".to_string(), "3".to_string()]
        );
        assert_eq!(
            concat_arrays((
                ["1".to_string()],
                ["2".to_string()],
                ["3".to_string()],
                ["4".to_string()]
            )),
            [
                "1".to_string(),
                "2".to_string(),
                "3".to_string(),
                "4".to_string()
            ]
        );
        assert_eq!(
            concat_arrays((
                ["1".to_string()],
                ["2".to_string()],
                ["3".to_string()],
                ["4".to_string(), "5".to_string(), "6".to_string()]
            )),
            [
                "1".to_string(),
                "2".to_string(),
                "3".to_string(),
                "4".to_string(),
                "5".to_string(),
                "6".to_string()
            ]
        );
        // With clone
        assert_eq!(
            concat_arrays((&["1".to_string()], &["2".to_string()])),
            ["1".to_string(), "2".to_string()]
        );
        assert_eq!(
            concat_arrays((&["1".to_string(), "2".to_string(), "3".to_string()],)),
            ["1".to_string(), "2".to_string(), "3".to_string()]
        );
        assert_eq!(
            concat_arrays((&[], &["1".to_string(), "2".to_string(), "3".to_string()])),
            ["1".to_string(), "2".to_string(), "3".to_string()]
        );
        assert_eq!(
            concat_arrays((
                &["1".to_string()],
                &["2".to_string()],
                &["3".to_string()],
                &["4".to_string()]
            )),
            [
                "1".to_string(),
                "2".to_string(),
                "3".to_string(),
                "4".to_string()
            ]
        );
        assert_eq!(
            concat_arrays((
                &["1".to_string()],
                &["2".to_string()],
                &["3".to_string()],
                &["4".to_string(), "5".to_string(), "6".to_string()]
            )),
            [
                "1".to_string(),
                "2".to_string(),
                "3".to_string(),
                "4".to_string(),
                "5".to_string(),
                "6".to_string()
            ]
        );

        assert_eq!(
            concat_arrays(([""; 0], [""; 0], [""; 0], [""; 0],)),
            [""; 0]
        );
        assert_eq!(concat_arrays(([0u8; 0], [1, 2], [3])), [1, 2, 3]);
    }
}

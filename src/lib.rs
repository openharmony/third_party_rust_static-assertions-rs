//! Compile-time assertions to ensure that invariants are met.
//!
//! # Usage
//!
//! This crate is available [on crates.io][crate] and can be used by adding the
//! following to your project's `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! static_assertions = "0.2.4"
//! ```
//!
//! and this to your crate root:
//!
//! ```
//! #[macro_use]
//! extern crate static_assertions;
//! # fn main() {}
//! ```
//!
//! # Limitations
//!
//! Due to implementation details, some macros can only be used normally from
//! within the context of a function. To use these macros in other contexts, a
//! unique label must be provided.
//!
//! This issue can be followed [here][issue1]. Feedback and potential solutions
//! are welcome!
//!
//! [issue1]: https://github.com/nvzqz/static-assertions-rs/issues/1
//! [crate]: https://crates.io/crates/static_assertions

#![no_std]

#![deny(unused_macros)]

#[doc(hidden)]
pub extern crate core as _core;

/// Asserts that the configuration is set.
///
/// # Examples
///
/// A project may not support a set of configurations and thus you may want to
/// report why:
///
/// ```
/// # #[macro_use]
/// # extern crate static_assertions;
/// // We should only be compiling for Unix or Linux
/// # #[cfg(any(unix, linux))]
/// assert_cfg!(any(unix, linux));
/// # fn main() {}
/// ```
///
/// If users need to specify a database back-end:
///
/// ```
/// # #[macro_use]
/// # extern crate static_assertions;
/// # #[cfg(target_pointer_width = "0")] // Impossible
/// assert_cfg!("Must exclusively use MySQL or MongoDB as database back-end",
///             all(not(all(feature = "mysql", feature = "mongodb")),
///                 any(    feature = "mysql", feature = "mongodb")));
/// # fn main() {}
/// ```
///
/// We can't be compiling for both Unix _and_ Windows simultaneously:
///
/// ```compile_fail
/// # #[macro_use] extern crate static_assertions;
/// # fn main() {
/// assert_cfg!(all(unix, windows));
/// # }
/// ```
#[macro_export]
macro_rules! assert_cfg {
    () => {};
    ($msg:expr, $($cfg:tt)*) => {
        #[cfg(not($($cfg)*))]
        compile_error!($msg);
    };
    ($($cfg:tt)*) => {
        #[cfg(not($($cfg)*))]
        compile_error!(concat!("Cfg does not pass: ", stringify!($($cfg)*)));
    };
}

/// Asserts at compile-time that the types have equal sizes.
///
/// When performing operations such as pointer casts or dealing with [`usize`]
/// versus [`u64`] versus [`u32`], the size of your types matter. This is where
/// this macro comes into play.
///
/// # Alternatives
///
/// There are also [`assert_eq_size_val`](macro.assert_eq_size_val.html) and
/// [`assert_eq_size_ptr`](macro.assert_eq_size_ptr.html). Instead of specifying
/// types to compare, values' sizes can be directly compared against each other.
///
/// # Example
///
/// ```
/// # #[macro_use]
/// # extern crate static_assertions;
/// // Can be declared outside of a function if labeled
/// assert_eq_size!(bytes; (u8, u8), u16);
///
/// fn main() {
///     // Supports unlimited arguments:
///     assert_eq_size!([u8; 4], (u16, u16), u32);
/// }
/// ```
///
/// The following produces a compilation failure because `u32` has 4 times the
/// size of `u8`:
///
/// ```compile_fail
/// # #[macro_use] extern crate static_assertions;
/// # fn main() {
/// assert_eq_size!(u32, u8);
/// # }
/// ```
///
/// [`usize`]: https://doc.rust-lang.org/std/primitive.usize.html
/// [`u64`]: https://doc.rust-lang.org/std/primitive.u64.html
/// [`u32`]: https://doc.rust-lang.org/std/primitive.u32.html
#[macro_export]
macro_rules! assert_eq_size {
    ($x:ty, $($xs:ty),+ $(,)*) => {
        $(let _ = $crate::_core::mem::transmute::<$x, $xs>;)+
    };
    ($label:ident; $($xs:tt)+) => {
        #[allow(dead_code, non_snake_case)]
        fn $label() { assert_eq_size!($($xs)+); }
    };
}

/// Asserts at compile-time that the values pointed to have equal sizes.
///
/// This especially is useful for when coercing pointers between different types
/// and ensuring the underlying values are the same size.
///
/// # Example
///
/// ```
/// # #[macro_use]
/// # extern crate static_assertions;
/// fn operation(x: &(u32, u32), y: &[u16; 4]) {
///     assert_eq_size_ptr!(x, y);
/// }
/// # fn main() {}
/// ```
#[macro_export]
macro_rules! assert_eq_size_ptr {
    ($x:expr, $($xs:expr),+ $(,)*) => {
        #[allow(unknown_lints, forget_copy, unused_unsafe, useless_transmute)]
        unsafe {
            use $crate::_core::{mem, ptr};
            let mut copy = ptr::read($x);
            $(ptr::write(&mut copy, mem::transmute(ptr::read($xs)));)+
            mem::forget(copy);
        }
    }
}

/// Asserts at compile-time that the values have equal sizes.
///
/// This macro doesn't consume its arguments and thus works for
/// non-[`Clone`]able values.
///
/// # Example
///
/// ```
/// # #[macro_use]
/// # extern crate static_assertions;
/// # fn main() {
/// struct Byte(u8);
///
/// let x = 10u8;
/// let y = Byte(42); // Works for non-cloneable types
///
/// assert_eq_size_val!(x, y);
/// assert_eq_size_val!(x, y, 0u8);
/// # }
/// ```
///
/// Even though both values are 0, they are of types with different sizes:
///
/// ```compile_fail
/// # #[macro_use] extern crate static_assertions;
/// # fn main() {
/// assert_eq_size_val!(0u8, 0u32);
/// # }
/// ```
///
/// [`Clone`]: https://doc.rust-lang.org/std/clone/trait.Clone.html
#[macro_export]
macro_rules! assert_eq_size_val {
    ($x:expr, $($xs:expr),+ $(,)*) => {
        assert_eq_size_ptr!(&$x, $(&$xs),+);
    }
}

/// Asserts at compile-time that the constant expression evaluates to `true`.
///
/// There also exists [`const_assert_eq`](macro.const_assert_eq.html) for
/// validating whether a sequence of expressions are equal to one another.
///
/// # Example
///
/// Constant expressions can be ensured to have certain properties via this
/// macro If the expression evaluates to `false`, the file will fail to compile.
/// This is synonymous to [`static_assert` in C++][static_assert].
///
/// As a [limitation](index.html#limitations), a unique label is required if
/// the macro is used outside of a function.
///
/// ```
/// # #[macro_use]
/// # extern crate static_assertions;
/// const FIVE: usize = 5;
///
/// const_assert!(twenty_five; FIVE * FIVE == 25);
///
/// fn main() {
///     const_assert!(2 + 2 == 4);
///     const_assert!(FIVE - FIVE == 0);
/// }
/// ```
///
/// Some expressions are blatantly false:
///
/// ```compile_fail
/// # #[macro_use] extern crate static_assertions;
/// # fn main() {
/// const_assert!(1 >= 2);
/// # }
/// ```
///
/// [static_assert]: http://en.cppreference.com/w/cpp/language/static_assert
#[macro_export]
macro_rules! const_assert {
    ($($xs:expr),+ $(,)*) => {
        #[allow(unknown_lints, eq_op)]
        let _ = [(); 0 - !($($xs)&&+) as usize];
    };
    ($label:ident; $($xs:tt)+) => {
        #[allow(dead_code, non_snake_case)]
        fn $label() { const_assert!($($xs)+); }
    };
}

/// Asserts at compile-time that the constants are equal in value.
///
/// # Example
///
/// Works as a shorthand for `const_assert!(a == b)`:
///
/// ```
/// # #[macro_use]
/// # extern crate static_assertions;
/// const TWO: usize = 2;
/// const_assert_eq!(two; TWO * TWO, TWO + TWO, 4);
///
/// fn main() {
///     const NUM: usize = 32;
///     const_assert_eq!(NUM + NUM, 64);
/// }
/// ```
///
/// Just because 2 × 2 = 2 + 2 doesn't mean it holds true for other numbers:
///
/// ```compile_fail
/// # #[macro_use] extern crate static_assertions;
/// # fn main() {
/// const_assert_eq!(4 + 4, 4 * 4);
/// # }
/// ```
#[macro_export]
macro_rules! const_assert_eq {
    ($x:expr, $($xs:expr),+ $(,)*) => {
        const_assert!($($x == $xs),+);
    };
    ($label:ident; $x:expr, $($xs:expr),+ $(,)*) => {
        const_assert!($label; $($x == $xs),+);
    };
}

/// Asserts at compile-time that the traits are object-safe.
///
/// This is useful for when changes are made to a trait that accidentally
/// prevent it from being used as an object. Such a case would be adding a
/// generic method and forgetting to add `where Self: Sized` after it. If left
/// unnoticed, that mistake will affect crate users and break both forward and
/// backward compatibility.
///
/// # Example
///
/// ```
/// # #[macro_use]
/// # extern crate static_assertions;
/// assert_obj_safe!(basic; Send, Sync, AsRef<str>);
///
/// trait MySafeTrait {
///     fn foo(&self) -> u32;
/// }
///
/// fn main() {
///     assert_obj_safe!(MySafeTrait);
/// }
/// ```
///
/// Generics without `where Self: Sized` are not allowed in object-safe traits:
///
/// ```compile_fail
/// # #[macro_use] extern crate static_assertions;
/// trait MyUnsafeTrait {
///     fn bar<T>(&self) -> T;
/// }
///
/// # fn main() {
/// assert_obj_safe!(MyUnsafeTrait);
/// # }
/// ```
#[macro_export]
macro_rules! assert_obj_safe {
    ($($xs:ty),+ $(,)*) => {
        $(let _: Option<&$xs> = None;)+
    };
    ($label:ident; $($xs:tt)+) => {
        #[allow(dead_code, non_snake_case)]
        fn $label() { assert_obj_safe!($($xs)+); }
    };
}

/// Asserts at compile-time that the type has the given fields.
///
/// This is useful for when types have odd fields as a result of `#[cfg]`.
#[macro_export]
macro_rules! assert_fields {
    ($t:path, $($f:ident),+) => {
        $(let $t { $f: _, .. };)+
    };
    ($label:ident; $($xs:tt)+) => {
        #[allow(dead_code, non_snake_case)]
        fn $label() { assert_fields!($($xs)+); }
    };
}

/// Asserts at compile-time that the type implements the given traits.
///
/// # Examples
///
/// Can be used to ensure types implement [`Send`], [`Sync`], and other traits:
///
/// ```
/// # #[macro_use] extern crate static_assertions;
/// # fn main() {}
/// assert_impl!(str; String, Send, Sync, From<&'static str>);
/// assert_impl!(vec; &'static [u8], Into<Vec<u8>>);
/// ```
///
/// Raw pointers cannot be sent between threads safely:
///
/// ```compile_fail
/// # #[macro_use] extern crate static_assertions;
/// # fn main() {
/// assert_impl!(*const u8, Send);
/// # }
/// ```
///
/// [`Send`]: https://doc.rust-lang.org/std/marker/trait.Send.html
/// [`Sync`]: https://doc.rust-lang.org/std/marker/trait.Sync.html
#[macro_export]
macro_rules! assert_impl {
    ($x:ty, $($t:path),+ $(,)*) => {
        $({
            fn assert_impl<T: ?Sized + $t>() {}
            assert_impl::<$x>();
        })+
    };
    ($label:ident; $($xs:tt)+) => {
        #[allow(dead_code, non_snake_case)]
        fn $label() { assert_impl!($($xs)+); }
    };
}

#![allow(unused)]
use core::cell::UnsafeCell;

/// A read-write register of type T.
///
/// Contains one value of type T and provides volatile read/write functions to it.
///
/// # Safety
/// This register should be used where reads and writes to this peripheral register do not
/// lead to memory unsafety. For example, it is a poor choice for a DMA target, but less
/// worrisome for a GPIO output data register.
///
/// Access to this register must be synchronised; if multiple threads (or the main thread and an
/// interrupt service routine) are accessing it simultaneously you may encounter data races.
pub struct RWRegister<T> {
    register: UnsafeCell<T>,
}

impl<T: Copy> RWRegister<T> {
    /// Reads the value of the register.
    #[inline(always)]
    pub fn read(&self) -> T {
        unsafe { ::core::ptr::read_volatile(self.register.get()) }
    }

    /// Writes a new value to the register.
    #[inline(always)]
    pub fn write(&self, val: T) {
        unsafe { ::core::ptr::write_volatile(self.register.get(), val) }
    }
}

/// A read-write register of type T, where read/write access is unsafe.
///
/// Contains one value of type T and provides volatile read/write functions to it.
///
/// # Safety
/// This register should be used where reads and writes to this peripheral may invoke
/// undefined behaviour or memory unsafety. For example, any registers you write a memory
/// address into.
///
/// Access to this register must be synchronised; if multiple threads (or the main thread and an
/// interrupt service routine) are accessing it simultaneously you may encounter data races.
pub struct UnsafeRWRegister<T> {
    register: UnsafeCell<T>,
}

impl<T: Copy> UnsafeRWRegister<T> {
    /// Reads the value of the register.
    #[inline(always)]
    pub unsafe fn read(&self) -> T {
        ::core::ptr::read_volatile(self.register.get())
    }

    /// Writes a new value to the register.
    #[inline(always)]
    pub unsafe fn write(&self, val: T) {
        ::core::ptr::write_volatile(self.register.get(), val)
    }
}

/// A read-only register of type T.
///
/// Contains one value of type T and provides a volatile read function to it.
///
/// # Safety
/// This register should be used where reads and writes to this peripheral register do not
/// lead to memory unsafety.
///
/// Access to this register must be synchronised; if multiple threads (or the main thread and an
/// interrupt service routine) are accessing it simultaneously you may encounter data races.
pub struct RORegister<T> {
    register: UnsafeCell<T>,
}

impl<T: Copy> RORegister<T> {
    /// Reads the value of the register.
    #[inline(always)]
    pub fn read(&self) -> T {
        unsafe { ::core::ptr::read_volatile(self.register.get()) }
    }
}

/// A read-only register of type T, where read access is unsafe.
///
/// Contains one value of type T and provides a volatile read function to it.
///
/// # Safety
/// This register should be used where reads to this peripheral may invoke
/// undefined behaviour or memory unsafety.
///
/// Access to this register must be synchronised; if multiple threads (or the main thread and an
/// interrupt service routine) are accessing it simultaneously you may encounter data races.
pub struct UnsafeRORegister<T> {
    register: UnsafeCell<T>,
}

impl<T: Copy> UnsafeRORegister<T> {
    /// Reads the value of the register.
    #[inline(always)]
    pub unsafe fn read(&self) -> T {
        ::core::ptr::read_volatile(self.register.get())
    }
}

/// A write-only register of type T.
///
/// Contains one value of type T and provides a volatile write function to it.
///
/// # Safety
/// This register should be used where writes to this peripheral register do not lead to memory
/// unsafety.
///
/// Access to this register must be synchronised; if multiple threads (or the main thread and an
/// interrupt service routine) are accessing it simultaneously you may encounter data races.
pub struct WORegister<T> {
    register: UnsafeCell<T>,
}

impl<T: Copy> WORegister<T> {
    /// Writes a new value to the register.
    #[inline(always)]
    pub fn write(&self, val: T) {
        unsafe { ::core::ptr::write_volatile(self.register.get(), val) }
    }
}

/// A write-only register of type T, where write access is unsafe.
///
/// Contains one value of type T and provides a volatile write function to it.
///
/// # Safety
/// This register should be used where reads and writes to this peripheral may invoke
/// undefined behaviour or memory unsafety.
///
/// Access to this register must be synchronised; if multiple threads (or the main thread and an
/// interrupt service routine) are accessing it simultaneously you may encounter data races.
pub struct UnsafeWORegister<T> {
    register: UnsafeCell<T>,
}

impl<T: Copy> UnsafeWORegister<T> {
    /// Writes a new value to the register.
    #[inline(always)]
    pub unsafe fn write(&self, val: T) {
        ::core::ptr::write_volatile(self.register.get(), val)
    }
}

/// Write to a RWRegister or UnsafeRWRegister.
///
/// # Examples
/// ```rust,no_run
/// # use stm32ral::{read_reg, write_reg, modify_reg, reset_reg}; fn main() {
/// // Safely acquire the peripheral instance (will panic if already acquired)
/// let gpioa = stm32ral::gpio::GPIOA::take().unwrap();
///
/// // Write some value to the register.
/// write_reg!(stm32ral::gpio, gpioa, ODR, 1<<3);
///
/// // Write values to specific fields. Unspecified fields are written to 0.
/// write_reg!(stm32ral::gpio, gpioa, MODER, MODER3: Output, MODER4: Analog);
///
/// // Unsafe access without requiring you to first `take()` the instance
/// unsafe { write_reg!(stm32ral::gpio, GPIOA, MODER, MODER3: Output, MODER4: Analog) };
/// # }
/// ```
///
/// # Usage
/// Like `modify_reg!`, this macro can be used in two ways, either with a single value to write to
/// the whole register, or with multiple fields each with their own value.
///
/// In both cases, the first arguments are:
/// * the path to the peripheral module: `stm32ral::gpio`,
/// * a reference to the instance of that peripheral: 'gpioa' (anything which dereferences to
///   `RegisterBlock`, such as `Instance`, `&Instance`, `&RegisterBlock`, or
///   `*const RegisterBlock`),
/// * the register you wish you access: `MODER` (a field on the `RegisterBlock`).
///
/// In the single-value usage, the final argument is just the value to write:
/// ```rust,no_run
/// # use stm32ral::{read_reg, write_reg, modify_reg, reset_reg}; fn main() {
/// # let gpioa = stm32ral::gpio::GPIOA::take().unwrap();
/// // Turn on PA3 (and turn everything else off).
/// write_reg!(stm32ral::gpio, gpioa, ODR, 1<<3);
/// # }
/// ```
///
/// Otherwise, the remaining arguments are each `Field: Value` pairs:
/// ```rust,no_run
/// # use stm32ral::{read_reg, write_reg, modify_reg, reset_reg}; fn main() {
/// // Set PA3 to Output, PA4 to Analog, and everything else to 0 (which is Input).
/// # let gpioa = stm32ral::gpio::GPIOA::take().unwrap();
/// write_reg!(stm32ral::gpio, gpioa, MODER, MODER3: 0b01, MODER4: 0b11);
/// # }
/// ```
/// For fields with annotated values, you can also specify a named value:
/// ```rust,no_run
/// # use stm32ral::{read_reg, write_reg, modify_reg, reset_reg}; fn main() {
/// // As above, but with named values.
/// # let gpioa = stm32ral::gpio::GPIOA::take().unwrap();
/// write_reg!(stm32ral::gpio, gpioa, MODER, MODER3: Output, MODER4: Analog);
/// # }
/// ```
///
/// This macro expands to calling `(*$instance).$register.write(value)`,
/// where in the second usage, the value is computed as the bitwise OR of
/// each field value, which are masked and shifted appropriately for the given field.
/// The named values are brought into scope by `use $peripheral::$register::$field::*` for
/// each field. The same constants could just be specified manually:
/// ```rust,no_run
/// # use stm32ral::{read_reg, write_reg, modify_reg, reset_reg}; fn main() {
/// // As above, but being explicit about named values.
/// # let gpioa = stm32ral::gpio::GPIOA::take().unwrap();
/// write_reg!(stm32ral::gpio, gpioa, MODER, MODER3: stm32ral::gpio::MODER::MODER3::RW::Output,
///                                          MODER4: stm32ral::gpio::MODER::MODER4::RW::Analog);
/// # }
/// ```
///
/// The fully expanded form is equivalent to:
/// ```rust,no_run
/// # use stm32ral::{read_reg, write_reg, modify_reg, reset_reg}; fn main() {
/// // As above, but expanded.
/// # let gpioa = stm32ral::gpio::GPIOA::take().unwrap();
/// (*gpioa).MODER.write(
///     ((stm32ral::gpio::MODER::MODER3::RW::Output << stm32ral::gpio::MODER::MODER3::offset)
///      & stm32ral::gpio::MODER::MODER3::mask)
///     |
///     ((stm32ral::gpio::MODER::MODER4::RW::Analog << stm32ral::gpio::MODER::MODER4::offset)
///      & stm32ral::gpio::MODER::MODER4::mask)
/// );
/// # }
/// ```
///
/// # Safety
/// This macro will require an unsafe function or block when used with an UnsafeRWRegister,
/// but not if used with RWRegister.
///
/// When run in an unsafe context, peripheral instances are directly accessible without requiring
/// having called `take()` beforehand:
/// ```rust,no_run
/// # use stm32ral::{read_reg, write_reg, modify_reg, reset_reg}; fn main() {
/// unsafe { write_reg!(stm32ral::gpio, GPIOA, MODER, MODER3: Output, MODER4: Analog) };
/// # }
/// ```
/// This works because `GPIOA` is a `*const RegisterBlock` in the `stm32ral::gpio` module;
/// and the macro brings such constants into scope and then dereferences the provided reference.
#[macro_export]
#[doc(hidden)]
macro_rules! write_reg {
    ( $periph:path, $instance:expr, $reg:ident, $( $field:ident : $value:expr ),+ ) => {{
        #[allow(unused_imports)]
        use $periph::{*};
        #[allow(unused_imports)]
        (*$instance).$reg.write(
            $({ use $periph::{$reg::$field::{mask, offset, W::*, RW::*}}; ($value << offset) & mask }) | *
        );
    }};
    ( $periph:path, $instance:expr, $reg:ident, $value:expr ) => {{
        #[allow(unused_imports)]
        use $periph::{*};
        (*$instance).$reg.write($value);
    }};
}

/// Modify a RWRegister or UnsafeRWRegister.
///
/// # Examples
/// ```rust,no_run
/// # use stm32ral::{read_reg, write_reg, modify_reg, reset_reg}; fn main() {
/// // Safely acquire the peripheral instance (will panic if already acquired)
/// let gpioa = stm32ral::gpio::GPIOA::take().unwrap();
///
/// // Update the register to ensure bit 3 is set.
/// modify_reg!(stm32ral::gpio, gpioa, ODR, |reg| reg | (1<<3));
///
/// // Write values to specific fields. Unspecified fields are left unchanged.
/// modify_reg!(stm32ral::gpio, gpioa, MODER, MODER3: Output, MODER4: Analog);
///
/// // Unsafe access without requiring you to first `take()` the instance
/// unsafe { modify_reg!(stm32ral::gpio, GPIOA, MODER, MODER3: Output, MODER4: Analog) };
/// # }
/// ```
///
/// # Usage
/// Like `write_reg!`, this macro can be used in two ways, either with a modification of the entire
/// register, or by specifying which fields to change and what value to change them to.
///
/// In both cases, the first arguments are:
/// * the path to the peripheral module: `stm32ral::gpio`,
/// * a reference to the instance of that peripheral: 'gpioa' (anything which dereferences to
///   `RegisterBlock`, such as `Instance`, `&Instance`, `&RegisterBlock`, or
///   `*const RegisterBlock`),
/// * the register you wish you access: `MODER` (a field on the `RegisterBlock`).
///
/// In the whole-register usage, the final argument is a closure that accepts the current value
/// of the register and returns the new value to write:
/// ```rust,no_run
/// # use stm32ral::{read_reg, write_reg, modify_reg, reset_reg}; fn main() {
/// # let gpioa = stm32ral::gpio::GPIOA::take().unwrap();
/// // Turn on PA3 without affecting anything else.
/// modify_reg!(stm32ral::gpio, gpioa, ODR, |reg| reg | (1<<3));
/// # }
/// ```
///
/// Otherwise, the remaining arguments are `Field: Value` pairs:
/// ```rust,no_run
/// # use stm32ral::{read_reg, write_reg, modify_reg, reset_reg}; fn main() {
/// # let gpioa = stm32ral::gpio::GPIOA::take().unwrap();
/// // Set PA3 to Output, PA4 to Analog, and leave everything else unchanged.
/// modify_reg!(stm32ral::gpio, gpioa, MODER, MODER3: 0b01, MODER4: 0b11);
/// # }
/// ```
///
/// For fields with annotated values, you can also specify a named value:
/// ```rust,no_run
/// # use stm32ral::{read_reg, write_reg, modify_reg, reset_reg}; fn main() {
/// # let gpioa = stm32ral::gpio::GPIOA::take().unwrap();
/// // As above, but with named values.
/// modify_reg!(stm32ral::gpio, gpioa, MODER, MODER3: Output, MODER4: Analog);
/// # }
/// ```
///
/// This macro expands to calling `(*instance).register.write(value)`.
/// When called with a closure, `(*instance).register.read()` is called, the result
/// passed in to the closure, and the return value of the closure is used for `value`.
/// When called with `Field: Value` arguments, the current value is read and then masked
/// according to the specified fields, and then ORd with the OR of each field value,
/// each masked and shifted appropriately for the field. The named values are brought into scope
/// by `use peripheral::register::field::*` for each field. The same constants could just be
/// specified manually:
/// ```rust,no_run
/// # use stm32ral::{read_reg, write_reg, modify_reg, reset_reg}; fn main() {
/// # let gpioa = stm32ral::gpio::GPIOA::take().unwrap();
/// // As above, but being explicit about named values.
/// modify_reg!(stm32ral::gpio, gpioa, MODER, MODER3: stm32ral::gpio::MODER::MODER3::RW::Output,
///                                           MODER4: stm32ral::gpio::MODER::MODER4::RW::Analog);
/// # }
/// ```
///
/// The fully expanded form is equivalent to:
/// ```rust,no_run
/// # use stm32ral::{read_reg, write_reg, modify_reg, reset_reg}; fn main() {
/// # let gpioa = stm32ral::gpio::GPIOA::take().unwrap();
/// // As above, but expanded.
/// (*gpioa).MODER.write(
///     (
///         // First read the current value...
///         (*gpioa).MODER.read()
///         // Then AND it with an appropriate mask...
///         &
///         !( stm32ral::gpio::MODER::MODER3::mask | stm32ral::gpio::MODER::MODER4::mask )
///     )
///     // Then OR with each field value.
///     |
///         ((stm32ral::gpio::MODER::MODER3::RW::Output << stm32ral::gpio::MODER::MODER3::offset)
///          & stm32ral::gpio::MODER::MODER3::mask)
///     |
///         ((stm32ral::gpio::MODER::MODER4::RW::Analog << stm32ral::gpio::MODER::MODER3::offset)
///          & stm32ral::gpio::MODER::MODER3::mask)
/// );
/// # }
/// ```
///
/// # Safety
/// This macro will require an unsafe function or block when used with an UnsafeRWRegister,
/// but not if used with RWRegister.
///
/// When run in an unsafe context, peripheral instances are directly accessible without requiring
/// having called `take()` beforehand:
/// ```rust,no_run
/// # use stm32ral::{read_reg, write_reg, modify_reg, reset_reg}; fn main() {
/// unsafe { modify_reg!(stm32ral::gpio, GPIOA, MODER, MODER3: Output, MODER4: Analog) };
/// # }
/// ```
/// This works because `GPIOA` is a `*const RegisterBlock` in the `stm32ral::gpio` module;
/// and the macro brings such constants into scope and then dereferences the provided reference.
#[macro_export]
#[doc(hidden)]
macro_rules! modify_reg {
    ( $periph:path, $instance:expr, $reg:ident, $( $field:ident : $value:expr ),+ ) => {{
        #[allow(unused_imports)]
        use $periph::{*};
        #[allow(unused_imports)]
        (*$instance).$reg.write(
            ((*$instance).$reg.read() & !( $({ use $periph::{$reg::$field::mask}; mask }) | * ))
            | $({ use $periph::{$reg::$field::{mask, offset, W::*, RW::*}}; ($value << offset) & mask }) | *);
    }};
    ( $periph:path, $instance:expr, $reg:ident, $fn:expr ) => {{
        #[allow(unused_imports)]
        use $periph::{*};
        (*$instance).$reg.write($fn((*$instance).$reg.read()));
    }};
}

/// Read the value from a RORegister, RWRegister, UnsafeRORegister, or UnsafeRWRegister.
///
/// # Examples
/// ```rust,no_run
/// # use stm32ral::{read_reg, write_reg, modify_reg, reset_reg}; fn main() {
/// // Safely acquire the peripheral instance (will panic if already acquired)
/// let gpioa = stm32ral::gpio::GPIOA::take().unwrap();
///
/// // Read the whole register.
/// let val = read_reg!(stm32ral::gpio, gpioa, IDR);
///
/// // Read one field from the register.
/// let val = read_reg!(stm32ral::gpio, gpioa, IDR, IDR2);
///
/// // Read multiple fields from the register.
/// let (val1, val2, val3) = read_reg!(stm32ral::gpio, gpioa, IDR, IDR0, IDR1, IDR2);
///
/// // Check if one field is equal to a specific value, with the field's named values in scope.
/// while read_reg!(stm32ral::gpio, gpioa, IDR, IDR2 == High) {}
///
/// // Unsafe access without requiring you to first `take()` the instance
/// let val = unsafe { read_reg!(stm32ral::gpio, GPIOA, IDR) };
/// # }
/// ```
///
/// # Usage
/// Like `write_reg!`, this macro can be used multiple ways, either reading the entire register or
/// reading a one or more fields from it and potentially performing a comparison with one field.
///
/// In all cases, the first arguments are:
/// * the path to the peripheral module: `stm32ral::gpio`,
/// * a reference to the instance of that peripheral: 'gpioa' (anything which dereferences to
///   `RegisterBlock`, such as `Instance`, `&Instance`, `&RegisterBlock`, or
///   `*const RegisterBlock`),
/// * the register you wish to access: `IDR` (a field on the `RegisterBlock`).
///
/// In the whole-register usage, the macro simply returns the register's value:
/// ```rust,no_run
/// # use stm32ral::{read_reg, write_reg, modify_reg, reset_reg}; fn main() {
/// # let gpioa = stm32ral::gpio::GPIOA::take().unwrap();
/// // Read the entire value of GPIOA.IDR into `val`.
/// let val = read_reg!(stm32ral::gpio, gpioa, IDR);
/// # }
/// ```
///
/// For reading individual fields, the macro masks and shifts appropriately:
/// ```rust,no_run
/// # use stm32ral::{read_reg, write_reg, modify_reg, reset_reg}; fn main() {
/// # let gpioa = stm32ral::gpio::GPIOA::take().unwrap();
/// // Read just the value of the field GPIOA.IDR2 into `val`.
/// let val = read_reg!(stm32ral::gpio, gpioa, IDR, IDR2);
///
/// // As above, but expanded for exposition:
/// let val = ((*gpioa).IDR.read() & stm32ral::gpio::IDR::IDR2::mask)
///           >> stm32ral::gpio::IDR::IDR2::offset;
///
/// // Read multiple fields
/// let (val1, val2) = read_reg!(stm32ral::gpio, gpioa, IDR, IDR2, IDR3);
///
/// // As above, but expanded for exposition:
/// let (val1, val2) = { let val = (*gpioa).IDR.read();
///     ((val & stm32ral::gpio::IDR::IDR2::mask) >> stm32ral::gpio::IDR::IDR2::offset,
///      (val & stm32ral::gpio::IDR::IDR3::mask) >> stm32ral::gpio::IDR::IDR3::offset,
///     )};
/// # }
/// ```
///
/// For comparing a single field, the macro masks and shifts and then performs the comparison:
/// ```rust,no_run
/// # use stm32ral::{read_reg, write_reg, modify_reg, reset_reg}; fn main() {
/// # let gpioa = stm32ral::gpio::GPIOA::take().unwrap();
/// # let rcc = stm32ral::rcc::RCC::take().unwrap();
/// // Loop while PA2 is High.
/// while read_reg!(stm32ral::gpio, gpioa, IDR, IDR2 == High) {}
///
/// // Only proceed if the clock is not the HSI.
/// if read_reg!(stm32ral::rcc, rcc, CFGR, SWS != HSI) { }
///
/// // Equivalent expansion:
/// if (((*rcc).CFGR.read() & stm32ral::rcc::CFGR::SWS::mask)
///     >> stm32ral::rcc::CFGR::SWS::offset) != stm32ral::rcc::CFGR::SWS::R::HSI { }
/// # }
/// ```
///
/// # Safety
/// This macro will require an unsafe function or block when used with an UnsafeRWRegister or
/// UnsafeRORegister, but not if used with RWRegister, or RORegister.
///
/// When run in an unsafe context, peripheral instances are directly accessible without requiring
/// having called `take()` beforehand:
/// ```rust,no_run
/// # use stm32ral::{read_reg, write_reg, modify_reg, reset_reg}; fn main() {
/// let val = unsafe { read_reg!(stm32ral::gpio, GPIOA, MODER) };
/// # }
/// ```
/// This works because `GPIOA` is a `*const RegisterBlock` in the `stm32ral::gpio` module;
/// and the macro brings such constants into scope and then dereferences the provided reference.
#[macro_export]
#[doc(hidden)]
macro_rules! read_reg {
    ( $periph:path, $instance:expr, $reg:ident, $( $field:ident ),+ ) => {{
        #[allow(unused_imports)]
        use $periph::{*};
        let val = ((*$instance).$reg.read());
        ( $({
            #[allow(unused_imports)]
            use $periph::{$reg::$field::{mask, offset, R::*, RW::*}};
            (val & mask) >> offset
        }) , *)
    }};
    ( $periph:path, $instance:expr, $reg:ident, $field:ident $($cmp:tt)* ) => {{
        #[allow(unused_imports)]
        use $periph::{*};
        #[allow(unused_imports)]
        use $periph::{$reg::$field::{mask, offset, R::*, RW::*}};
        (((*$instance).$reg.read() & mask) >> offset) $($cmp)*
    }};
    ( $periph:path, $instance:expr, $reg:ident ) => {{
        #[allow(unused_imports)]
        use $periph::{*};
        ((*$instance).$reg.read())
    }};
}

/// Reset a RWRegister, UnsafeRWRegister, WORegister, or UnsafeWORegister to its reset value.
///
/// # Examples
/// ```rust,no_run
/// # use stm32ral::{read_reg, write_reg, modify_reg, reset_reg}; fn main() {
/// // Safely acquire the peripheral instance (will panic if already acquired)
/// let gpioa = stm32ral::gpio::GPIOA::take().unwrap();
///
/// // Reset PA14 and PA15 to their reset state
/// reset_reg!(stm32ral::gpio, gpioa, GPIOA, MODER, MODER14, MODER15);
///
/// // Reset the entire GPIOA.MODER to its reset state
/// reset_reg!(stm32ral::gpio, gpioa, GPIOA, MODER);
/// # }
/// ```
///
/// # Usage
/// Like `write_reg!`, this macro can be used in two ways, either resetting the entire register
/// or just resetting specific fields within in. The register or fields are written with their
/// reset values.
///
/// In both cases, the first arguments are:
/// * the path to the peripheral module: `stm32ral::gpio`,
/// * a reference to the instance of that peripheral: 'gpioa' (anything which dereferences to
///   `RegisterBlock`, such as `Instance`, `&Instance`, `&RegisterBlock`, or
///   `*const RegisterBlock`),
/// * the module for the instance of that peripheral: `GPIOA`,
/// * the register you wish to access: `MODER` (a field on the `RegisterBlock`).
///
/// In the whole-register usage, that's it:
/// ```rust,no_run
/// # use stm32ral::{read_reg, write_reg, modify_reg, reset_reg}; fn main() {
/// # let gpioa = stm32ral::gpio::GPIOA::take().unwrap();
/// // Reset the entire GPIOA.MODER
/// reset_reg!(stm32ral::gpio, gpioa, GPIOA, MODER);
/// # }
/// ```
///
/// Otherwise, the remaining arguments are each field names:
/// ```rust,no_run
/// # use stm32ral::{read_reg, write_reg, modify_reg, reset_reg}; fn main() {
/// # let gpioa = stm32ral::gpio::GPIOA::take().unwrap();
/// // Reset the JTAG pins
/// reset_reg!(stm32ral::gpio, gpioa, GPIOA, MODER, MODER13, MODER14, MODER15);
/// reset_reg!(stm32ral::gpio, gpioa, GPIOB, MODER, MODER3, MODER4);
/// # }
/// ```
///
/// The second form is only available to RWRegister and UnsafeRWRegister, since `.read()` is
/// not available for WORegister and UnsafeWORegister.
///
/// This macro expands to calling `(*$instance).$register.write(value)`, where
/// `value` is either the register's reset value, or the current read value of the register
/// masked appropriately and combined with the reset value for each field.
///
/// # Safety
/// This macro will require an unsafe function or block when used with an UnsafeRWRegister or
/// UnsafeRORegister, but not if used with RWRegister or RORegister.
///
/// When run in an unsafe context, peripheral instances are directly accessible without requiring
/// having called `take()` beforehand:
/// ```rust,no_run
/// # use stm32ral::{read_reg, write_reg, modify_reg, reset_reg}; fn main() {
/// unsafe { reset_reg!(stm32ral::gpio, GPIOA, GPIOA, MODER) };
/// # }
/// ```
/// This works because `GPIOA` is a `*const RegisterBlock` in the `stm32ral::gpio` module;
/// and the macro brings such constants into scope and then dereferences the provided reference.
///
/// Note that the second argument is a `*const` and the third is a path; despite both being written
/// `GPIOA` they are not the same thing.
#[macro_export]
#[doc(hidden)]
macro_rules! reset_reg {
    ( $periph:path, $instance:expr, $instancemod:path, $reg:ident, $( $field:ident ),+ ) => {{
        #[allow(unused_imports)]
        use $periph::{*};
        use $periph::{$instancemod::{reset}};
        #[allow(unused_imports)]
        (*$instance).$reg.write({
            let resetmask: u32 = $({ use $periph::{$reg::$field::mask}; mask }) | *;
            ((*$instance).$reg.read() & !resetmask) | (reset.$reg & resetmask)
        });
    }};
    ( $periph:path, $instance:expr, $instancemod:path, $reg:ident ) => {{
        #[allow(unused_imports)]
        use $periph::{*};
        use $periph::{$instancemod::{reset}};
        (*$instance).$reg.write(reset.$reg);
    }};
}

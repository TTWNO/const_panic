use crate::{fmt::FmtKind, panic_val::PanicVal, utils::WasTruncated};

/// Panics by concatenating the argument slice.
///
/// This is the function that the [`concat_panic`](macro@concat_panic) macro calls to panic.
///
/// # Example
///
/// Here's how to panic with formatting without using any macros:
///
/// ```compile_fail
/// use const_panic::{FmtArg, PanicVal, concat_panic};
///
/// const _: () = concat_panic(&[&[
///     PanicVal::write_str("\nthe error was "),
///     PanicVal::from_u8(100, FmtArg::DISPLAY),
///     PanicVal::write_str(" and "),
///     PanicVal::from_str("\nHello\tworld", FmtArg::DEBUG),
/// ]]);
///
///
/// ```
/// That fails to compile with this error message:
/// ```text
/// error[E0080]: evaluation of constant value failed
///   --> src/concat_panic_.rs:13:15
///    |
/// 6  |   const _: () = concat_panic(&[&[
///    |  _______________^
/// 7  | |     PanicVal::write_str("\nthe error was "),
/// 8  | |     PanicVal::from_u8(100, FmtArg::DISPLAY),
/// 9  | |     PanicVal::write_str(" and "),
/// 10 | |     PanicVal::from_str("\nHello\tworld", FmtArg::DEBUG),
/// 11 | | ]]);
///    | |___^ the evaluated program panicked at '
/// the error was 100 and "\nHello\tworld"', src/concat_panic_.rs:6:15
/// ```
///
#[cold]
#[inline(never)]
#[track_caller]
pub const fn concat_panic(args: &[&[PanicVal<'_>]]) -> ! {
    // The panic message capacity starts small and gets larger each time,
    // so that platforms with smaller stacks can call this at runtime.
    //
    // Also, given that most(?) panic messages are smaller than 1024 bytes long,
    // it's not going to be any less efficient in the common case.
    if let Err(_) = panic_inner::<1024>(args) {}

    if let Err(_) = panic_inner::<{ 1024 * 6 }>(args) {}

    match panic_inner::<MAX_PANIC_MSG_LEN>(args) {
        Ok(x) => match x {},
        Err(_) => panic!(
            "\
            unreachable:\n\
            the `write_panicval_to_buffer` macro must not return Err when \
            $capacity == $max_capacity\
        "
        ),
    }
}

/// The maximum length of panic messages (in bytes),
/// after which the message is truncated.
// this should probably be smaller on platforms where this
// const fn is called at runtime, and the stack is finy.
pub const MAX_PANIC_MSG_LEN: usize = 32768;

macro_rules! write_panicval_to_buffer {
    (
        $outer_label:lifetime,
        $buffer:ident,
        $len:ident,
        ($capacity:expr, $max_capacity:expr),
        $panicval:expr
        $(,)*
    ) => {
        let rem_space = $capacity - $len;
        let arg = $panicval;
        let (mut lpad, mut rpad, string, fmt_kind, was_truncated) = arg.__string(rem_space);
        let trunc_len = was_truncated.get_length(string);

        while lpad != 0 {
            __write_array! {$buffer, $len, b' '}
            lpad -= 1;
        }

        if let FmtKind::Display = fmt_kind {
            let mut i = 0;
            while i < trunc_len {
                __write_array! {$buffer, $len, string[i]}
                i += 1;
            }
        } else if rem_space != 0 {
            __write_array! {$buffer, $len, b'"'}
            let mut i = 0;
            while i < trunc_len {
                use crate::debug_str_fmt::{hex_as_ascii, ForEscaping};

                let c = string[i];
                let mut written_c = c;
                if ForEscaping::is_escaped(c) {
                    __write_array! {$buffer, $len, b'\\'}
                    if ForEscaping::is_backslash_escaped(c) {
                        written_c = ForEscaping::get_backslash_escape(c);
                    } else {
                        __write_array! {$buffer, $len, b'x'}
                        __write_array! {$buffer, $len, hex_as_ascii(c >> 4)}
                        written_c = hex_as_ascii(c & 0b1111);
                    };
                }
                __write_array! {$buffer, $len, written_c}

                i += 1;
            }
            if let WasTruncated::No = was_truncated {
                __write_array_checked! {$buffer, $len, b'"'}
            }
        }

        while rpad != 0 {
            __write_array! {$buffer, $len, b' '}
            rpad -= 1;
        }

        if let WasTruncated::Yes(_) = was_truncated {
            if $capacity < $max_capacity {
                return Err(NotEnoughSpace);
            } else {
                break $outer_label;
            }
        }
    };
}

macro_rules! write_to_buffer {
    ($args:ident, $buffer:ident, $len:ident, $wptb_args:tt $(,)*) => {
        let mut args = $args;
        'outer: while let [mut outer, ref nargs @ ..] = args {
            while let [arg, nouter @ ..] = outer {
                match arg.var {
                    #[cfg(feature = "non_basic")]
                    #[cfg_attr(feature = "docsrs", doc(cfg(feature = "non_basic")))]
                    crate::panic_val::PanicVariant::Slice(slice) => {
                        let mut iter = slice.iter();

                        'iter: loop {
                            let (two_args, niter) = iter.next();

                            let mut two_args: &[_] = &two_args;
                            while let [arg, ntwo_args @ ..] = two_args {
                                write_panicval_to_buffer! {'outer, $buffer, $len, $wptb_args, arg}
                                two_args = ntwo_args;
                            }

                            match niter {
                                Some(x) => iter = x,
                                None => break 'iter,
                            }
                        }
                    }
                    _ => {
                        write_panicval_to_buffer! {'outer, $buffer, $len, $wptb_args, arg}
                    }
                }

                outer = nouter;
            }
            args = nargs;
        }
    };
}

#[cold]
#[inline(never)]
#[track_caller]
const fn panic_inner<const LEN: usize>(args: &[&[PanicVal<'_>]]) -> Result<Never, NotEnoughSpace> {
    let mut buffer = [0u8; LEN];
    let mut len = 0usize;

    write_to_buffer! {
        args,
        buffer,
        len,
        (LEN, MAX_PANIC_MSG_LEN),
    }

    unsafe {
        let str = core::str::from_utf8_unchecked(&buffer);
        panic!("{}", str)
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct NotEnoughSpace;
enum Never {}

#[cfg(feature = "test")]
use crate::test_utils::TestString;

#[doc(hidden)]
#[cfg(feature = "test")]
pub fn format_panic_message<const LEN: usize>(
    args: &[&[PanicVal<'_>]],
    capacity: usize,
    max_capacity: usize,
) -> Result<TestString<LEN>, NotEnoughSpace> {
    let mut buffer = [0u8; LEN];
    let mut len = 0usize;
    {
        // intentionally shadowed
        let buffer = &mut buffer[..capacity];

        write_to_buffer! {
            args,
            buffer,
            len,
            (capacity, max_capacity),
        }
    }

    Ok(TestString { buffer, len })
}

#[cfg(feature = "non_basic")]
#[cfg_attr(feature = "docsrs", doc(cfg(feature = "non_basic")))]
#[doc(hidden)]
pub(crate) const fn make_panic_string<const LEN: usize>(
    args: &[&[PanicVal<'_>]],
) -> Result<crate::ArrayString<LEN>, NotEnoughSpace> {
    let mut buffer = [0u8; LEN];
    let mut len = 0usize;

    write_to_buffer! {
        args,
        buffer,
        len,
        (LEN, LEN + 1),
    }

    assert!(len as u32 as usize == len, "the panic message is too large");

    Ok(crate::ArrayString {
        buffer,
        len: len as u32,
    })
}

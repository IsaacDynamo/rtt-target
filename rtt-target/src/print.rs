use core::cell::RefCell;
use core::fmt::{self, Write as _};
use critical_section::Mutex;

use crate::{TerminalChannel, TerminalWriter, UpChannel};

static PRINT_TERMINAL: Mutex<RefCell<Option<TerminalChannel>>> = Mutex::new(RefCell::new(None));

/// Sets the channel to use for [`rprint`] and [`rprintln`].
pub fn set_print_channel(channel: UpChannel) {
    critical_section::with(|cs| {
        *PRINT_TERMINAL.borrow_ref_mut(cs) = Some(TerminalChannel::new(UpChannel(channel.0)))
    });
}

/// Public due to access from macro.
#[doc(hidden)]
pub mod print_impl {
    use super::*;

    fn with_writer<F: Fn(TerminalWriter) -> ()>(number: u8, f: F) {
        critical_section::with(|cs| {
            if let Some(term) = &mut *PRINT_TERMINAL.borrow_ref_mut(cs) {
                f(term.write(number))
            }
        });
    }

    /// Public due to access from macro.
    #[doc(hidden)]
    pub fn write_str(number: u8, s: &str) {
        with_writer(number, |mut w| {
            w.write_str(s).ok();
        });
    }

    /// Public due to access from macro.
    #[doc(hidden)]
    pub fn write_fmt(number: u8, arg: fmt::Arguments) {
        with_writer(number, |mut w| {
            w.write_fmt(arg).ok();
        });
    }
}

/// Prints to the print RTT channel. Works just like the standard `print`.
///
/// Before use the print channel has to be set with [`rtt_init_print`] or [`set_print_channel`]. If
/// the channel isn't set, the message is silently discarded.
///
/// The macro also supports output to multiple virtual terminals on the channel. Use the syntax
/// ```rprint!(=> 1, "Hello!");``` to write to terminal number 1, for example. Terminal numbers
/// range from 0 to 15.
#[macro_export]
macro_rules! rprint {
    (=> $terminal:expr, $s:expr) => {
        $crate::print_impl::write_str($terminal, $s);
    };
    (=> $terminal:expr, $($arg:tt)*) => {
        $crate::print_impl::write_fmt($terminal, format_args!($($arg)*));
    };
    ($s:expr) => {
        $crate::print_impl::write_str(0, $s);
    };
    ($($arg:tt)*) => {
        $crate::print_impl::write_fmt(0, format_args!($($arg)*));
    };
}

/// Prints to the print RTT channel, with a newline. Works just like the standard `println`.
///
/// Before use the print channel has to be set with [`rtt_init_print`] or [`set_print_channel`]. If
/// the channel isn't set, the message is silently discarded.
///
/// The macro also supports output to multiple virtual terminals on the channel. Use the syntax
/// ```rprintln!(=> 1, "Hello!");``` to write to terminal number 1, for example. Terminal numbers
/// range from 0 to 15.
#[macro_export]
macro_rules! rprintln {
    (=> $terminal:expr) => {
        $crate::print_impl::write_str($terminal, "\n");
    };
    (=> $terminal:expr, $fmt:expr) => {
        $crate::print_impl::write_str($terminal, concat!($fmt, "\n"));
    };
    (=> $terminal:expr, $fmt:expr, $($arg:tt)*) => {
        $crate::print_impl::write_fmt($terminal, format_args!(concat!($fmt, "\n"), $($arg)*));
    };
    () => {
        $crate::print_impl::write_str(0, "\n");
    };
    ($fmt:expr) => {
        $crate::print_impl::write_str(0, concat!($fmt, "\n"));
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::print_impl::write_fmt(0, format_args!(concat!($fmt, "\n"), $($arg)*));
    };
}

/// Initializes RTT with a single up channel and sets it as the print channel for the printing
/// macros.
///
/// The optional arguments specify the blocking mode (default: `NoBlockSkip`) and size of the buffer
/// in bytes (default: 1024). See [`rtt_init`] for more details.
#[macro_export]
macro_rules! rtt_init_print {
    ($mode:ident, $size:literal) => {
        let channels = $crate::rtt_init! {
            up: {
                0: {
                    size: $size
                    mode: $mode
                    name: "Terminal"
                }
            }
        };

        $crate::set_print_channel(channels.up.0);
    };

    ($mode:ident) => {
        $crate::rtt_init_print!($mode, 1024);
    };

    () => {
        $crate::rtt_init_print!(NoBlockSkip, 1024);
    };
}

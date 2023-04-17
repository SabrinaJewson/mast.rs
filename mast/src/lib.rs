//! Mast is a flexible build and caching system configured by Rust code.
//!
//! # Non-features
//!
//! - Automatic cleaning
#![warn(
    noop_method_call,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    missing_docs,
    missing_debug_implementations,
    clippy::pedantic
)]
#![no_std]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod asset;
pub mod etag;

macro_rules! for_tuples {
    ($macro:ident) => {
        $macro!(Tuple0:                         );
        $macro!(Tuple1:             A           );
        $macro!(Tuple2:            A B          );
        $macro!(Tuple3:           A B C         );
        $macro!(Tuple4:          A B C D        );
        $macro!(Tuple5:         A B C D E       );
        $macro!(Tuple6:        A B C D E F      );
        $macro!(Tuple7:       A B C D E F G     );
        $macro!(Tuple8:      A B C D E F G H    );
        $macro!(Tuple9:     A B C D E F G H I   );
        $macro!(Tuple10:   A B C D E F G H I J  );
        $macro!(Tuple11:  A B C D E F G H I J K );
        $macro!(Tuple12: A B C D E F G H I J K L);
    };
}
use for_tuples;

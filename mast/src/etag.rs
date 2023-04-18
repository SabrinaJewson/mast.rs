//! The [`Etag`] trait,
//! representing a fingerprint of a value.

/// An `Etag` represents a fingerprint of a value,
/// such that identical etags imply identical full data
/// (but identical full data does not necessarily imply identical etags).
///
/// # Invariants
///
/// Implementations of this trait are assumed to uphold the following invariants:
///
/// - The serialized forms of etags must be byte-for-byte equal
///     only if the data the etags represent is equal.
/// - If the etags implement [`PartialEq`], two etags must compare equal
///     if and only if their serialized forms are byte-for-byte equal.
/// - Serializing and then deserializing an etag
///     must successfully result in the same etag.
///     - Note that the converse is not always true:
///         deserializing and then serializing a byte sequence as the etag
///         does not necessarily produce the same byte sequence.
/// - Serialization must produce an architecture-independent format.
/// - Deserialization must not replace the [`Reader`] with a different one.
/// - The exact bytes produced by serialization are not considered to be part of the public API,
///     but it *is* a breaking change if old data previously returned by `serialize`
///     now deserializes to a different thing or fails to deserialize.
///     It is *not* a breaking change to allow data that previously failed to deserialize
///     to successfully deserialize,
///     or to disallow data that previously successfully deserialized
///     but could not have been returned from `serialize`.
pub trait Etag: 'static + Sized + Debug + Default {
    /// Serialize the etag into its architecture-independent binary format.
    fn serialize<W: ?Sized + Writer>(&self, writer: &mut W);

    /// Serialize the etag to a `Vec<u8>`.
    #[cfg(feature = "alloc")]
    #[cfg_attr(doc_nightly, doc(cfg(feature = "alloc")))]
    fn to_vec(&self) -> alloc::vec::Vec<u8> {
        let mut vec = alloc::vec::Vec::new();
        self.serialize(&mut vec);
        vec
    }

    /// Attempt to deserialize this etag from its binary format.
    ///
    /// # Errors
    ///
    /// Fails if the input format is invalid.
    /// It is not specified how many bytes the `reader` has consumed.
    fn deserialize(reader: &mut Reader<'_>) -> Result<Self, DeserializeError>;

    /// Attempt to deserialize this etag from bytes.
    ///
    /// # Errors
    ///
    /// Fails if the input format is invalid,
    /// or [`Self::deserialize`] does not consume all the bytes.
    fn from_bytes(bytes: &[u8]) -> Result<Self, FromBytesError> {
        let mut reader = Reader::new(bytes);
        let deserialized = Self::deserialize(&mut reader).map_err(FromBytesError::Deserialize)?;
        let trailing = reader.remaining().len();
        if let Some(number) = NonZeroU32::new(trailing.try_into().unwrap_or(u32::MAX)) {
            return Err(FromBytesError::TrailingBytes(TrailingBytes { number }));
        }
        Ok(deserialized)
    }
}

mod deserialize_error {
    /// An error in [`Etag::deserialize`](super::Etag::deserialize).
    ///
    /// This type is intentionally opaque
    /// because the binary etag format is not human-writeable,
    /// and so detailed error messages are unlikely to be helpful.
    #[derive(Clone)]
    pub struct DeserializeError {
        // Use this instead of `non_exhaustive` to disallow construction within this crate.
        _private: (),
    }

    impl DeserializeError {
        /// Construct a nonspecific error.
        #[must_use]
        pub const fn nonspecific() -> Self {
            Self { _private: () }
        }
    }

    impl Debug for DeserializeError {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            f.pad("DeserializeError")
        }
    }

    impl Display for DeserializeError {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            f.write_str("invalid etag")
        }
    }

    #[cfg(feature = "std")]
    #[cfg_attr(doc_nightly, doc(cfg(feature = "std")))]
    impl std::error::Error for DeserializeError {}

    use core::fmt;
    use core::fmt::Debug;
    use core::fmt::Display;
    use core::fmt::Formatter;
}
pub use deserialize_error::DeserializeError;

/// An error in [`Etag::from_bytes`].
#[derive(Debug, Clone)]
pub enum FromBytesError {
    /// An error deserializing the etag.
    Deserialize(DeserializeError),

    /// The call to [`Etag::deserialize`] didn’t consume all the input.
    TrailingBytes(TrailingBytes),
}

impl Display for FromBytesError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("failed to deserialize etag from bytes")
    }
}

#[cfg(feature = "std")]
#[cfg_attr(doc_nightly, doc(cfg(feature = "std")))]
impl std::error::Error for FromBytesError {
    fn source(&self) -> Option<&(dyn 'static + std::error::Error)> {
        match self {
            Self::Deserialize(e) => Some(e),
            Self::TrailingBytes(e) => Some(e),
        }
    }
}

/// The call to [`Etag::deserialize`] didn’t consume all the input.
#[derive(Debug, Clone)]
pub struct TrailingBytes {
    number: NonZeroU32,
}

impl Display for TrailingBytes {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.number.get() {
            u32::MAX => f.write_str("over 4 billion trailing bytes found in input"),
            1 => write!(f, "1 trailing byte found in input"),
            n => write!(f, "{n} trailing bytes found in input"),
        }
    }
}

#[cfg(feature = "std")]
#[cfg_attr(doc_nightly, doc(cfg(feature = "std")))]
impl std::error::Error for TrailingBytes {}

macro_rules! impl_for_tuple {
    ($name:ident: $($t:ident)*) => {
        impl<$($t: Etag,)*> Etag for ($($t,)*) {
            #[allow(non_snake_case)]
            fn serialize<W: ?Sized + Writer>(&self, _writer: &mut W) {
                let ($($t,)*) = self;
                $($t.serialize(_writer);)*
            }
            fn deserialize(_reader: &mut Reader<'_>) -> Result<Self, DeserializeError> {
                Ok(($($t::deserialize(_reader)?,)*))
            }
        }
    }
}
crate::for_tuples!(impl_for_tuple);

/// A sink of bytes to serialize into.
///
/// # Varint encoding
///
/// This type offers various `_var` methods for writing variable-width integers,
/// which saves space for the common case of small integers.
/// The encoding used for unsigned integers is given below:
///
/// ```text
///            Prefix │ Total Bytes │ Bits Encoded │ Endianness │ Types Used │
///          1xxxxxxx │           1 │            7 │ N/A        │        All │
///          01xxxxxx │           2 │           14 │ big        │        All │
///          001xxxxx │           3 │           21 │ big        │     >= u32 │
///          0001xxxx │           4 │           28 │ big        │     >= u32 │
///          00001xxx │           5 │           35 │ big        │     >= u64 │
///          000001xx │           6 │           42 │ big        │     >= u64 │
///          0000001x │           7 │           49 │ big        │     >= u64 │
///          00000001 │           8 │           56 │ big        │     >= u64 │
/// 00000000 1xxxxxxx │           9 │           63 │ big        │       u128 │
/// 00000000 01xxxxxx │          10 │           70 │ big        │       u128 │
/// 00000000 001xxxxx │          11 │           77 │ big        │       u128 │
/// 00000000 0001xxxx │          12 │           84 │ big        │       u128 │
/// 00000000 00001xxx │          13 │           91 │ big        │       u128 │
/// 00000000 000001xx │          14 │           98 │ big        │       u128 │
/// 00000000 0000001x │          15 │          105 │ big        │       u128 │
/// 00000000 00000001 │          16 │          112 │ big        │       u128 │
///          00000000 │ 1+size of T │      T::BITS │ little     │   not u128 │
/// 00000000 00000000 │          18 │   u128::BITS │ little     │       u128 │
/// ```
///
/// That is, the number of leading zeros is one less than the total number of bytes,
/// and the special leading pattern of all-zeros encodes the entire integer
/// directly in little-endian.
///
/// For signed integers,
/// we first transform it to be unsigned
/// using the [zigzag algorithm]
/// and then encode them as unsigned.
///
/// [zigzag algorithm]: https://protobuf.dev/programming-guides/encoding/#types
pub trait Writer {
    /// Write some bytes to the sink.
    ///
    /// Note that this does **not** prefix the slice with a length.
    fn write_bytes(&mut self, bytes: &[u8]);

    /// Whether the implementation should use variable-int encoding;
    /// if this is `false`,
    /// all `_var` methods will simply forward to their non-`_var` counterpart.
    ///
    /// This is mostly useful for `Writer`s that are actually hashers,
    /// for which it’s nor really worth compacting the integers.
    fn use_varint(&self) -> bool {
        true
    }

    prim_writer_methods! {
        write_u16 write_u16_var(encode_unsigned) u16,
        write_u32 write_u32_var(encode_unsigned) u32,
        write_u64 write_u64_var(encode_unsigned) u64,
        write_u128 write_u128_var(encode_unsigned) u128,
        write_i16 write_i16_var(encode_signed) i16,
        write_i32 write_i32_var(encode_signed) i32,
        write_i64 write_i64_var(encode_signed) i64,
        write_i128 write_i128_var(encode_signed) i128,
        // TODO: usize/isize
    }
}

macro_rules! prim_writer_methods {
    ($($write_type:ident $write_type_var:ident($encode:ident) $type:ident,)*) => { $(
        #[doc = concat!("Write a fixed-width `", stringify!($type), "` in little-endian.")]
        fn $write_type(&mut self, value: $type) {
            self.write_bytes(&value.to_le_bytes());
        }
        #[doc = concat!("Write a `", stringify!($type), "` using a variable-width encoding.")]
        ///
        /// See the docs of [`Writer`] for details on how this works.
        fn $write_type_var(&mut self, value: $type) {
            if self.use_varint() {
                varint::$encode(self, value);
            } else {
                self.$write_type(value);
            }
        }
    )* }
}
use prim_writer_methods;

impl<W: Writer> Writer for &mut W {
    fn write_bytes(&mut self, bytes: &[u8]) {
        (**self).write_bytes(bytes);
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(doc_nightly, doc(cfg(feature = "alloc")))]
impl Writer for alloc::vec::Vec<u8> {
    fn write_bytes(&mut self, bytes: &[u8]) {
        self.extend_from_slice(bytes);
    }
}

/// A cursor around an in-memory buffer to deserialize from.
///
/// # Errors
///
/// Methods on this type that return a [`DeserializeError`]
/// give the guarantee that failure will *not* advance the `Reader`.
#[derive(Debug)]
pub struct Reader<'buf> {
    buf: &'buf [u8],
}

#[allow(clippy::missing_errors_doc)]
impl<'buf> Reader<'buf> {
    /// Construct a new `Reader` from the given byte slice.
    #[must_use]
    pub const fn new(buf: &'buf [u8]) -> Self {
        Self { buf }
    }

    /// Obtain all the remaining bytes in the reader as a byte slice.
    #[must_use]
    pub const fn remaining(&self) -> &'buf [u8] {
        self.buf
    }

    /// Consume `n` bytes from the reader,
    /// such that they will no longer be returned from calls to [`Self::remaining`].
    pub fn consume(&mut self, n: usize) {
        self.buf = &self.buf[n..];
    }

    /// Peek from the reader.
    /// Semantically the same as a clone.
    #[must_use]
    pub const fn peek(&self) -> Self {
        Self::new(self.remaining())
    }

    /// Read a slice of bytes from the reader.
    pub fn read_bytes(&mut self, n: usize) -> Result<&'buf [u8], DeserializeError> {
        let buf = self
            .remaining()
            .get(..n)
            .ok_or_else(DeserializeError::nonspecific)?;
        self.consume(n);
        Ok(buf)
    }

    /// Read an array of bytes from the reader.
    pub fn read_array<const N: usize>(&mut self) -> Result<[u8; N], DeserializeError> {
        let msg = "`read_bytes` should read the correct number of bytes";
        Ok(self.read_bytes(N)?.try_into().expect(msg))
    }

    /// Fill the given buffer with bytes from the reader.
    pub fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), DeserializeError> {
        buf.copy_from_slice(self.read_bytes(buf.len())?);
        Ok(())
    }

    /// Read and consume a single byte from the reader.
    pub fn read_u8(&mut self) -> Result<u8, DeserializeError> {
        self.read_array().map(|[byte]| byte)
    }

    prim_reader_methods! {
        read_u16 read_u16_var(decode_unsigned) u16,
        read_u32 read_u32_var(decode_unsigned) u32,
        read_u64 read_u64_var(decode_unsigned) u64,
        read_u128 read_u128_var(decode_unsigned) u128,
        read_i16 read_i16_var(decode_signed) i16,
        read_i32 read_i32_var(decode_signed) i32,
        read_i64 read_i64_var(decode_signed) i64,
        read_i128 read_i128_var(decode_signed) i128,
        // TODO: usize/isize
    }
}

macro_rules! prim_reader_methods {
    ($($read_type:ident $read_type_var:ident($decode:ident) $type:ident,)*) => { $(
        #[doc = concat!("Read a fixed-width `", stringify!($type), "` in little-endian.")]
        pub fn $read_type(&mut self) -> Result<$type, DeserializeError> {
            self.read_array().map($type::from_le_bytes)
        }
        #[doc = concat!("Read a `", stringify!($type), "` using a variable-width encoding.")]
        ///
        /// See the docs of [`Writer`] for details on how this works.
        pub fn $read_type_var(&mut self) -> Result<$type, DeserializeError> {
            varint::$decode(self)
        }
    )* }
}
use prim_reader_methods;

mod varint {
    pub(crate) fn encode_unsigned<W: ?Sized + Writer, T: Unsigned>(writer: &mut W, value: T) {
        for total_bytes in 1..=mem::size_of::<T>() {
            let leading_zeros = total_bytes - 1;
            if value < (T::ONE << (total_bytes * 7)) {
                let mut be = value.to_be_bytes();
                let slice = &mut be.as_mut()[mem::size_of::<T>() - total_bytes..];
                slice[leading_zeros / 8] |= 0b1000_0000 >> (leading_zeros % 8);
                writer.write_bytes(slice);
                return;
            }
        }
        if mem::size_of::<u64>() < mem::size_of::<T>() {
            writer.write_bytes(&[0, 0]);
        } else {
            writer.write_bytes(&[0]);
        }
        writer.write_bytes(value.to_le_bytes().as_ref());
    }

    pub(crate) fn encode_signed<W: ?Sized + Writer, T: Signed>(writer: &mut W, value: T) {
        encode_unsigned(writer, zigzag::encode(value));
    }

    pub(crate) fn decode_unsigned<T: Unsigned>(
        reader: &mut Reader<'_>,
    ) -> Result<T, DeserializeError> {
        let first_byte = reader.peek().read_u8()?;
        let first_byte_leading = first_byte.leading_zeros() as usize;
        let (leading_zeros, initial) =
            if mem::size_of::<u64>() < mem::size_of::<T>() && first_byte_leading == 8 {
                let [_, second_byte] = reader.peek().read_array()?;
                (8 + second_byte.leading_zeros() as usize, 2)
            } else {
                (first_byte_leading, 1)
            };
        let total_bytes = leading_zeros + 1;

        let mut bytes = T::Bytes::default();
        let res = if let Some(first_byte_index) = mem::size_of::<T>().checked_sub(total_bytes) {
            reader.read_exact(&mut bytes.as_mut()[first_byte_index..])?;
            bytes.as_mut()[first_byte_index + leading_zeros / 8] &=
                0b0111_1111 >> (leading_zeros % 8);
            T::from_be_bytes(bytes)
        } else if leading_zeros % 8 == 0 {
            reader.consume(initial);
            reader.read_exact(bytes.as_mut())?;
            T::from_le_bytes(bytes)
        } else {
            return Err(DeserializeError::nonspecific());
        };
        Ok(res)
    }

    pub(crate) fn decode_signed<T: Signed>(reader: &mut Reader<'_>) -> Result<T, DeserializeError> {
        decode_unsigned::<T::Unsigned>(reader).map(zigzag::decode)
    }

    #[cfg(all(test, feature = "alloc"))]
    mod tests {
        #[test]
        fn unsigned() {
            check_unsigned(0_u16, &[0b1000_0000]);
            check_unsigned(37_u16, &[0b1000_0000 + 37]);
            check_unsigned(127_u16, &[255]);
            check_unsigned(0_u64, &[0b1000_0000]);
            check_unsigned(37_u64, &[0b1000_0000 + 37]);
            check_unsigned(127_u64, &[255]);
            check_unsigned(128_u16, &[0b0100_0000, 0b1000_0000]);
            check_unsigned(16_383_u16, &[0b0111_1111, 0b1111_1111]);
            check_unsigned(16_383_u32, &[0b0111_1111, 0b1111_1111]);
            check_unsigned(16_384_u16, &[0b0000_0000, 0b0000_0000, 0b0100_0000]);
            check_unsigned(16_384_u32, &[0b0010_0000, 0b0100_0000, 0b0000_0000]);
            check_fail::<u16>(&[0b0010_0000, 0b0100_0000, 0b0000_0000]);
            check_unsigned(2_u32.pow(28), b"\x00\x00\x00\x00\x10");
            check_unsigned(2_u64.pow(28), b"\x08\x10\x00\x00\x00");
            check_fail::<u32>(b"\x08\x08\x00\x00\x00");
            check_unsigned(2_u64.pow(56) - 1, b"\x01\xFF\xFF\xFF\xFF\xFF\xFF\xFF");
            check_unsigned(2_u64.pow(56), b"\x00\x00\x00\x00\x00\x00\x00\x00\x01");
            check_unsigned(2_u128.pow(56) - 1, b"\x01\xFF\xFF\xFF\xFF\xFF\xFF\xFF");
            check_unsigned(2_u128.pow(56), b"\x00\x81\x00\x00\x00\x00\x00\x00\x00");
            check_unsigned(
                2_u128.pow(103),
                b"\x00\x02\x80\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00",
            );
            check_unsigned(u16::MAX, b"\x00\xFF\xFF");
            check_unsigned(u32::MAX, b"\x00\xFF\xFF\xFF\xFF");
            check_unsigned(u64::MAX, b"\x00\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF");
            check_unsigned(
                u128::MAX,
                b"\x00\x00\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF",
            );
        }

        #[test]
        fn fail() {
            check_fail::<u8>(&[]);
            check_fail::<u128>(&[]);
            check_fail::<u16>(&[0b0100_0000]);
        }

        #[track_caller]
        fn check_unsigned<T: Unsigned>(value: T, encoded: &[u8]) {
            let mut actual_encoded = Vec::new();
            super::encode_unsigned(&mut actual_encoded, value);
            assert_eq!(actual_encoded, encoded, "encoding is incorrect");

            let mut reader = Reader::new(encoded);
            let actual_value = super::decode_unsigned(&mut reader).unwrap();
            assert_eq!(reader.remaining(), &[], "reader did not finish");
            assert_eq!(value, actual_value, "decoding is incorrect");
        }

        #[track_caller]
        fn check_fail<T: Unsigned>(encoded: &[u8]) {
            let mut reader = Reader::new(encoded);
            super::decode_unsigned::<T>(&mut reader).unwrap_err();
            assert_eq!(reader.remaining(), encoded, "reader advanced");
        }

        use super::super::num::Unsigned;
        use super::super::Reader;
        use alloc::vec::Vec;
    }

    use super::num::Signed;
    use super::num::Unsigned;
    use super::zigzag;
    use super::DeserializeError;
    use super::Reader;
    use super::Writer;
    use core::mem;
}

/// “zigzag” encoding of signed integers
mod zigzag {
    pub(crate) fn encode<T: Signed>(n: T) -> T::Unsigned {
        ((n >> (T::BITS - 1)) ^ (n << 1)).cast_unsigned()
    }

    pub(crate) fn decode<T: Unsigned>(n: T) -> T::Signed {
        (n >> 1).cast_signed() ^ -(n & T::ONE).cast_signed()
    }

    #[cfg(test)]
    mod tests {
        #[allow(clippy::identity_op)]
        const EXPECTATIONS: [(i32, u32); 8] = [
            (0 + 0, 0),
            (0 - 1, 1),
            (0 + 1, 2),
            (0 - 2, 3),
            (0 + 2, 4),
            (0 - 3, 5),
            (i32::MAX, u32::MAX - 1),
            (i32::MIN, u32::MAX),
        ];

        #[test]
        fn works() {
            for (int, encoded) in EXPECTATIONS {
                assert_eq!(encode(int), encoded);
                assert_eq!(decode(encoded), int);
            }
        }

        use super::decode;
        use super::encode;
    }

    use super::num::Signed;
    use super::num::Unsigned;
}

mod num {
    pub(crate) trait Int:
        'static
        + Sized
        + Debug
        + Copy
        + Ord
        + BitXor<Output = Self>
        + BitAnd<Output = Self>
        + Shl<usize, Output = Self>
        + Shr<u32, Output = Self>
    {
        const ZERO: Self;
        const ONE: Self;
        const BITS: u32;
        type Bytes: Default + AsRef<[u8]> + AsMut<[u8]>;
        fn to_le_bytes(self) -> Self::Bytes;
        fn to_be_bytes(self) -> Self::Bytes;
        fn from_le_bytes(bytes: Self::Bytes) -> Self;
        fn from_be_bytes(bytes: Self::Bytes) -> Self;
        fn wrapping_add(self, other: Self) -> Self;
        fn wrapping_sub(self, other: Self) -> Self;
        fn leading_zeros(self) -> u32;
    }

    pub(crate) trait Unsigned: Int {
        type Signed: Signed<Bytes = Self::Bytes, Unsigned = Self>;
        fn cast_signed(self) -> Self::Signed;
    }

    pub(crate) trait Signed: Int + Neg<Output = Self> {
        type Unsigned: Unsigned<Bytes = Self::Bytes, Signed = Self>;
        fn cast_unsigned(self) -> Self::Unsigned;
    }

    macro_rules! impl_int {
        ($t:ident) => {
            impl Int for $t {
                const ZERO: Self = 0;
                const ONE: Self = 1;
                const BITS: u32 = Self::BITS;
                type Bytes = [u8; mem::size_of::<Self>()];
                fn to_le_bytes(self) -> Self::Bytes {
                    self.to_le_bytes()
                }
                fn to_be_bytes(self) -> Self::Bytes {
                    self.to_be_bytes()
                }
                fn from_le_bytes(bytes: Self::Bytes) -> Self {
                    Self::from_le_bytes(bytes)
                }
                fn from_be_bytes(bytes: Self::Bytes) -> Self {
                    Self::from_be_bytes(bytes)
                }
                fn wrapping_add(self, other: Self) -> Self {
                    self.wrapping_add(other)
                }
                fn wrapping_sub(self, other: Self) -> Self {
                    self.wrapping_sub(other)
                }
                fn leading_zeros(self) -> u32 {
                    self.leading_zeros()
                }
            }
        };
    }
    macro_rules! impl_signed {
        ($($i:ident $u:ident,)*) => { $(
            impl Signed for $i {
                type Unsigned = $u;
                fn cast_unsigned(self) -> Self::Unsigned {
                    self as $u
                }
            }
            impl Unsigned for $u {
                type Signed = $i;
                fn cast_signed(self) -> Self::Signed {
                    self as $i
                }
            }
            impl_int!($i);
            impl_int!($u);
        )* };
    }
    impl_signed! {
        i8 u8,
        i16 u16,
        i32 u32,
        i64 u64,
        i128 u128,
    }

    use core::fmt::Debug;
    use core::mem;
    use core::ops::BitAnd;
    use core::ops::BitXor;
    use core::ops::Neg;
    use core::ops::Shl;
    use core::ops::Shr;
}

use core::fmt;
use core::fmt::Debug;
use core::fmt::Display;
use core::fmt::Formatter;
use core::num::NonZeroU32;

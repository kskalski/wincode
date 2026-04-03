/// Standalone decode functions using [`DefaultConfig`](crate::config::DefaultConfig).
///
/// Two families are provided:
///
/// - **`decode` / `decode_into` / `decode_exact`** — the type is both the schema and the
///   destination (`T::Dst = T`). Rust can infer `T` from a type annotation, so no turbofish
///   is needed in the common case.
///
/// - **`decode_via` / `decode_into_via` / `decode_exact_via`** — the schema `S` and the
///   destination `S::Dst` may differ. Use these when you want to drive decoding through a
///   specialized container schema (e.g. `containers::Vec`). A turbofish is always required
///   because `S` cannot be inferred from `S::Dst` alone.
use {
    crate::{
        config::DefaultConfig,
        error::{self, ReadResult},
        io::Reader,
        schema::SchemaRead,
    },
    core::mem::MaybeUninit,
};

// ── inference-friendly variants (T == T::Dst) ────────────────────────────────

/// Decode a value of type `T` from `reader`.
///
/// `T` acts as its own schema (`T::Dst = T`), so Rust can infer `T` from a type
/// annotation without a turbofish.
///
/// For decoding through a schema whose destination differs from the schema type
/// itself, use [`decode_via`].
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "alloc")] {
/// use wincode::{decode, encode};
///
/// let bytes = encode::encode(&42u64).unwrap();
/// let value: u64 = decode::decode(&bytes[..]).unwrap();
/// assert_eq!(value, 42);
/// # }
/// ```
#[inline(always)]
pub fn decode<'de, T>(reader: impl Reader<'de>) -> ReadResult<T>
where
    T: SchemaRead<'de, DefaultConfig, Dst = T>,
{
    T::get(reader)
}

/// Decode into an existing `MaybeUninit<T>` slot from `reader`.
///
/// `T` acts as its own schema. For the schema-explicit variant see [`decode_into_via`].
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "alloc")] {
/// use core::mem::MaybeUninit;
/// use wincode::{decode, encode};
///
/// let bytes = encode::encode(&7u64).unwrap();
/// let mut dst = MaybeUninit::<u64>::uninit();
/// decode::decode_into(&bytes[..], &mut dst).unwrap();
/// assert_eq!(unsafe { dst.assume_init() }, 7);
/// # }
/// ```
#[inline(always)]
pub fn decode_into<'de, T>(reader: impl Reader<'de>, dst: &mut MaybeUninit<T>) -> ReadResult<()>
where
    T: SchemaRead<'de, DefaultConfig, Dst = T>,
{
    T::read(reader, dst)
}

/// Decode a value of type `T` from a byte slice, returning an error if any trailing
/// bytes remain.
///
/// `T` acts as its own schema. For the schema-explicit variant see [`decode_exact_via`].
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "alloc")] {
/// use wincode::{decode, encode};
///
/// let bytes = encode::encode(&123u64).unwrap();
/// assert_eq!(decode::decode_exact::<u64>(&bytes).unwrap(), 123);
///
/// let mut extra = bytes.clone();
/// extra.push(0xAA);
/// assert!(decode::decode_exact::<u64>(&extra).is_err());
/// # }
/// ```
#[inline(always)]
pub fn decode_exact<'de, T>(mut src: &'de [u8]) -> ReadResult<T>
where
    T: SchemaRead<'de, DefaultConfig, Dst = T>,
{
    let value = T::get(src.by_ref())?;
    if src.is_empty() {
        Ok(value)
    } else {
        Err(error::trailing_bytes())
    }
}

// ── schema-explicit variants (S::Dst may differ from S) ──────────────────────

/// Decode from `reader` using schema `S`, returning `S::Dst`.
///
/// Use this when the schema type and the destination type differ — for example
/// when decoding through a [`containers`](crate::containers) adapter.
/// A turbofish is always required because `S` cannot be inferred from `S::Dst` alone.
///
/// For the common case where `T` is its own schema, prefer [`decode`].
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "alloc")] {
/// use wincode::{decode, encode, containers, len::BincodeLen};
///
/// let original: Vec<u8> = vec![10, 20, 30];
/// let bytes = encode::encode(&original).unwrap();
///
/// type VecSchema = containers::Vec<u8, BincodeLen>;
/// let decoded: Vec<u8> = decode::decode_via::<VecSchema>(&bytes[..]).unwrap();
/// assert_eq!(decoded, original);
/// # }
/// ```
#[inline(always)]
pub fn decode_via<'de, S>(reader: impl Reader<'de>) -> ReadResult<S::Dst>
where
    S: SchemaRead<'de, DefaultConfig>,
{
    S::get(reader)
}

/// Decode from `reader` into an existing `MaybeUninit<S::Dst>` slot using schema `S`.
///
/// For the common case where `T` is its own schema, prefer [`decode_into`].
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "alloc")] {
/// use core::mem::MaybeUninit;
/// use wincode::{decode, encode, containers, len::BincodeLen};
///
/// let original: Vec<u8> = vec![1, 2, 3];
/// let bytes = encode::encode(&original).unwrap();
///
/// type VecSchema = containers::Vec<u8, BincodeLen>;
/// let mut dst = MaybeUninit::<Vec<u8>>::uninit();
/// decode::decode_into_via::<VecSchema>(&bytes[..], &mut dst).unwrap();
/// assert_eq!(unsafe { dst.assume_init() }, original);
/// # }
/// ```
#[inline(always)]
pub fn decode_into_via<'de, S>(
    reader: impl Reader<'de>,
    dst: &mut MaybeUninit<S::Dst>,
) -> ReadResult<()>
where
    S: SchemaRead<'de, DefaultConfig>,
{
    S::read(reader, dst)
}

/// Decode from a byte slice using schema `S`, returning an error if any trailing bytes remain.
///
/// For the common case where `T` is its own schema, prefer [`decode_exact`].
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "alloc")] {
/// use wincode::{decode, encode, containers, len::BincodeLen};
///
/// let original: Vec<u8> = vec![1, 2, 3];
/// let bytes = encode::encode(&original).unwrap();
///
/// type VecSchema = containers::Vec<u8, BincodeLen>;
/// let decoded: Vec<u8> = decode::decode_exact_via::<VecSchema>(&bytes).unwrap();
/// assert_eq!(decoded, original);
/// # }
/// ```
#[inline(always)]
pub fn decode_exact_via<'de, S>(mut src: &'de [u8]) -> ReadResult<S::Dst>
where
    S: SchemaRead<'de, DefaultConfig>,
{
    let value = S::get(src.by_ref())?;
    if src.is_empty() {
        Ok(value)
    } else {
        Err(error::trailing_bytes())
    }
}

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use {
        super::*,
        crate::{containers, encode, len::BincodeLen},
        alloc::vec,
        alloc::vec::Vec,
    };

    /// `decode` infers `T` from the type annotation — no turbofish needed.
    #[test]
    fn decode_infers_type() {
        let bytes = encode::encode(&42u32).unwrap();
        let value: u32 = decode(&bytes[..]).unwrap();
        assert_eq!(value, 42);
    }

    /// `decode_via` uses a container schema whose Dst differs from the schema type.
    #[test]
    fn decode_via_explicit_container_schema() {
        let original: Vec<u8> = vec![10, 20, 30];
        let bytes = encode::encode(&original).unwrap();
        let decoded: Vec<u8> = decode_via::<containers::Vec<u8, BincodeLen>>(&bytes[..]).unwrap();
        assert_eq!(decoded, original);
    }

    /// `decode_into` writes directly into a `MaybeUninit` slot.
    #[test]
    fn decode_into_maybe_uninit() {
        let bytes = encode::encode(&0xDEAD_BEEF_u64).unwrap();
        let mut dst = MaybeUninit::<u64>::uninit();
        decode_into(&bytes[..], &mut dst).unwrap();
        assert_eq!(unsafe { dst.assume_init() }, 0xDEAD_BEEF_u64);
    }

    /// `decode_exact` rejects trailing bytes.
    #[test]
    fn decode_exact_rejects_trailing_bytes() {
        let bytes = encode::encode(&42u32).unwrap();
        assert_eq!(decode_exact::<u32>(&bytes).unwrap(), 42u32);
        let mut extra = bytes.clone();
        extra.push(0xFF);
        assert!(decode_exact::<u32>(&extra).is_err());
    }

    /// Any `impl Reader<'de>` is accepted, not just `&[u8]`.
    #[cfg(feature = "std")]
    #[test]
    fn decode_from_cursor_reader() {
        use crate::io::Cursor;
        let bytes = encode::encode(&999u32).unwrap();
        let value: u32 = decode(Cursor::new(bytes)).unwrap();
        assert_eq!(value, 999);
    }

    /// `decode` with `&mut [u8]` subsumes `deserialize_mut`: no separate `decode_mut` needed.
    /// Passing a mutable slice reader lets zero-copy schemas yield mutable references directly
    /// into the backing buffer.
    #[cfg(feature = "derive")]
    #[test]
    fn decode_mut_slice_subsumes_deserialize_mut() {
        #[derive(crate::SchemaWrite, crate::SchemaRead, Debug, PartialEq, Eq, Clone, Copy)]
        #[wincode(internal)]
        #[repr(C)]
        struct Frame {
            data: [u8; 4],
            tag: u8,
            _pad: [u8; 3],
        }

        let original = Frame { data: [1, 2, 3, 4], tag: 7, _pad: [0; 3] };
        let mut serialized = encode::encode(&original).unwrap();

        // &mut [u8] is just another Reader — the schema yields &mut Frame directly
        // into the serialized buffer, no decode_mut variant required.
        let view: &mut Frame = decode(&mut serialized[..]).unwrap();
        view.data = [10, 20, 30, 40];
        view.tag = 99;

        let result: Frame = decode(&serialized[..]).unwrap();
        assert_eq!(result, Frame { data: [10, 20, 30, 40], tag: 99, _pad: [0; 3] });
    }
}

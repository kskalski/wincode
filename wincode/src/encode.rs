use crate::{config::DefaultConfig, error::WriteResult, io::Writer, schema::SchemaWrite};
/// Standalone encode functions using [`DefaultConfig`](crate::config::DefaultConfig).
///
/// Two families are provided:
///
/// - **`encode` / `encode_into` / `encoded_size`** — the type is both the schema and the
///   source (`T::Src = T`). Rust can infer `T` from the argument type, so no turbofish
///   is needed in the common case.
///
/// - **`encode_via` / `encode_into_via` / `encoded_size_via`** — the schema `S` and the
///   source `S::Src` may differ. Use these when you want to drive encoding through a
///   specialized container schema (e.g. `containers::Vec`). A turbofish is always required
///   because `S` cannot be inferred from the source value alone.
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

// ── inference-friendly variants (T == T::Src) ────────────────────────────────

/// Encode `src` into a newly allocated `Vec<u8>`.
///
/// `T` acts as its own schema (`T::Src = T`), so Rust can infer `T` from the
/// argument type without a turbofish.
///
/// For encoding through a schema whose source type differs, use [`encode_via`].
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "alloc")] {
/// use wincode::encode;
///
/// let bytes = encode::encode(&42u64).unwrap();
/// assert_eq!(bytes.len(), 8);
/// # }
/// ```
#[cfg(feature = "alloc")]
#[inline]
pub fn encode<T>(src: &T) -> WriteResult<Vec<u8>>
where
    T: SchemaWrite<DefaultConfig, Src = T> + ?Sized,
{
    let capacity = T::size_of(src)?;
    let mut buffer = Vec::with_capacity(capacity);
    let mut writer = buffer.spare_capacity_mut();
    T::write(writer.by_ref(), src)?;
    let remaining = writer.len();
    unsafe {
        #[allow(clippy::arithmetic_side_effects)]
        buffer.set_len(capacity - remaining);
    }
    Ok(buffer)
}

/// Encode `src` into the provided `writer`.
///
/// `T` acts as its own schema. For the schema-explicit variant see [`encode_into_via`].
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "alloc")] {
/// use wincode::{encode, decode};
///
/// let mut buf = [0u8; 8];
/// encode::encode_into(&mut buf[..], &42u64).unwrap();
/// let value: u64 = decode::decode(&buf[..]).unwrap();
/// assert_eq!(value, 42);
/// # }
/// ```
#[inline]
pub fn encode_into<T>(mut writer: impl Writer, src: &T) -> WriteResult<()>
where
    T: SchemaWrite<DefaultConfig, Src = T> + ?Sized,
{
    T::write(writer.by_ref(), src)?;
    Ok(writer.finish()?)
}

/// Return the number of bytes that `src` would occupy when encoded.
///
/// `T` acts as its own schema. For the schema-explicit variant see [`encoded_size_via`].
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "alloc")] {
/// use wincode::encode;
///
/// let vec: Vec<u8> = vec![1, 2, 3];
/// // 8 bytes for the length prefix + 3 bytes of data.
/// assert_eq!(encode::encoded_size(&vec).unwrap(), 11);
/// # }
/// ```
#[inline(always)]
pub fn encoded_size<T>(src: &T) -> WriteResult<usize>
where
    T: SchemaWrite<DefaultConfig, Src = T> + ?Sized,
{
    T::size_of(src)
}

// ── schema-explicit variants (S::Src may differ from S) ──────────────────────

/// Encode `src` into a newly allocated `Vec<u8>` using schema `S`.
///
/// Use this when the schema type and the source type differ — for example when
/// encoding through a [`containers`](crate::containers) adapter.
/// A turbofish is always required.
///
/// For the common case where `T` is its own schema, prefer [`encode`].
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "alloc")] {
/// use wincode::{encode, decode, containers, len::BincodeLen};
///
/// let vec: Vec<u8> = vec![1, 2, 3];
/// type VecSchema = containers::Vec<u8, BincodeLen>;
/// let bytes = encode::encode_via::<VecSchema>(&vec).unwrap();
/// let decoded: Vec<u8> = decode::decode(&bytes[..]).unwrap();
/// assert_eq!(decoded, vec);
/// # }
/// ```
#[cfg(feature = "alloc")]
#[inline]
pub fn encode_via<S>(src: &S::Src) -> WriteResult<Vec<u8>>
where
    S: SchemaWrite<DefaultConfig>,
{
    let capacity = S::size_of(src)?;
    let mut buffer = Vec::with_capacity(capacity);
    let mut writer = buffer.spare_capacity_mut();
    S::write(writer.by_ref(), src)?;
    let remaining = writer.len();
    unsafe {
        #[allow(clippy::arithmetic_side_effects)]
        buffer.set_len(capacity - remaining);
    }
    Ok(buffer)
}

/// Encode `src` into the provided `writer` using schema `S`.
///
/// For the common case where `T` is its own schema, prefer [`encode_into`].
#[inline]
pub fn encode_into_via<S>(mut writer: impl Writer, src: &S::Src) -> WriteResult<()>
where
    S: SchemaWrite<DefaultConfig>,
{
    S::write(writer.by_ref(), src)?;
    Ok(writer.finish()?)
}

/// Return the number of bytes that `src` would occupy when encoded with schema `S`.
///
/// For the common case where `T` is its own schema, prefer [`encoded_size`].
#[inline(always)]
pub fn encoded_size_via<S>(src: &S::Src) -> WriteResult<usize>
where
    S: SchemaWrite<DefaultConfig>,
{
    S::size_of(src)
}

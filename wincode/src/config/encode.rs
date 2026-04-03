/// Configuration-aware encode functions.
///
/// These mirror [`wincode::encode`](crate::encode) but let you specify the configuration `C`
/// as an additional generic type parameter — no runtime config value required.
///
/// Two families are provided:
///
/// - **`encode` / `encode_into` / `encoded_size`** — `T` is both schema and source
///   (`T::Src = T`); Rust can infer `T` from the argument type.
///
/// - **`encode_via` / `encode_into_via` / `encoded_size_via`** — schema `S` and source
///   `S::Src` may differ; a turbofish is always required.
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use crate::{
    config::Config,
    error::WriteResult,
    io::Writer,
    schema::SchemaWrite,
};

// ── inference-friendly variants (T == T::Src) ────────────────────────────────

/// Encode `src` into a newly allocated `Vec<u8>` using configuration `C`.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "alloc")] {
/// use wincode::{config::{self, Configuration, encode as cencode}, len::FixIntLen};
///
/// type Cfg = Configuration<true, { config::DEFAULT_PREALLOCATION_SIZE_LIMIT }, FixIntLen<u32>>;
///
/// let bytes = cencode::encode::<u64, Cfg>(&42u64).unwrap();
/// assert_eq!(bytes.len(), 8);
/// # }
/// ```
#[cfg(feature = "alloc")]
#[inline]
pub fn encode<T, C: Config>(src: &T) -> WriteResult<Vec<u8>>
where
    T: SchemaWrite<C, Src = T> + ?Sized,
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

/// Encode `src` into the provided `writer` using configuration `C`.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "alloc")] {
/// use wincode::{config::{self, Configuration, encode as cencode, decode as cdecode}, len::FixIntLen};
///
/// type Cfg = Configuration<true, { config::DEFAULT_PREALLOCATION_SIZE_LIMIT }, FixIntLen<u32>>;
///
/// let mut buf = [0u8; 8];
/// cencode::encode_into::<u64, Cfg>(&mut buf[..], &42u64).unwrap();
/// let value: u64 = cdecode::decode::<u64, Cfg>(&buf[..]).unwrap();
/// assert_eq!(value, 42);
/// # }
/// ```
#[inline]
pub fn encode_into<T, C: Config>(mut writer: impl Writer, src: &T) -> WriteResult<()>
where
    T: SchemaWrite<C, Src = T> + ?Sized,
{
    T::write(writer.by_ref(), src)?;
    Ok(writer.finish()?)
}

/// Return the number of bytes that `src` would occupy when encoded with configuration `C`.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "alloc")] {
/// use wincode::{config::{self, Configuration, encode as cencode}, len::FixIntLen};
///
/// type Cfg = Configuration<true, { config::DEFAULT_PREALLOCATION_SIZE_LIMIT }, FixIntLen<u32>>;
///
/// let vec: Vec<u8> = vec![1, 2, 3];
/// // 4-byte FixIntLen<u32> length prefix + 3 bytes data = 7 bytes total.
/// assert_eq!(cencode::encoded_size::<Vec<u8>, Cfg>(&vec).unwrap(), 7);
/// # }
/// ```
#[inline(always)]
pub fn encoded_size<T, C: Config>(src: &T) -> WriteResult<usize>
where
    T: SchemaWrite<C, Src = T> + ?Sized,
{
    T::size_of(src)
}

// ── schema-explicit variants (S::Src may differ from S) ──────────────────────

/// Encode `src` into a newly allocated `Vec<u8>` using schema `S` and configuration `C`.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "alloc")] {
/// use wincode::{config::{self, Configuration, encode as cencode}, containers, len::FixIntLen};
///
/// type Cfg = Configuration<true, { config::DEFAULT_PREALLOCATION_SIZE_LIMIT }, FixIntLen<u32>>;
///
/// let vec: Vec<u8> = vec![1, 2, 3];
/// // 4-byte FixIntLen<u32> length prefix + 3 bytes data = 7 bytes total.
/// let bytes = cencode::encode_via::<containers::Vec<u8, FixIntLen<u32>>, Cfg>(&vec).unwrap();
/// assert_eq!(bytes.len(), 7);
/// # }
/// ```
#[cfg(feature = "alloc")]
#[inline]
pub fn encode_via<S, C: Config>(src: &S::Src) -> WriteResult<Vec<u8>>
where
    S: SchemaWrite<C>,
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

/// Encode `src` into the provided `writer` using schema `S` and configuration `C`.
#[inline]
pub fn encode_into_via<S, C: Config>(mut writer: impl Writer, src: &S::Src) -> WriteResult<()>
where
    S: SchemaWrite<C>,
{
    S::write(writer.by_ref(), src)?;
    Ok(writer.finish()?)
}

/// Return the number of bytes that `src` would occupy when encoded with schema `S`
/// and configuration `C`.
#[inline(always)]
pub fn encoded_size_via<S, C: Config>(src: &S::Src) -> WriteResult<usize>
where
    S: SchemaWrite<C>,
{
    S::size_of(src)
}

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use {
        super::*,
        crate::{
            config::{self, Configuration, decode as cdecode},
            len::FixIntLen,
        },
        alloc::{vec, vec::Vec},
    };

    type FixLenConfig =
        Configuration<true, { config::DEFAULT_PREALLOCATION_SIZE_LIMIT }, FixIntLen<u32>>;

    /// `encode` infers `T` from the argument — no turbofish needed.
    #[test]
    fn encode_infers_type() {
        let bytes = encode::<u64, FixLenConfig>(&42u64).unwrap();
        assert_eq!(bytes.len(), 8);
    }

    /// 4-byte `FixIntLen<u32>` prefix makes the encoding differ from the default.
    #[test]
    fn encode_with_explicit_config() {
        let vec: Vec<u8> = vec![1, 2, 3];
        let bytes = encode::<Vec<u8>, FixLenConfig>(&vec).unwrap();
        assert_eq!(bytes.len(), 7);
        let len = u32::from_le_bytes(bytes[..4].try_into().unwrap());
        assert_eq!(len as usize, vec.len());
    }

    #[test]
    fn encode_into_buffer() {
        let mut buf = [0u8; 8];
        encode_into::<u64, FixLenConfig>(&mut buf[..], &0xCAFEu64).unwrap();
        let value: u64 = cdecode::decode::<u64, FixLenConfig>(&buf[..]).unwrap();
        assert_eq!(value, 0xCAFEu64);
    }

    #[test]
    fn encoded_size_with_config() {
        let vec: Vec<u8> = vec![1, 2, 3];
        assert_eq!(encoded_size::<Vec<u8>, FixLenConfig>(&vec).unwrap(), 7);
    }
}

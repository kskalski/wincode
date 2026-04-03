/// Configuration-aware encode functions.
///
/// These mirror [`wincode::encode`](crate::encode) but accept a `config: C` value so that
/// both `T` and `C` are inferred at the call site — no turbofish required for the common case.
///
/// Two families are provided:
///
/// - **`encode` / `encode_into` / `encoded_size`** — `T` is both schema and source
///   (`T::Src = T`); fully inferred from the argument type.
///
/// - **`encode_via` / `encode_into_via` / `encoded_size_via`** — schema `S` and source
///   `S::Src` may differ; `S` must still be named but `C` is inferred from the config value.
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use crate::{
    config::Config,
    error::WriteResult,
    io::Writer,
    schema::SchemaWrite,
};

// ── inference-friendly variants (T == T::Src) ────────────────────────────────

/// Encode `src` into a newly allocated `Vec<u8>` using the given `config`.
///
/// Both `T` and `C` are inferred — `T` from `src`, `C` from `config`.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "alloc")] {
/// use wincode::{config::{self, Configuration, encode as cencode}, len::FixIntLen};
///
/// type Cfg = Configuration<true, { config::DEFAULT_PREALLOCATION_SIZE_LIMIT }, FixIntLen<u32>>;
///
/// // T = u64 inferred from argument, C = Cfg inferred from config value.
/// let bytes = cencode::encode(&42u64, Cfg::new()).unwrap();
/// assert_eq!(bytes.len(), 8);
/// # }
/// ```
#[cfg(feature = "alloc")]
#[inline]
#[expect(unused_variables)]
pub fn encode<T, C: Config>(src: &T, config: C) -> WriteResult<Vec<u8>>
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

/// Encode `src` into the provided `writer` using the given `config`.
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
/// cencode::encode_into(&mut buf[..], &42u64, Cfg::new()).unwrap();
/// let value: u64 = cdecode::decode(&buf[..], Cfg::new()).unwrap();
/// assert_eq!(value, 42);
/// # }
/// ```
#[inline]
#[expect(unused_variables)]
pub fn encode_into<T, C: Config>(mut writer: impl Writer, src: &T, config: C) -> WriteResult<()>
where
    T: SchemaWrite<C, Src = T> + ?Sized,
{
    T::write(writer.by_ref(), src)?;
    Ok(writer.finish()?)
}

/// Return the number of bytes that `src` would occupy when encoded with the given `config`.
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
/// assert_eq!(cencode::encoded_size(&vec, Cfg::new()).unwrap(), 7);
/// # }
/// ```
#[inline(always)]
#[expect(unused_variables)]
pub fn encoded_size<T, C: Config>(src: &T, config: C) -> WriteResult<usize>
where
    T: SchemaWrite<C, Src = T> + ?Sized,
{
    T::size_of(src)
}

// ── schema-explicit variants (S::Src may differ from S) ──────────────────────

/// Encode `src` into a newly allocated `Vec<u8>` using schema `S` and the given `config`.
///
/// `C` is inferred from `config`; only `S` needs to be named.
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
/// let bytes = cencode::encode_via::<containers::Vec<u8, FixIntLen<u32>>, _>(&vec, Cfg::new()).unwrap();
/// assert_eq!(bytes.len(), 7);
/// # }
/// ```
#[cfg(feature = "alloc")]
#[inline]
#[expect(unused_variables)]
pub fn encode_via<S, C: Config>(src: &S::Src, config: C) -> WriteResult<Vec<u8>>
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

/// Encode `src` into the provided `writer` using schema `S` and the given `config`.
#[inline]
#[expect(unused_variables)]
pub fn encode_into_via<S, C: Config>(mut writer: impl Writer, src: &S::Src, config: C) -> WriteResult<()>
where
    S: SchemaWrite<C>,
{
    S::write(writer.by_ref(), src)?;
    Ok(writer.finish()?)
}

/// Return the number of bytes that `src` would occupy when encoded with schema `S`
/// and the given `config`.
#[inline(always)]
#[expect(unused_variables)]
pub fn encoded_size_via<S, C: Config>(src: &S::Src, config: C) -> WriteResult<usize>
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

    /// `T` inferred from argument, `C` from config value — no turbofish at all.
    /// 4-byte `FixIntLen<u32>` prefix makes the encoding differ from the default.
    #[test]
    fn encode_with_config() {
        let vec: Vec<u8> = vec![1, 2, 3];
        let bytes = encode(&vec, FixLenConfig::new()).unwrap();
        assert_eq!(bytes.len(), 7);
        let len = u32::from_le_bytes(bytes[..4].try_into().unwrap());
        assert_eq!(len as usize, vec.len());
    }

    /// `T` inferred from argument (encode) and return annotation (decode).
    #[test]
    fn roundtrip_no_turbofish() {
        let mut buf = [0u8; 8];
        encode_into(&mut buf[..], &0xCAFEu64, FixLenConfig::new()).unwrap();
        let value: u64 = cdecode::decode(&buf[..], FixLenConfig::new()).unwrap();
        assert_eq!(value, 0xCAFEu64);
    }

    #[test]
    fn encoded_size_with_config() {
        let vec: Vec<u8> = vec![1, 2, 3];
        assert_eq!(encoded_size(&vec, FixLenConfig::new()).unwrap(), 7);
    }

    /// Demonstrate using `config::encode` / `config::decode` both inside impl bodies
    /// (via `C::new()`) and at the call site.
    ///
    /// `ConfigCore::new()` lets generic impl bodies construct a `C` value, bridging
    /// the type-parameter-only world of `SchemaRead`/`SchemaWrite` into the
    /// value-based config:: API.
    #[test]
    fn custom_schema_impl_with_config_api() {
        use crate::{
            SchemaRead, SchemaWrite, TypeMeta,
            config::Config,
            error::{ReadResult, WriteResult},
            io::{Reader, Writer},

        };
        use core::mem::MaybeUninit;

        struct Pair {
            a: u32,
            b: u32,
        }

        // ── impl bodies: use config:: functions via C::new() ─────────────────
        unsafe impl<C: Config> SchemaWrite<C> for Pair {
            type Src = Pair;

            const TYPE_META: TypeMeta = TypeMeta::Static {
                size: 8,
                zero_copy: false,
            };

            fn size_of(_src: &Pair) -> WriteResult<usize> {
                Ok(8)
            }

            fn write(mut writer: impl Writer, src: &Pair) -> WriteResult<()> {
                encode_into_via::<u32, C>(writer.by_ref(), &src.a, C::new())?;
                encode_into_via::<u32, C>(writer.by_ref(), &src.b, C::new())?;
                Ok(())
            }
        }

        unsafe impl<'de, C: Config> SchemaRead<'de, C> for Pair {
            type Dst = Pair;

            const TYPE_META: TypeMeta = TypeMeta::Static {
                size: 8,
                zero_copy: false,
            };

            fn read(mut reader: impl Reader<'de>, dst: &mut MaybeUninit<Pair>) -> ReadResult<()> {
                let a: u32 = cdecode::decode(reader.by_ref(), C::new())?;
                let b: u32 = cdecode::decode(reader.by_ref(), C::new())?;
                dst.write(Pair { a, b });
                Ok(())
            }
        }

        // ── call site: config:: APIs, no turbofish ────────────────────────────
        let pair = Pair { a: 0xAA, b: 0xBB };

        // DefaultConfig
        let bytes = encode(&pair, crate::config::DefaultConfig::default()).unwrap();
        let decoded: Pair = cdecode::decode(&bytes[..], crate::config::DefaultConfig::default()).unwrap();
        assert_eq!(decoded.a, 0xAA);
        assert_eq!(decoded.b, 0xBB);

        // FixIntLen<u32> config — same schema impl, different config, no code change
        let bytes = encode(&pair, FixLenConfig::new()).unwrap();
        let decoded: Pair = cdecode::decode(&bytes[..], FixLenConfig::new()).unwrap();
        assert_eq!(decoded.a, 0xAA);
        assert_eq!(decoded.b, 0xBB);
    }
}

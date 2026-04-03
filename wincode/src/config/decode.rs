/// Configuration-aware decode functions.
///
/// These mirror [`wincode::decode`](crate::decode) but let you specify the configuration `C`
/// as an additional generic type parameter — no runtime config value required.
///
/// Two families are provided:
///
/// - **`decode` / `decode_into` / `decode_exact`** — `T` is both schema and destination
///   (`T::Dst = T`); Rust can infer `T` from a type annotation.
///
/// - **`decode_via` / `decode_into_via` / `decode_exact_via`** — schema `S` and destination
///   `S::Dst` may differ; a turbofish is always required.
use {
    crate::{
        config::Config,
        error::{self, ReadResult},
        io::Reader,
        schema::SchemaRead,
    },
    core::mem::MaybeUninit,
};

// ── inference-friendly variants (T == T::Dst) ────────────────────────────────

/// Decode a value of type `T` from `reader` using configuration `C`.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "alloc")] {
/// use wincode::{config::{self, Configuration, decode as cdecode, encode as cencode}, len::FixIntLen};
///
/// type Cfg = Configuration<true, { config::DEFAULT_PREALLOCATION_SIZE_LIMIT }, FixIntLen<u32>>;
///
/// let bytes = cencode::encode::<u64, Cfg>(&99u64).unwrap();
/// let value: u64 = cdecode::decode::<u64, Cfg>(&bytes[..]).unwrap();
/// assert_eq!(value, 99);
/// # }
/// ```
#[inline(always)]
pub fn decode<'de, T, C: Config>(reader: impl Reader<'de>) -> ReadResult<T>
where
    T: SchemaRead<'de, C, Dst = T>,
{
    T::get(reader)
}

/// Decode into an existing `MaybeUninit<T>` slot from `reader` using configuration `C`.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "alloc")] {
/// use core::mem::MaybeUninit;
/// use wincode::{config::{self, Configuration, decode as cdecode, encode as cencode}, len::FixIntLen};
///
/// type Cfg = Configuration<true, { config::DEFAULT_PREALLOCATION_SIZE_LIMIT }, FixIntLen<u32>>;
///
/// let bytes = cencode::encode::<u64, Cfg>(&7u64).unwrap();
/// let mut dst = MaybeUninit::<u64>::uninit();
/// cdecode::decode_into::<u64, Cfg>(&bytes[..], &mut dst).unwrap();
/// assert_eq!(unsafe { dst.assume_init() }, 7);
/// # }
/// ```
#[inline(always)]
pub fn decode_into<'de, T, C: Config>(
    reader: impl Reader<'de>,
    dst: &mut MaybeUninit<T>,
) -> ReadResult<()>
where
    T: SchemaRead<'de, C, Dst = T>,
{
    T::read(reader, dst)
}

/// Decode from a byte slice using configuration `C`, returning an error if any trailing
/// bytes remain.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "alloc")] {
/// use wincode::{config::{self, Configuration, decode as cdecode, encode as cencode}, len::FixIntLen};
///
/// type Cfg = Configuration<true, { config::DEFAULT_PREALLOCATION_SIZE_LIMIT }, FixIntLen<u32>>;
///
/// let bytes = cencode::encode::<u64, Cfg>(&55u64).unwrap();
/// assert_eq!(cdecode::decode_exact::<u64, Cfg>(&bytes).unwrap(), 55);
///
/// let mut extra = bytes.clone();
/// extra.push(0xFF);
/// assert!(cdecode::decode_exact::<u64, Cfg>(&extra).is_err());
/// # }
/// ```
#[inline(always)]
pub fn decode_exact<'de, T, C: Config>(mut src: &'de [u8]) -> ReadResult<T>
where
    T: SchemaRead<'de, C, Dst = T>,
{
    let value = T::get(src.by_ref())?;
    if src.is_empty() {
        Ok(value)
    } else {
        Err(error::trailing_bytes())
    }
}

// ── schema-explicit variants (S::Dst may differ from S) ──────────────────────

/// Decode from `reader` using schema `S` and configuration `C`, returning `S::Dst`.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "alloc")] {
/// use wincode::{
///     config::{self, Configuration, decode as cdecode, encode as cencode},
///     containers, len::FixIntLen,
/// };
///
/// type Cfg = Configuration<true, { config::DEFAULT_PREALLOCATION_SIZE_LIMIT }, FixIntLen<u32>>;
///
/// let original: Vec<u8> = vec![1, 2, 3];
/// let bytes = cencode::encode::<Vec<u8>, Cfg>(&original).unwrap();
/// // containers::Vec with matching FixIntLen<u32> length encoding.
/// type VecSchema = containers::Vec<u8, FixIntLen<u32>>;
/// let decoded: Vec<u8> = cdecode::decode_via::<VecSchema, Cfg>(&bytes[..]).unwrap();
/// assert_eq!(decoded, original);
/// # }
/// ```
#[inline(always)]
pub fn decode_via<'de, S, C: Config>(reader: impl Reader<'de>) -> ReadResult<S::Dst>
where
    S: SchemaRead<'de, C>,
{
    S::get(reader)
}

/// Decode from `reader` into an existing `MaybeUninit<S::Dst>` slot using schema `S`
/// and configuration `C`.
#[inline(always)]
pub fn decode_into_via<'de, S, C: Config>(
    reader: impl Reader<'de>,
    dst: &mut MaybeUninit<S::Dst>,
) -> ReadResult<()>
where
    S: SchemaRead<'de, C>,
{
    S::read(reader, dst)
}

/// Decode from a byte slice using schema `S` and configuration `C`, returning an error
/// if any trailing bytes remain.
#[inline(always)]
pub fn decode_exact_via<'de, S, C: Config>(mut src: &'de [u8]) -> ReadResult<S::Dst>
where
    S: SchemaRead<'de, C>,
{
    let value = S::get(src.by_ref())?;
    if src.is_empty() {
        Ok(value)
    } else {
        Err(error::trailing_bytes())
    }
}

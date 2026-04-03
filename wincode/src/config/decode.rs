/// Configuration-aware decode functions.
///
/// These mirror [`wincode::decode`](crate::decode) but accept a `config: C` value so that
/// both `T` and `C` are inferred at the call site — no turbofish required for the common case.
///
/// Two families are provided:
///
/// - **`decode` / `decode_into` / `decode_exact`** — `T` is both schema and destination
///   (`T::Dst = T`); fully inferred from a return-type annotation.
///
/// - **`decode_via` / `decode_into_via` / `decode_exact_via`** — schema `S` and destination
///   `S::Dst` may differ; `S` must still be named but `C` is inferred from the config value.
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

/// Decode a value of type `T` from `reader` using the given `config`.
///
/// Both `T` and `C` are inferred — `T` from a return-type annotation, `C` from `config`.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "alloc")] {
/// use wincode::{config::{self, Configuration, decode as cdecode, encode as cencode}, len::FixIntLen};
///
/// type Cfg = Configuration<true, { config::DEFAULT_PREALLOCATION_SIZE_LIMIT }, FixIntLen<u32>>;
///
/// let bytes = cencode::encode(&99u64, Cfg::new()).unwrap();
/// let value: u64 = cdecode::decode(&bytes[..], Cfg::new()).unwrap();
/// assert_eq!(value, 99);
/// # }
/// ```
#[inline(always)]
#[expect(unused_variables)]
pub fn decode<'de, T, C: Config>(reader: impl Reader<'de>, config: C) -> ReadResult<T>
where
    T: SchemaRead<'de, C, Dst = T>,
{
    T::get(reader)
}

/// Decode into an existing `MaybeUninit<T>` slot from `reader` using the given `config`.
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
/// let bytes = cencode::encode(&7u64, Cfg::new()).unwrap();
/// let mut dst = MaybeUninit::uninit();
/// cdecode::decode_into(&bytes[..], &mut dst, Cfg::new()).unwrap();
/// let value: u64 = unsafe { dst.assume_init() };
/// assert_eq!(value, 7);
/// # }
/// ```
#[inline(always)]
#[expect(unused_variables)]
pub fn decode_into<'de, T, C: Config>(
    reader: impl Reader<'de>,
    dst: &mut MaybeUninit<T>,
    config: C,
) -> ReadResult<()>
where
    T: SchemaRead<'de, C, Dst = T>,
{
    T::read(reader, dst)
}

/// Decode from a byte slice using the given `config`, returning an error if any trailing
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
/// let bytes = cencode::encode(&55u64, Cfg::new()).unwrap();
/// let value: u64 = cdecode::decode_exact(&bytes, Cfg::new()).unwrap();
/// assert_eq!(value, 55);
///
/// let mut extra = bytes.clone();
/// extra.push(0xFF);
/// assert!(cdecode::decode_exact::<u64, _>(&extra, Cfg::new()).is_err());
/// # }
/// ```
#[inline(always)]
#[expect(unused_variables)]
pub fn decode_exact<'de, T, C: Config>(mut src: &'de [u8], config: C) -> ReadResult<T>
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

/// Decode from `reader` using schema `S` and the given `config`, returning `S::Dst`.
///
/// `C` is inferred from `config`; only `S` needs to be named.
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
/// let bytes = cencode::encode(&original, Cfg::new()).unwrap();
/// type VecSchema = containers::Vec<u8, FixIntLen<u32>>;
/// let decoded: Vec<u8> = cdecode::decode_via::<VecSchema, _>(&bytes[..], Cfg::new()).unwrap();
/// assert_eq!(decoded, original);
/// # }
/// ```
#[inline(always)]
#[expect(unused_variables)]
pub fn decode_via<'de, S, C: Config>(reader: impl Reader<'de>, config: C) -> ReadResult<S::Dst>
where
    S: SchemaRead<'de, C>,
{
    S::get(reader)
}

/// Decode from `reader` into an existing `MaybeUninit<S::Dst>` slot using schema `S`
/// and the given `config`.
#[inline(always)]
#[expect(unused_variables)]
pub fn decode_into_via<'de, S, C: Config>(
    reader: impl Reader<'de>,
    dst: &mut MaybeUninit<S::Dst>,
    config: C,
) -> ReadResult<()>
where
    S: SchemaRead<'de, C>,
{
    S::read(reader, dst)
}

/// Decode from a byte slice using schema `S` and the given `config`, returning an error
/// if any trailing bytes remain.
#[inline(always)]
#[expect(unused_variables)]
pub fn decode_exact_via<'de, S, C: Config>(mut src: &'de [u8], config: C) -> ReadResult<S::Dst>
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

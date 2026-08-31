//! What a struct's field layout costs its encoding and decoding.
//!
//! The derive emits one read or write per field at constant offsets, so how much the backend
//! can merge, and how many bounds checks it can drop, follows from the fields themselves:
//! where they sit, and which of them have a size known ahead of time. Arms in a group encode
//! the same number of wire bytes -- the size the group is named for -- so a gap between two of
//! them reflects whatever that group varies rather than a difference in how much data moved.
//!
//! For a number worth quoting use the protocol in `benchmarks.rs`: 64-byte function alignment,
//! one benchmark per process, pinned to a core, keeping `-Ctarget-cpu=native`.
//!
//! ```text
//! RUSTFLAGS="-Ctarget-cpu=native -Cllvm-args=-align-all-functions=6" \
//!     cargo bench --features derive --bench wire_layout --no-run
//! taskset -c 4 target/release/deps/wire_layout-<hash> \
//!     --bench --exact "Struct32/prefixed/deserialize"
//! ```

use {
    criterion::{
        Bencher, Criterion, Throughput, criterion_group, criterion_main, measurement::WallTime,
    },
    std::hint::black_box,
    wincode::{
        SchemaRead, SchemaWrite, config::DefaultConfig, deserialize, serialize_into,
        serialized_size,
    },
};

/// Serialized size shared by all three arms.
const WIRE: usize = 32;

/// No dynamic field, so every offset is fixed.
#[derive(SchemaWrite, SchemaRead)]
struct Header {
    h: u64,
    i: u64,
    f: u32,
    g: u32,
    d: u16,
    e: u16,
    a: u8,
    b: u8,
    c: u8,
    k: u8,
}

impl Header {
    fn new() -> Self {
        Self {
            h: 1,
            i: 2,
            f: 3,
            g: 4,
            d: 5,
            e: 6,
            a: 7,
            b: 8,
            c: 9,
            k: 10,
        }
    }
}

/// Eight static fields worth 30 bytes, then a dynamic one: the first eight offsets are fixed.
#[derive(SchemaWrite, SchemaRead)]
struct Prefixed {
    h: u64,
    i: u64,
    f: u32,
    g: u32,
    d: u16,
    e: u16,
    a: u8,
    b: u8,
    tail: Option<u8>,
}

impl Prefixed {
    fn new() -> Self {
        Self {
            h: 1,
            i: 2,
            f: 3,
            g: 4,
            d: 5,
            e: 6,
            a: 7,
            b: 8,
            tail: Some(11),
        }
    }
}

/// Dynamic field first, so no offset is fixed. Control: should not move.
#[derive(SchemaWrite, SchemaRead)]
struct Suffixed {
    head: Option<u8>,
    h: u64,
    i: u64,
    f: u32,
    g: u32,
    d: u16,
    e: u16,
    a: u8,
    b: u8,
}

impl Suffixed {
    fn new() -> Self {
        Self {
            head: Some(11),
            h: 1,
            i: 2,
            f: 3,
            g: 4,
            d: 5,
            e: 6,
            a: 7,
            b: 8,
        }
    }
}

/// Encode once, checking the arm is the size the group claims.
fn encoded<T>(data: &T) -> [u8; WIRE]
where
    T: SchemaWrite<DefaultConfig, Src = T>,
{
    assert_eq!(serialized_size(data).unwrap() as usize, WIRE);
    let mut bytes = [0u8; WIRE];
    serialize_into(bytes.as_mut_slice(), data).unwrap();
    bytes
}

/// Setup stays behind the helper: criterion filters inside `bench_function`, so setup left in
/// the bench fn runs even for benchmarks a `--bench` filter excluded.
fn run_serialize<T>(new_data: impl Fn() -> T) -> impl FnMut(&mut Bencher<'_, WallTime>)
where
    T: SchemaWrite<DefaultConfig, Src = T>,
{
    move |b| {
        let data = new_data();
        let mut buffer = encoded(&data);
        b.iter(|| serialize_into(black_box(buffer.as_mut_slice()), black_box(&data)).unwrap());
    }
}

/// [`run_serialize`] for the decode, which measures against the encoded bytes.
fn run_deserialize<T, R>(
    new_data: impl Fn() -> T,
    deserialize_fn: impl Fn(&[u8]) -> R,
) -> impl FnMut(&mut Bencher<'_, WallTime>)
where
    T: SchemaWrite<DefaultConfig, Src = T>,
{
    move |b| {
        let bytes = encoded(&new_data());
        b.iter(|| deserialize_fn(black_box(bytes.as_slice())));
    }
}

/// Where a dynamic field sits. Nothing after one has a fixed offset, so the arms differ in how
/// many of their fields keep theirs. That field is an `Option<u8>`: its size varies with the
/// variant, but it is owned, tiny and allocation-free, so neither side pays for anything else.
#[inline(never)]
fn bench_static_prefix(c: &mut Criterion) {
    let mut group = c.benchmark_group("Struct32");
    group.throughput(Throughput::Elements(1));

    group.bench_function("header/serialize_into", run_serialize(Header::new));
    group.bench_function(
        "header/deserialize",
        run_deserialize(Header::new, |s| deserialize::<Header>(s).unwrap()),
    );

    group.bench_function("prefixed/serialize_into", run_serialize(Prefixed::new));
    group.bench_function(
        "prefixed/deserialize",
        run_deserialize(Prefixed::new, |s| deserialize::<Prefixed>(s).unwrap()),
    );

    group.bench_function("suffixed/serialize_into", run_serialize(Suffixed::new));
    group.bench_function(
        "suffixed/deserialize",
        run_deserialize(Suffixed::new, |s| deserialize::<Suffixed>(s).unwrap()),
    );

    group.finish();
}

criterion_group!(benches, bench_static_prefix);

criterion_main!(benches);

use {
    criterion::{Criterion, Throughput, criterion_group, criterion_main},
    std::hint::black_box,
    wincode::{
        deserialize_from,
        io::{LimitReader, Reader},
        serialize,
    },
};

fn bench_limit(c: &mut Criterion) {
    let ints = (0..4096u64).collect::<Vec<_>>();
    let bytes = serialize(&ints).unwrap();
    let strings = (0..2048).map(|i| format!("value-{i}")).collect::<Vec<_>>();
    let string_bytes = serialize(&strings).unwrap();

    let mut group = c.benchmark_group("limit/vec_u64");
    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("unlimited", |b| {
        b.iter(|| deserialize_from::<Vec<u64>>(black_box(bytes.as_slice())).unwrap())
    });
    // Control: one level of reader indirection, no limiting at all.
    group.bench_function("by_ref", |b| {
        b.iter(|| {
            let mut src = black_box(bytes.as_slice());
            deserialize_from::<Vec<u64>>(src.by_ref()).unwrap()
        })
    });
    group.bench_function("limit_reader", |b| {
        b.iter(|| {
            deserialize_from::<Vec<u64>>(LimitReader::new(black_box(bytes.as_slice()), usize::MAX))
                .unwrap()
        })
    });
    group.bench_function("limited_slice", |b| {
        b.iter(|| {
            let mut src = black_box(bytes.as_slice());
            deserialize_from::<Vec<u64>>(src.as_limited_for(usize::MAX)).unwrap()
        })
    });
    // No wrapper at all: the limit is applied by shortening the slice, so the reader type is
    // identical to the unlimited case.
    group.bench_function("truncated", |b| {
        b.iter(|| {
            let src = black_box(bytes.as_slice());
            deserialize_from::<Vec<u64>>(&src[..usize::MAX.min(src.len())]).unwrap()
        })
    });
    group.finish();

    let mut group = c.benchmark_group("limit/vec_string");
    group.throughput(Throughput::Bytes(string_bytes.len() as u64));
    group.bench_function("unlimited", |b| {
        b.iter(|| deserialize_from::<Vec<String>>(black_box(string_bytes.as_slice())).unwrap())
    });
    group.bench_function("by_ref", |b| {
        b.iter(|| {
            let mut src = black_box(string_bytes.as_slice());
            deserialize_from::<Vec<String>>(src.by_ref()).unwrap()
        })
    });
    group.bench_function("limit_reader", |b| {
        b.iter(|| {
            deserialize_from::<Vec<String>>(LimitReader::new(
                black_box(string_bytes.as_slice()),
                usize::MAX,
            ))
            .unwrap()
        })
    });
    group.bench_function("limited_slice", |b| {
        b.iter(|| {
            let mut src = black_box(string_bytes.as_slice());
            deserialize_from::<Vec<String>>(src.as_limited_for(usize::MAX)).unwrap()
        })
    });
    group.bench_function("truncated", |b| {
        b.iter(|| {
            let src = black_box(string_bytes.as_slice());
            deserialize_from::<Vec<String>>(&src[..usize::MAX.min(src.len())]).unwrap()
        })
    });
    group.finish();
}

criterion_group!(benches, bench_limit);
criterion_main!(benches);

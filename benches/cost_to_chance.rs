use criterion::{Criterion, criterion_group, criterion_main};
use honing_forecast::*;
use std::hint::black_box;
use std::time::Instant;
pub fn criterion_benchmark(c: &mut Criterion) {
    let now: Instant = Instant::now();
    let mut group = c.benchmark_group("sample-size-example");

    // Set the sample size to 10
    group.sample_size(10);

    group.bench_function("cost_to_chance", |b| {
        b.iter(|| {
            cost_to_chance_test_wrapper(
                black_box(vec![vec![true; 25]; 6]),
                black_box(vec![vec![true; 4]; 6]),
                black_box(
                    [
                        431777, 1064398, 23748, 9010948, 15125, 1803792, 4294967295, 0, 0, 20000,
                    ]
                    .to_vec(),
                ),
                true,
            )
        })
    });
    // println!("{}", chance.to_string());
    // println!("{}", reason.to_string());
    let elapsed: std::time::Duration = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

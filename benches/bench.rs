use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use dispatch_bench::Dispatcher;
use rand::Rng;
use std::sync::atomic::{AtomicPtr, Ordering};

#[inline(always)]
fn xor(i: &[u8]) -> u8 {
    use std::ops::BitXor;
    i.iter().fold(0, u8::bitxor)
}

fn fancy_branching_xor(i: &[u8]) -> u8 {
    Dispatcher::<false>::dispatch(|| xor(i))
}

fn fancy_indirect_xor(i: &[u8]) -> u8 {
    Dispatcher::<true>::dispatch(|| xor(i))
}

fn standard_indirect_xor(i: &[u8]) -> u8 {
    #[target_feature(enable = "avx2")]
    #[target_feature(enable = "avx")]
    unsafe fn avx2_avx(i: &[u8]) -> u8 {
        xor(i)
    }

    #[target_feature(enable = "avx")]
    unsafe fn avx(i: &[u8]) -> u8 {
        xor(i)
    }

    unsafe fn none(i: &[u8]) -> u8 {
        xor(i)
    }

    static FN: AtomicPtr<()> = AtomicPtr::new(detect as *mut ());

    fn detect(i: &[u8]) -> u8 {
        let f = if is_x86_feature_detected!("avx") && is_x86_feature_detected!("avx") {
            avx2_avx
        } else if is_x86_feature_detected!("avx") {
            avx
        } else {
            none
        };

        FN.store(f as *mut (), Ordering::Relaxed);

        unsafe { f(i) }
    }

    unsafe {
        let f = FN.load(Ordering::Relaxed);
        let f: fn(&[u8]) -> u8 = std::mem::transmute(f);
        f(i)
    }
}

fn bench_dispatcher(c: &mut Criterion) {
    let mut group = c.benchmark_group("Dispatchers");
    let mut rng = rand::thread_rng();
    for len in [8, 1024].iter() {
        let i: Vec<u8> = (0..*len).map(|_| rng.gen()).collect();
        group.bench_with_input(
            BenchmarkId::new("Fancy branching dispatch", len),
            i.as_slice(),
            |b, i| b.iter(|| fancy_branching_xor(i)),
        );
        group.bench_with_input(
            BenchmarkId::new("Fancy indirect dispatch", len),
            i.as_slice(),
            |b, i| b.iter(|| fancy_indirect_xor(i)),
        );
        group.bench_with_input(
            BenchmarkId::new("Standard indirect dispatch", len),
            i.as_slice(),
            |b, i| b.iter(|| standard_indirect_xor(i)),
        );
        group.bench_with_input(
            BenchmarkId::new("No dispatching", len),
            i.as_slice(),
            |b, i| b.iter(|| xor(i)),
        );
    }
}

criterion_group!(benches, bench_dispatcher);
criterion_main!(benches);

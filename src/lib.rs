use std::sync::atomic::{AtomicUsize, Ordering};

pub struct Dispatcher<const INDIRECT: bool>;

impl<const INDIRECT: bool> Dispatcher<INDIRECT> {
    #[target_feature(enable = "avx2")]
    #[target_feature(enable = "avx")]
    unsafe fn avx2_avx<Output>(f: impl FnOnce() -> Output) -> Output {
        f()
    }

    #[target_feature(enable = "avx")]
    unsafe fn avx<Output>(f: impl FnOnce() -> Output) -> Output {
        f()
    }

    unsafe fn none<Output>(f: impl FnOnce() -> Output) -> Output {
        f()
    }

    #[cold]
    fn detect() -> usize {
        if is_x86_feature_detected!("avx2") & is_x86_feature_detected!("avx") {
            2
        } else if is_x86_feature_detected!("avx") {
            1
        } else {
            0
        }
    }

    pub fn dispatch<Output>(f: impl FnOnce() -> Output) -> Output {
        static SELECTED: AtomicUsize = AtomicUsize::new(usize::MAX);
        let selected = SELECTED.load(Ordering::Relaxed);
        let selected = if selected == usize::MAX {
            let selected = Self::detect();
            SELECTED.store(selected, Ordering::Relaxed);
            selected
        } else {
            selected
        };

        if INDIRECT {
            let fns = [Self::none, Self::avx, Self::avx2_avx];
            unsafe { fns.get_unchecked(selected)(f) }
        } else {
            match selected {
                2 => unsafe { Self::avx2_avx(f) },
                1 => unsafe { Self::avx(f) },
                0 => unsafe { Self::none(f) },
                _ => unsafe { std::hint::unreachable_unchecked() },
            }
        }
    }
}

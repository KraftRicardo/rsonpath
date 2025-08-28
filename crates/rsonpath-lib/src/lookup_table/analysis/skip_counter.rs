use std::sync::atomic::{AtomicU64, Ordering};

static SKIP_TIME_ATOMIC_CUTOFF_0: AtomicU64 = AtomicU64::new(0);
static SKIP_TIME_ATOMIC_CUTOFF_64: AtomicU64 = AtomicU64::new(0);
static SKIP_TIME_ATOMIC_CUTOFF_128: AtomicU64 = AtomicU64::new(0);
static SKIP_TIME_ATOMIC_CUTOFF_256: AtomicU64 = AtomicU64::new(0);
static SKIP_TIME_ATOMIC_CUTOFF_512: AtomicU64 = AtomicU64::new(0);
static SKIP_TIME_ATOMIC_CUTOFF_1024: AtomicU64 = AtomicU64::new(0);
static SKIP_TIME_ATOMIC_CUTOFF_2048: AtomicU64 = AtomicU64::new(0);

#[inline]
pub fn add_skip_time(distance: usize, added_time: u64) {
    if distance > 0 {
        SKIP_TIME_ATOMIC_CUTOFF_0.fetch_add(added_time, Ordering::Relaxed);
    }
    if distance > 64 {
        SKIP_TIME_ATOMIC_CUTOFF_64.fetch_add(added_time, Ordering::Relaxed);
    }
    if distance > 128 {
        SKIP_TIME_ATOMIC_CUTOFF_128.fetch_add(added_time, Ordering::Relaxed);
    }
    if distance > 256 {
        SKIP_TIME_ATOMIC_CUTOFF_256.fetch_add(added_time, Ordering::Relaxed);
    }
    if distance > 512 {
        SKIP_TIME_ATOMIC_CUTOFF_512.fetch_add(added_time, Ordering::Relaxed);
    }
    if distance > 1024 {
        SKIP_TIME_ATOMIC_CUTOFF_1024.fetch_add(added_time, Ordering::Relaxed);
    }
    if distance > 2048 {
        SKIP_TIME_ATOMIC_CUTOFF_2048.fetch_add(added_time, Ordering::Relaxed);
    }
}

#[inline]
pub fn reset_skip_counters() {
    SKIP_TIME_ATOMIC_CUTOFF_0.store(0, Ordering::Relaxed);
    SKIP_TIME_ATOMIC_CUTOFF_64.store(0, Ordering::Relaxed);
    SKIP_TIME_ATOMIC_CUTOFF_128.store(0, Ordering::Relaxed);
    SKIP_TIME_ATOMIC_CUTOFF_256.store(0, Ordering::Relaxed);
    SKIP_TIME_ATOMIC_CUTOFF_512.store(0, Ordering::Relaxed);
    SKIP_TIME_ATOMIC_CUTOFF_1024.store(0, Ordering::Relaxed);
    SKIP_TIME_ATOMIC_CUTOFF_2048.store(0, Ordering::Relaxed);
}

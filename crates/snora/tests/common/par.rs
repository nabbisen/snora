//! The drawn sweeps' presets, on parallel threads (0.66: the sweeps' own time).
//!
//! Each sweep draws about 5,000 frames, one after another, and the tiny-skia sweeps are the longest jobs in CI. Their time is
//! the rasterising (`Headless::screenshot`, 90% of it on tiny-skia: `housekeeping-0-66`'s measurement), so it is the presets
//! that run at once: **a preset's cells are read by one thread, with renderers of its own taken from [`removal`]'s pool**, and
//! the results are put back **in preset order**, so that the cells, and the failures a sweep reports, come out exactly as
//! they did when one thread drew them all.
//!
//! * **How many threads:** `SNORA_SWEEP_THREADS` (an integer), else **one per item up to eight on tiny-skia, and one on wgpu**:
//!   the GPU stack's contexts do not share a process safely (RFC-116 Unit 0 found that parallel wgpu contexts segfault; the
//!   wgpu sweeps were stable and identical in 7 of 7 runs with 4, 6 and 8 threads here, a local GPU, and only 30% faster, and
//!   CI's wgpu is lavapipe, which that finding may not have been about: so it is an opt-in, not a default). A run with
//!   `SNORA_SWEEP_THREADS=1` is the old, sequential sweep.
//! * **The pool:** a `static` that is never dropped (a thread-local one aborted at exit on wgpu). It is warmed on the calling
//!   thread before any worker starts, so that renderers are created one at a time, and each worker then only borrows and
//!   returns: a `Mutex` serialises that, and nothing else is shared.
//! * **A panic** in a worker is raised again on the calling thread with its own message.

use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use snora::design::Tokens;

use super::removal;

/// The most workers a sweep starts by default. A sweep's items (six presets, 22 themes) are not equal in cost, so a worker per
/// item balances better than a worker per core: measured on four cores (`taskset -c 0-3`), the three tiny-skia sweeps took
/// 126 s with four workers and 91 s with six (against 350 s with one).
const MOST_BY_DEFAULT: usize = 8;

/// How many worker threads for `items` items.
pub fn threads(items: usize) -> usize {
    if let Some(n) = std::env::var("SNORA_SWEEP_THREADS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
    {
        return n.clamp(1, items.max(1));
    }
    if removal::backend() != "tiny-skia" {
        return 1;
    }
    items.clamp(1, MOST_BY_DEFAULT)
}

/// Runs `f` for every preset, and returns what each returned **in preset order**.
pub fn by_preset<T: Send>(f: impl Fn(&'static str, Tokens) -> T + Sync) -> Vec<T> {
    let presets: Vec<(&'static str, Tokens)> = Tokens::presets();
    map(&presets, |(name, tokens)| f(name, tokens.clone()))
}

/// Runs `f` for every item, on up to [`threads`] threads, and returns what each returned **in the items' order**.
pub fn map<X: Sync, T: Send>(items: &[X], f: impl Fn(&X) -> T + Sync) -> Vec<T> {
    let n = threads(items.len());
    if n <= 1 {
        return items.iter().map(&f).collect();
    }
    // Two renderers a worker (a rig for the rest frame, and one for a pointer state), created here, one at a time.
    removal::warm(2 * n);
    let next = AtomicUsize::new(0);
    let slots: Vec<Mutex<Option<T>>> = items.iter().map(|_| Mutex::new(None)).collect();
    std::thread::scope(|scope| {
        let workers: Vec<_> = (0..n)
            .map(|_| {
                scope.spawn(|| {
                    loop {
                        let i = next.fetch_add(1, Ordering::SeqCst);
                        let Some(item) = items.get(i) else {
                            break;
                        };
                        let out = f(item);
                        *slots[i]
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(out);
                    }
                })
            })
            .collect();
        for worker in workers {
            if let Err(panic) = worker.join() {
                std::panic::resume_unwind(panic);
            }
        }
    });
    slots
        .into_iter()
        .map(|slot| {
            slot.into_inner()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .expect("every item was drawn")
        })
        .collect()
}

/// `SNORA_SWEEP_CELLS=<dir>`: writes a sweep's cells, one per line and **to the last digit** (`f32`s as `{:?}`), to
/// `<dir>/<name>-<renderer>.tsv`, so that two runs of a sweep can be compared with `cmp`. A sweep does it once, when its
/// cells are drawn. Without the variable it does nothing.
pub fn dump_cells(name: &str, lines: impl IntoIterator<Item = String>) {
    let Ok(dir) = std::env::var("SNORA_SWEEP_CELLS") else {
        return;
    };
    std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("{dir}: {e}"));
    let path = format!("{dir}/{name}-{}.tsv", removal::backend());
    let mut text = lines.into_iter().collect::<Vec<_>>().join("\n");
    text.push('\n');
    std::fs::write(&path, text).unwrap_or_else(|e| panic!("{path}: {e}"));
}

use std::time::{Duration, Instant};

use smol::stream::StreamExt;
use smol_concurrency_tools::*;

fn main() {
    smol::block_on(async {
        let mut timer = Interval::new(Duration::from_secs_f32(0.5), true);
        let total = Instant::now();
        while let Some(time) = timer.next().await {
            println!(
                "{:0.05}s - {:0.05}s",
                time.as_secs_f32(),
                total.elapsed().as_secs_f32()
            );
        }
    })
}

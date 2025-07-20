# smol-concurrency-tools
A collection of nice tools developed for my own projects using [smol](https://github.com/smol-rs/smol).
They are small, but work pretty nicely so thought I could publish them.

Documentation can be found here: [loafey.se/smol-concurrency-tools](https://loafey.se/smol-concurrency-tools).

## Core content:
- `Interval`: A Stream which runs infinitely and returns on a set interval.
- `Repeat`: A stream which contains a task which will be repeated infinitely (i.e as long as it gets awaited).
- `Repeats`: A stream which repeats a set of tasks infinitely (i.e as long as it gets awaited).

## Macros: 
- `repeat`: A clean wrapper around the `Repeats` struct:
    ```rs
    let mut streams = repeat!(
        async {
            Timer::after(Duration::from_secs(1)).await;
            1
        },
        async {
            Timer::after(Duration::from_secs(2)).await;
            2
        }
    );
    while let Some(p) = streams.next().await {
        println!("{p}");
    }
    ```
- `select`: A port of the popular `select!` macro from other async frameworks, enabled with the `select` feature:
    ```rs
    loop {
        select!(
            (sec1.next(), |d| {
                let d = d.unwrap();
                println!(
                    "1 second - since_last: {:0.5}s, total_time: {:0.5}s",
                    d.as_secs_f32(),
                    timer.elapsed().as_secs_f32()
                );
            }),
            (sec2.next(), |d| {
                let d = d.unwrap();
                println!(
                    "2 second - since_last: {:0.5}s, total_time: {:0.5}s",
                    d.as_secs_f32(),
                    timer.elapsed().as_secs_f32()
                );
            })
        );
    }
    ```

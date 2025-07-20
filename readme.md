# smol-concurrency-tools
A collection of nice tools developed for projects using [smol](https://github.com/smol-rs/smol).

Documentation can be found here: [loafey.se/smol-concurrency-tools](https://loafey.se/smol-concurrency-tools).

## Core content:
- `Interval`: A Stream which runs infinitely and returns on a set interval.
- `Repeat`: A stream which contains a task which will be repeated infinitely (i.e as long as it gets awaited).
- `Repeats`: A stream which repeats a set of tasks infinitely (i.e as long as it gets awaited).

## Macros: 
- `repeat`: Nice wrapper around `Repeats`
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
- `select`: Port of the popular `select!` macro from other async toolkits, enabled with the `select` feature:
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

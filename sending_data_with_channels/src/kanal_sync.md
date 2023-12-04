# Let's Dig a Kanal

[Kanal](https://github.com/fereidani/kanal) proudly names itself "the fast sync and async channel that Rust deserves". When you read some of the development history, you might well think that the creators are a little crazy: they do a *lot* of unsafe code, in quite ingenious ways. For example, Kanal moves stack pointers around to allow for zero-copy operations.

So if you're uncomfortable with lots of unsafe code---albeit unsafe code nicely packaged behind a safe face, Kanal may not be for you. If you have a serious need for speed, Kanal can work miracles.

## Send Time with Kanal

Let's take our send-time benchmark and port it to use Kanal. We'll run `cargo add kanal` to add it to our project, and replace the channels with Kanal channels:

> The source code for this is in `channel_code/send_bench_kanal`.

We actually changed one line:

```rust
let (sender, receiver) = kanal::bounded(10_000);
```

The results are:

```
Mean: 57.8329 ns
Max : 6851 ns
Min : 19 ns
```

In other words, it's basically the same on sending time.

## How about the latency test?

Once again, we added Kanal with `cargo add kanal` and changed one line of code:

```rust
let (send, recv) = kanal::bounded::<Command>(100);
```

Once again, the results are about the same:

```
Message latency: 100883 nanos
Message latency: 106617 nanos
Message latency: 107095 nanos
Message latency: 107475 nanos
Message latency: 107831 nanos
Message latency: 108187 nanos
Message latency: 108540 nanos
Message latency: 108940 nanos
Message latency: 109308 nanos
Message latency: 109679 nanos
Quitting.
```

So why does Kanal show up in the benchmarks as being up to 80x faster? The answer lies in its handling of a *lot* of messages, with tons of threads.

Let's build a slightly overkill testbed:

> The source code for this is in `channel_code/channel_latency_kanal_big`. Not that `Cargo.toml` has changed to allow for feature flags.

```rust
use std::{time::Instant, sync::Mutex};

enum Command {
    Latency(Instant),
    Quit
}

const CHANNEL_SIZE: usize = 100;
const THREADS: usize = 100;
static LATENCIES: Mutex<Vec<u128>> = Mutex::new(Vec::new());

#[cfg(feature = "kanal")]
fn connect() -> (kanal::Sender<Command>, kanal::Receiver<Command>) {
    kanal::bounded::<Command>(CHANNEL_SIZE)
}

#[cfg(not(feature = "kanal"))]
fn connect() -> (std::sync::mpsc::SyncSender<Command>, std::sync::mpsc::Receiver<Command>) {
    std::sync::mpsc::sync_channel::<Command>(CHANNEL_SIZE)
}

fn main() {
    let (send, recv) = connect();

    let join_handle = std::thread::spawn(move || {
        while let Ok(command) = recv.recv() {
            match command {
                Command::Latency(instant) => {
                    let elapsed = instant.elapsed();
                    LATENCIES.lock().unwrap().push(elapsed.as_nanos());
                }
                Command::Quit => {
                    println!("Quitting.");
                    break;
                }
            }
        }
    });

    let mut handles = Vec::new();
    for _ in 0..THREADS {
        let my_send = send.clone();
        handles.push(std::thread::spawn(move || {
            for _ in 0..100_000 {
                let _ = my_send.send(Command::Latency(Instant::now())).unwrap();
            }
        }));
    }
    handles.into_iter().for_each(|handle| handle.join().unwrap());
    send.send(Command::Quit).unwrap();

    join_handle.join().unwrap();

    let latencies = LATENCIES.lock().unwrap();
    let sum: u128 = latencies.iter().sum();
    let count = latencies.len();
    let average = sum / count as u128;
    println!("Average latency: {} nanos", average);

}
```

This is setup so we can `cargo run --release` to test the standard library, and `cargo run --release --features kanal` to test Kanal in the same testbed. It's a little crazy, we are spawning 100 threads, each of which sends 100,000 messages - into a bounded channel of 100 messages.

With the standard library, I achieved `271,548 nanos` message latency. Kanal gave me `137,710 nanos` for the same test. So in a heavily contended setup, Kanal is a big winner.

Let's tweak the constants and re-run a few times:

Threads | Bounds | Std Latency (nanos) | Kanal Latency (nanos) | Comment
-|-|-|-|-|
100|1|466,204|61,547|Extreme back-pressure. Don't do this in real life.
100|100|273,950|140,122|High back-pressure
100|1,000|1,499,566|871,578|Still quite a bit of pressure. Performance is degrading from too large a queue causing heavy allocations.

The lesson here is that if you have a lot of back-pressure: Kanal is your savior. If you don't, then the standard library is fine.


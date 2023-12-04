# Async Channels

Rust's `async` system gets a bit of bad press, but the foundations are all there to build really impressive systems.

First of all, the good news: you can use thread-based channels in async-land unchanged in many cases. The bad news: occasionally, this will bite you. It will also be a little slower.

Fortunately, it's easy to write an async channel system that looks a lot like our original channel demo. We've added `tokio` with `cargo add tokio -F full` (full featureset because I'm lazy):

> The source code is in `channel_code/async_mpsc`.

```rust
enum Command {
    Print(String),
    Quit,
}

#[tokio::main]
async fn main() {
    let (sender, mut receiver) = tokio::sync::mpsc::channel::<Command>(100);

    tokio::spawn(async move {
        loop {
            while let Some(command) = receiver.recv().await {
                match command {
                    Command::Print(s) => println!("{}", s),
                    Command::Quit => break,
                }
            }
        }
    });

    for i in 0..10 {
        let sender = sender.clone();
        tokio::spawn(async move {
            sender
                .send(Command::Print(format!("Hello, world! {}", i)))
                .await
                .unwrap();
        });
    }
    sender.send(Command::Quit).await.unwrap();
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
}
```

Notice that the syntax is basically the same. We're waiting rather than joining at the end, we *had* to bound our channel size (Tokio channels don't provide an unbounded option, just like Go), and our receiver had to be mutable. Otherwise, it works the same.

You can `tokio::spawn` inside your receiver, to spin off even more tasks. Remember, tasks are cheap and threads are expensive. So when you're in async land, feel free to spawn a lot!

## Timing Tokio and Kanal

Kanal can do async too! Let's build a similar test to the one we used for synchronous code, but all async.

> The code is in `channel_code/async_mpsc_latency`.

```rust
use std::time::{Instant, Duration};
use once_cell::sync::Lazy;
use tokio::sync::Mutex;

const CHANNEL_SIZE: usize = 100;
const TASKS: usize = 100;
static LATENCIES: Lazy<Mutex<Vec<u128>>> = Lazy::new(|| Mutex::new(Vec::new()));

enum Command {
    Latency(Instant),
    Quit
}

#[cfg(not(feature = "kanal"))]
fn connect() -> (tokio::sync::mpsc::Sender<Command>, tokio::sync::mpsc::Receiver<Command>) {
    tokio::sync::mpsc::channel::<Command>(CHANNEL_SIZE)
}

#[cfg(feature = "kanal")]
fn connect() -> (kanal::AsyncSender<Command>, kanal::AsyncReceiver<Command>) {
    kanal::async_bounded::<Command>(CHANNEL_SIZE)
}

#[tokio::main]
async fn main() {
    let (send, mut recv) = connect();

    // Spawn the receiver
    tokio::spawn(async move {
        while let Some(command) = recv.recv().await {
            match command {
                Command::Latency(instant) => {
                    let elapsed = instant.elapsed();
                    LATENCIES.lock().await.push(elapsed.as_nanos());
                }
                Command::Quit => {
                    println!("Quitting.");
                    break;
                }
            }
        }
    });

    // Spawn senders
    let mut handles = Vec::new();
    for _ in 0..TASKS {
        let my_send = send.clone();
        let handle = tokio::spawn(async move {
            for _ in 0..100_000 {
                let _ = my_send.send(Command::Latency(Instant::now())).await.unwrap();
            }
        });
        handles.push(handle);
    }

    // Wait for senders to finish
    for handle in handles {
        handle.await.unwrap();
    }

    // Send quit command
    send.send(Command::Quit).await.unwrap();
    tokio::time::sleep(Duration::from_secs(1)).await;

    let latencies = LATENCIES.lock().await;
    let sum: u128 = latencies.iter().sum();
    let count = latencies.len();
    let average = sum / count as u128;
    println!("Average latency: {} nanos (over {} messages)", average, latencies.len());
}
```

We've done the same trick with feature flags so we can run the same task on either channel type.

Let's look at some results:

Tasks | Bounds | Std Latency (nanos) | Kanal Latency (nanos) | Comment
-|-|-|-|-|
100|1|44,314|30,170|Extreme back-pressure. But it's really not bad now?
100|100|86,266|72,541|High back-pressure
100|1,000|498,710|433,868|Still quite a bit of pressure. Performance is degrading from too large a queue causing heavy allocations.
1,000|1,000|913,614|875,690|We were doing so well, let's try extreme!

So Tokio is a bit better than the standard library, but Kanal can still help if you need it. With timings like these, you can easily build a system with a *lot* of messaging. In fact, that's exactly what Axum and other servers do.
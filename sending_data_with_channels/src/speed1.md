# How Fast is This?

There's some good news and bad news here. Channels are fast---but the standard library channels aren't as fast as they could be.

## Overhead of Sending a Message

Let's create a program to approximately benchmark how long it takes to *send* a message. Sending a message is asynchronous---your thread doesn't block while it waits for the thread to do something.

> The source code is in `channel_code/send_bench`.

```rust
use std::time::Instant;

enum Command {
    Print(String),
    Quit,
}

fn main() {
    let (sender, receiver) = std::sync::mpsc::channel();

    let join_handle = std::thread::spawn(move || loop {
        let command = receiver.recv().unwrap();
        match command {
            Command::Print(_s) => {
                //println!("{}", s)
            }
            Command::Quit => break,
        }
    });

    const NUM_THREADS: usize = 10_000;
    let mut timings_nanos = vec![0; NUM_THREADS];
    for i in 0..NUM_THREADS {
        let s = format!("Hello, world! {}", i);
        let now = Instant::now();
        sender.send(Command::Print(s)).unwrap();
        let elapsed = now.elapsed();
        timings_nanos[i] = elapsed.as_nanos();
    }
    sender.send(Command::Quit).unwrap();

    join_handle.join().unwrap();

    let sum: u128 = timings_nanos.iter().sum();
    let mean = sum as f64 / NUM_THREADS as f64;
    println!("Mean: {} ns", mean);
    println!("Max : {} ns", timings_nanos.iter().max().unwrap());
    println!("Min : {} ns", timings_nanos.iter().min().unwrap());
}
```

On my computer, in `release` mode I get the following timings:

```
Mean: 81.644 ns
Max : 1705 ns
Min : 24 ns
```

So we're burning 81 nanoseconds on average, with a worst-case of 1.7 us and a best-case of 24 nanoseconds. That's not too bad. Your worker thread isn't going to slow down much by sending notifications.

## Sending with Bounded Channels

A pitfall here is that we're using an *unbounded* channel. That is, the channel can grow forever! It is also not pre-allocating. If you keep sending messages faster than you handle them, you will eventually crash with an out of memory error. That's not great.

The standard library also includes `sync_channel` as a channel constructor. This allows you to specify the queue size. You can't add messages forever, but you also now *block* when the channel is full.

Let's try the same example, but with a bounded channel:

```rust
use std::time::Instant;

enum Command {
    Print(String),
    Quit,
}

fn main() {
    const NUM_THREADS: usize = 10_000;
    const CHANNEL_SIZE: usize = 10_000;

    let (sender, receiver) = std::sync::mpsc::sync_channel(CHANNEL_SIZE);

    let join_handle = std::thread::spawn(move || loop {
        let command = receiver.recv().unwrap();
        match command {
            Command::Print(_s) => {
                //println!("{}", s)
            }
            Command::Quit => break,
        }
    });

    let mut timings_nanos = vec![0; NUM_THREADS];
    for i in 0..NUM_THREADS {
        let s = format!("Hello, world! {}", i);
        let now = Instant::now();
        sender.send(Command::Print(s)).unwrap();
        let elapsed = now.elapsed();
        timings_nanos[i] = elapsed.as_nanos();
    }
    sender.send(Command::Quit).unwrap();

    join_handle.join().unwrap();

    let sum: u128 = timings_nanos.iter().sum();
    let mean = sum as f64 / NUM_THREADS as f64;
    println!("Mean: {} ns", mean);
    println!("Max : {} ns", timings_nanos.iter().max().unwrap());
    println!("Min : {} ns", timings_nanos.iter().min().unwrap());
}
```

The only changes here are that we specify a maximum channel size, and use the `sync_channel` constructor instead of the `channel` constructor. The timings are still similar:

```
Mean: 52.2703 ns
Max : 2608 ns
Min : 22 ns
```

What is we change the bounds to be really small? With a channel that can only hold 1 message at a time, everything grinds to a halt:

```
Mean: 3172.9658 ns
Max : 106179 ns
Min : 49 ns
```

> In other words, with great power comes great responsibility: you have to figure out your ideal channel sizes.

## Channel Latency

So how long does it take for a message to arrive inside a channel? Let's find out!

> The source code is in `channel_code/channel_latency`.

```rust
use std::time::Instant;

enum Command {
    Latency(Instant),
    Quit
}

fn main() {
    let (send, recv) = std::sync::mpsc::channel::<Command>();

    let join_handle = std::thread::spawn(move || {
        while let Ok(command) = recv.recv() {
            match command {
                Command::Latency(instant) => {
                    let elapsed = instant.elapsed();
                    println!("Message latency: {} nanos", elapsed.as_nanos());
                }
                Command::Quit => {
                    println!("Quitting.");
                    break;
                }
            }
        }
    });

    for _ in 0..10 {
        let _ = send.send(Command::Latency(Instant::now())).unwrap();
    }
    send.send(Command::Quit).unwrap();

    join_handle.join().unwrap();
}
```

The results are:

```
Message latency: 102472 nanos
Message latency: 107275 nanos
Message latency: 107787 nanos
Message latency: 108134 nanos
Message latency: 108485 nanos
Message latency: 108882 nanos
Message latency: 109223 nanos
Message latency: 109589 nanos
Message latency: 109932 nanos
Message latency: 110271 nanos
Quitting.
```

So about 105,000 nanos (105 us) per message. That's not terrible, but it's not ideal for a really low-latency system. There's also a bit of an impedance mismatch: we can send really, really fast (60 nanos)---but it takes 100 nanos for the message to arrive. So it's easy to send faster than you are receiving---even without any processing. So be careful to bound your channels!

## Messages Per Second

I've not included all the source code for this. Check out [Rust Channel Benchmarks](https://github.com/fereidani/rust-channel-benchmarks).

In Fereidani's benchmarks, he was able to get about 31 million messages per second out of the standard library with empty messages---beating Go's 24 million per second. Sending larger amounts of data is obviously slower, but his "big" test still hits about 23 million messages per second---vs Go's 22 million.

With unbounded channels, the results were a bit worse. 29 million and 11 million messages per second respectively (Go isn't benchmarked, it wisely doesn't offer unbounded channels).

So that's pretty good! But, we're Rustaceans - we can do better!


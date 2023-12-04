# Basic Thread Communication

Let's start with a very simple threaded "hello world":

```rust
use std::thread;

fn main() {
    // Spawn a thread to print a message
    let join_handle = thread::spawn(|| println!("Hello from a thread!"));

    // Print from the main thread
    println!("Hello from the main thread!");

    // Wait for the thread to finish
    join_handle.join().unwrap();
}
```

## Sending Messages the Really Hard Way

What if we want the thread to print a custom message? A really inefficient way to do this is as follows:

> The source code for this is in `channel_code/channel_free_messaging`. I don't recommend ever using this pattern.

```rust
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use std::thread;

static MESSAGE: Mutex<Option<String>> = Mutex::new(None);
static QUITTING: AtomicBool = AtomicBool::new(false);

fn main() {
    let join_handle = thread::spawn(|| loop {
        let mut lock = MESSAGE.lock().unwrap();
        if let Some(message) = &mut *lock {
            println!("{message}");
            *lock = None;
        }
        if QUITTING.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }
    });

    for i in 0..5 {
        {
            let mut lock = MESSAGE.lock().unwrap();
            *lock = Some(format!("Hello {i}"));
        }
        thread::sleep(std::time::Duration::from_millis(500));
    }
    QUITTING.store(true, std::sync::atomic::Ordering::Relaxed);

    join_handle.join().unwrap();
}
```

So this works. But it's really not good:

* You have a lot of complexity.
    * You are storing `QUITTING` as an atomic.
    * You have a `Mutex` around your messages, and have to remember to lock/unlock. Rust makes it hard to forget, but if you don't explicitly drop the lock in the string emitter the thread will *never* see the chain of messages. You could add further complexity by adding a vector...
    * You had to remember to sleep after emitting messages, lest you overwrite the message before the emitter runs. That's not a data-race that Rust helps you with---you aren't doing anything dangerous, but you might scratch your head as to why some messages never print.
* You are spinning your emitter thread at 100% CPU utilization. That's not usually a great plan.
* If you have a lot of these in your program, you are going to find yourself in spaghetti code land.

## Let's use a Channel

> The source code for this is in `channel_code/simple_mpsc_channel`.

```rust
fn main() {
    // Create the channel. I've used the long path here for clarity.    
    let (sender, receiver) = std::sync::mpsc::channel::<String>();

    // Spawn a thread to receive messages
    let join_handle = std::thread::spawn(move || {
        loop {
            // Block until a message is received
            let msg = receiver.recv().unwrap();
            println!("Received: {}", msg);
            if msg == "QUIT" {
                break;
            }
        }
    });

    // Send some messages and then quit
    for i in 0..10 {
        let msg = format!("Message #{}", i);
        sender.send(msg).unwrap();
    }
    sender.send("QUIT".to_string()).unwrap();

    // Wait for the thread to finish
    join_handle.join().unwrap();
}
```

Notice:

* The code is actually slightly longer, mostly because I tried to make it easy to read!
* You don't have any `Atomic` or `Mutex` types. It's all taken care of for you.
* Sending messages is as simple as calling `.send(message)`.
* Receiving messages can loop on `.recv()` and the thread goes to sleep for you---blocks---until it has something to do. You aren't eating 100% of the CPU.
* We decided to emulate Go and crash if the channel is closed by calling `unwrap` on the channel receiver---which will return an error if the channel no longer exists. You could handle this gracefully. Channels will self-close when there are no more transmitters in scope.

## Let's Unleash some Ergonomics

Sending strings around isn't a lot of fun. Sometimes you actually want to print a message, but most of the time you actually want to do something complicated. You may not be happy with a thread that can only do one thing, too! Rust's super-powerful enumeration types are *wonderful* for this type of thing.

> The source code for this is in `channel_code/mpsc_enums`.

```rust
enum Command {
    Print(String),
    Quit,
}

fn main() {
    // Build the channel - this time using enums for commands
    let (sender, receiver) = std::sync::mpsc::channel::<Command>();

    // Spawn a thread to receive messages
    let join_handle = std::thread::spawn(move || {
        loop {
            // Block until a message is received
            let msg = receiver.recv().unwrap();
            match msg {
                Command::Print(msg) => println!("Received: {}", msg),
                Command::Quit => break,
            }
        }
    });

    // Send some messages and then quit
    for i in 0..10 {
        let msg = format!("Message #{}", i);
        sender.send(Command::Print(msg)).unwrap();
    }
    sender.send(Command::Quit).unwrap();

    // Wait for the thread to finish
    join_handle.join().unwrap();
}
```

The code only changed a little, but now we're sending enumerations---giving out explicit instructions and not relying on the contents of the string. This is *really* powerful, because enumerations can hold just about everything. You have a concise, safe mechanism for telling your threads what to do.

Let's mix things up a bit and look at some cool uses for channels.
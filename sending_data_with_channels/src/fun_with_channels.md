# Fun With Channels

Let's look at a couple of things you can do with channels. We'll start by having a bunch of threads sending events into a single channel.

## Lots of Threaded Producers - and still no Mutex in sight

What if we want to send messages from lots of threads?

```rust
enum Command {
    Print(String),
    Quit,
}

fn main() {
    // Build the channel - this time using enums for commands
    let (sender, receiver) = std::sync::mpsc::channel::<Command>();
    let mut join_handles = Vec::new();

    // Send some messages and then quit
    for i in 0..10 {
        // Senders are designed to be cloned.
        let my_sender = sender.clone();
        let join_handle = std::thread::spawn(move || {
            // We're moving `my_sender` into the thread. We can use it
            // here and take ownership of our *clone* - but the main
            // sender remains valid.
            for j in 0..10 {
                let msg = format!("Message from thread #{j} #{i}");
                my_sender.send(Command::Print(msg)).unwrap();
            }
        });
        join_handles.push(join_handle);
    }

    // Spawn a thread to receive messages
    // Note that we're spawning this last: the other threads start,
    // and will be sending messages by the time this thread starts.
    // Channels enqueue messages, so we won't lose any.
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
    join_handles.push(join_handle);
    sender.send(Command::Quit).unwrap();

    // Wait for all threads to finish
    join_handles.into_iter().for_each(|h| h.join().unwrap());
}
```

In this example, we're spawning 10 threads that are each sending 10 messages into the channel. Things to note:

* `sender` is designed to be cloned. It's like `Arc`---it keeps a reference count, and doesn't actually duplicate the sender. All the locking is handled for you.
* We moved the receiver creation to the end so that we wouldn't be closing the receiver before the senders.
* We still don't have a terrible mess of locks and atomics.

## You Can Even Send Functions

You can send *anything* that supports the `Send` trait over a channel. Let's send an arbitrary, user-provided function!

> The source code for this is in `channel_code/channel_functions`.

```rust
enum Command {
    Execute(Box<dyn Send + 'static + FnOnce(i32) -> i32>),
    Quit,
}

fn main() {
    // Build the channel - this time using enums for commands
    let (sender, receiver) = std::sync::mpsc::channel::<Command>();
    let mut join_handles = Vec::new();

    // Spawn a thread to receive messages
    // Note that we're spawning this last: the other threads start,
    // and will be sending messages by the time this thread starts.
    // Channels enqueue messages, so we won't lose any.
    let join_handle = std::thread::spawn(move || {
        loop {
            // Block until a message is received
            let msg = receiver.recv().unwrap();
            match msg {
                Command::Execute(f) => {
                    let result = f(10);
                    println!("Result: {}", result);
                }
                Command::Quit => break,
            }
        }
    });
    join_handles.push(join_handle);

    sender.send(Command::Execute(Box::new(|x| x + 1))).unwrap();
    sender.send(Command::Execute(Box::new(|x| x*2))).unwrap();
    sender.send(Command::Quit).unwrap();

    // Wait for all threads to finish
    join_handles.into_iter().for_each(|h| h.join().unwrap());
}
```

This is a primitive example---we're not really doing anything with the function we're sending. But you really can define a channel type to receive functions and arbitrarily send out functions to be executed on another thread. We've skipped some of the parts that make this useful---using a channel to send the reply somewhere. But this is the basis of a thread-pool. You could even have a channel that enqueues tasks and notifies a pool of workers to do something. Crossbeam does that.

Now imagine trying to do that without channels. It's possible. You are just likely to need a lot of headache medicine!
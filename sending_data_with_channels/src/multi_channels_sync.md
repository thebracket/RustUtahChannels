# Multiple Channels

Sitting and blocking on a single channel may not be what you want---it's very clean, but it's not always the approach you need. What if you want to receive from multiple channels?

Here's a simple example that makes use of `recv_timeout` that can allow you to wait for a maximum amount of time for a message and then do something else if you prefer:

```rust
use std::{sync::mpsc::{Receiver, RecvTimeoutError}, time::Duration};

fn receiver(recv_quit: Receiver<()>, recv_print: Receiver<String>) {
    loop {
        match recv_print.recv_timeout(Duration::from_millis(100)) {
            Ok(message) => println!("{}", message),
            Err(RecvTimeoutError::Timeout) => {
                if recv_quit.try_recv().is_ok() {
                    println!("Quitting.");
                    break;
                }
            }
            Err(_) => {
                println!("The channel was closed. Quitting.");
                break;
            }
        }
    }
}

fn main() {
    let (send_quit, recv_quit) = std::sync::mpsc::channel::<()>();
    let (send_print, recv_print) = std::sync::mpsc::channel::<String>();

    let join_handle = std::thread::spawn(move || {
        receiver(recv_quit, recv_print);
    });

    for i in 0..10 {
        let _ = send_print.send(format!("Message {}", i));
    }
    std::thread::sleep(Duration::from_millis(100));
    send_quit.send(()).unwrap();
    join_handle.join().unwrap();
}
```

You can use `recv_timeout` to wait for a specified period in the hopes that some work arrives. You can use `try_recv` to try and fetch a message, and return an error type if no message is available. By combining these, you can multiplex channels!
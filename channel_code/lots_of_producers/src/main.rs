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
    sender.send(Command::Quit).unwrap();

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

    // Wait for all threads to finish
    join_handles.into_iter().for_each(|h| h.join().unwrap());
}

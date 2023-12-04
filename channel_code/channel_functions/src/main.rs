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

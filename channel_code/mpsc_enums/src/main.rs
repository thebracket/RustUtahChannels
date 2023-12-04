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

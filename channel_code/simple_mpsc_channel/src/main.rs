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
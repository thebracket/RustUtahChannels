use std::time::Instant;

enum Command {
    Latency(Instant),
    Quit
}

fn main() {
    let (send, recv) = kanal::bounded::<Command>(100);

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

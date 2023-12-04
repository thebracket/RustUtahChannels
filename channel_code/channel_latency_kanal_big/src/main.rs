use std::{time::Instant, sync::Mutex};

enum Command {
    Latency(Instant),
    Quit
}

const CHANNEL_SIZE: usize = 1000;
const THREADS: usize = 100;
static LATENCIES: Mutex<Vec<u128>> = Mutex::new(Vec::new());

#[cfg(feature = "kanal")]
fn connect() -> (kanal::Sender<Command>, kanal::Receiver<Command>) {
    kanal::bounded::<Command>(CHANNEL_SIZE)
}

#[cfg(not(feature = "kanal"))]
fn connect() -> (std::sync::mpsc::SyncSender<Command>, std::sync::mpsc::Receiver<Command>) {
    std::sync::mpsc::sync_channel::<Command>(CHANNEL_SIZE)
}

fn main() {
    let (send, recv) = connect();

    let join_handle = std::thread::spawn(move || {
        while let Ok(command) = recv.recv() {
            match command {
                Command::Latency(instant) => {
                    let elapsed = instant.elapsed();
                    LATENCIES.lock().unwrap().push(elapsed.as_nanos());
                }
                Command::Quit => {
                    println!("Quitting.");
                    break;
                }
            }
        }
    });

    let mut handles = Vec::new();
    for _ in 0..THREADS {
        let my_send = send.clone();
        handles.push(std::thread::spawn(move || {
            for _ in 0..100_000 {
                let _ = my_send.send(Command::Latency(Instant::now())).unwrap();
            }
        }));
    }
    handles.into_iter().for_each(|handle| handle.join().unwrap());
    send.send(Command::Quit).unwrap();

    join_handle.join().unwrap();

    let latencies = LATENCIES.lock().unwrap();
    let sum: u128 = latencies.iter().sum();
    let count = latencies.len();
    let average = sum / count as u128;
    println!("Average latency: {} nanos", average);

}

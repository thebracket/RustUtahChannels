use std::time::{Instant, Duration};
use once_cell::sync::Lazy;
use tokio::sync::Mutex;

const CHANNEL_SIZE: usize = 1000;
const TASKS: usize = 100;
static LATENCIES: Lazy<Mutex<Vec<u128>>> = Lazy::new(|| Mutex::new(Vec::new()));

enum Command {
    Latency(Instant),
    Quit
}

#[cfg(not(feature = "kanal"))]
fn connect() -> (tokio::sync::mpsc::Sender<Command>, tokio::sync::mpsc::Receiver<Command>) {
    tokio::sync::mpsc::channel::<Command>(CHANNEL_SIZE)
}

#[cfg(feature = "kanal")]
fn connect() -> (kanal::AsyncSender<Command>, kanal::AsyncReceiver<Command>) {
    kanal::bounded_async::<Command>(CHANNEL_SIZE)
}

#[tokio::main]
async fn main() {
    let (send, mut recv) = connect();

    // Spawn the receiver
    tokio::spawn(async move {
        #[cfg(not(feature = "kanal"))]
        while let Some(command) = recv.recv().await {
            match command {
                Command::Latency(instant) => {
                    let elapsed = instant.elapsed();
                    LATENCIES.lock().await.push(elapsed.as_nanos());
                }
                Command::Quit => {
                    println!("Quitting.");
                    break;
                }
            }
        }

        #[cfg(feature = "kanal")]
        while let Ok(command) = recv.recv().await {
            match command {
                Command::Latency(instant) => {
                    let elapsed = instant.elapsed();
                    LATENCIES.lock().await.push(elapsed.as_nanos());
                }
                Command::Quit => {
                    println!("Quitting.");
                    break;
                }
            }
        }
    });

    // Spawn senders
    let mut handles = Vec::new();
    for _ in 0..TASKS {
        let my_send = send.clone();
        let handle = tokio::spawn(async move {
            for _ in 0..100_000 {
                let _ = my_send.send(Command::Latency(Instant::now())).await.unwrap();
            }
        });
        handles.push(handle);
    }

    // Wait for senders to finish
    for handle in handles {
        handle.await.unwrap();
    }

    // Send quit command
    send.send(Command::Quit).await.unwrap();
    tokio::time::sleep(Duration::from_secs(1)).await;

    let latencies = LATENCIES.lock().await;
    let sum: u128 = latencies.iter().sum();
    let count = latencies.len();
    let average = sum / count as u128;
    println!("Average latency: {} nanos (over {} messages)", average, latencies.len());
}

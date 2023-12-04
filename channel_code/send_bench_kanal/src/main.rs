use std::time::Instant;

enum Command {
    Print(String),
    Quit,
}

fn main() {
    let (sender, receiver) = kanal::bounded(10_000);

    let join_handle = std::thread::spawn(move || loop {
        let command = receiver.recv().unwrap();
        match command {
            Command::Print(_s) => {
                //println!("{}", s)
            }
            Command::Quit => break,
        }
    });

    const NUM_THREADS: usize = 10_000;
    let mut timings_nanos = vec![0; NUM_THREADS];
    for i in 0..NUM_THREADS {
        let s = format!("Hello, world! {}", i);
        let now = Instant::now();
        sender.send(Command::Print(s)).unwrap();
        let elapsed = now.elapsed();
        timings_nanos[i] = elapsed.as_nanos();
    }
    sender.send(Command::Quit).unwrap();

    join_handle.join().unwrap();

    let sum: u128 = timings_nanos.iter().sum();
    let mean = sum as f64 / NUM_THREADS as f64;
    println!("Mean: {} ns", mean);
    println!("Max : {} ns", timings_nanos.iter().max().unwrap());
    println!("Min : {} ns", timings_nanos.iter().min().unwrap());
}

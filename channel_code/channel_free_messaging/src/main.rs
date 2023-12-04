use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use std::thread;

static MESSAGE: Mutex<Option<String>> = Mutex::new(None);
static QUITTING: AtomicBool = AtomicBool::new(false);

fn main() {
    let join_handle = thread::spawn(|| loop {
        let mut lock = MESSAGE.lock().unwrap();
        if let Some(message) = &mut *lock {
            println!("{message}");
            *lock = None;
        }
        if QUITTING.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }
    });

    for i in 0..5 {
        {
            let mut lock = MESSAGE.lock().unwrap();
            *lock = Some(format!("Hello {i}"));
        }
        thread::sleep(std::time::Duration::from_millis(500));
    }
    QUITTING.store(true, std::sync::atomic::Ordering::Relaxed);

    join_handle.join().unwrap();
}

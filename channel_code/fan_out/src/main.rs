use std::{time::Duration, sync::{Arc, Mutex}};

#[derive(Debug)]
struct ImportantData {
    value: i32,
    another_value: i32,
}

impl Drop for ImportantData {
    fn drop(&mut self) {
        println!("Dropping ImportantData with value: {} and another_value: {}", self.value, self.another_value);
    }
}

type ImportantDataPtr = Arc<Mutex<ImportantData>>;

fn main() {
    let (tx, rx) = std::sync::mpsc::channel::<ImportantDataPtr>();
    let (tx2, rx2) = std::sync::mpsc::channel::<ImportantDataPtr>();

    std::thread::spawn(move || {
        while let Ok(data) = rx.recv() {
            let mut lock = data.lock().unwrap();
            lock.value += 1;
        }
    });

    std::thread::spawn(move || {
        while let Ok(data) = rx2.recv() {
            let mut lock = data.lock().unwrap();
            lock.another_value += 1;
        }
    });

    let my_data = Arc::new(Mutex::new(ImportantData { value: 0, another_value: 0 }));
    tx.send(my_data.clone()).unwrap();
    tx2.send(my_data.clone()).unwrap();
    std::thread::sleep(Duration::from_secs(1));
    println!("{:?}", my_data);
}
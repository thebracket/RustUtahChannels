# Fan-Out Processing

If you have a high "fan out" pattern---a process receives data, and then spits out *many* different tasks that use part of the data, then Rust's ownership model can feel a little painful.

The solution is often to combine `Arc` with channels---and tasks if you are in `async` mode.

Let's look at an example:

> The code for this is in `channel_code/fan_out`.

```rust
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
```

The takeaway here is that you should use an `Arc` to make your data sendable, and synchronize---in this case with `Mutex`. We tracked `Drop` so that you can see that the data isn't disposed until every task is done with it. We have garbage collection, we just prefer it to be deterministic!

The other big benefit is that you are transmitting the size of a pointer rather than all the data.

This is a trivial example. If you're dealing with real data, you could:

* Wrap individual members in their own synchronization primitives to allow for zero contention while different threads work on different parts of the problem.
* Attach a "save" event or a shared-state progress meter to know when every task has done.

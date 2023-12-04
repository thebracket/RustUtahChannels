# Mixing Async and Sync

It often feels like the Sync and Async people in the Rust world don't talk to one another. This is a shame, because sometimes you can achieve amazing performance by mixing your metaphors---and not resorting to microservices.

Say you have a task that is mostly CPU bound. So you allocate a bunch of threads to the processing. But you *also* want high performance IO with minimal blocking penalty while you wait for data to come in and out. You're even really clever, and remember to limit Tokio to the appropriate number of threads.

Rust can do this, and it's surprisingly easy.

> The code for this is in `channel_code/mixed_sync_async`

```rust
#[tokio::main]
async fn main() {
    let (async_tx, mut async_rx) = tokio::sync::mpsc::channel::<i32>(1);
    let (sync_tx, sync_rx) = std::sync::mpsc::channel::<i32>();

    // Spawn a THREAD to receive sync messages, and send them to the async channel
    std::thread::spawn(move || {
        while let Ok(msg) = sync_rx.recv() {
            println!("Received sync message: {}", msg);
            async_tx.blocking_send(msg).unwrap();
        }
    });

    // Spawn a TASK to receive async messages
    tokio::spawn(async move {
        while let Some(msg) = async_rx.recv().await {
            println!("Received async message: {}", msg);
        }
    });

    // Send a message to the sync channel
    sync_tx.send(42).unwrap();

    // Sleep for a second to allow the async task to receive the message
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
}
```
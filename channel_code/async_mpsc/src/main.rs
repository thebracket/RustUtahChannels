enum Command {
    Print(String),
    Quit,
}

#[tokio::main]
async fn main() {
    let (sender, mut receiver) = tokio::sync::mpsc::channel::<Command>(100);

    tokio::spawn(async move {
        loop {
            while let Some(command) = receiver.recv().await {
                match command {
                    Command::Print(s) => println!("{}", s),
                    Command::Quit => break,
                }
            }
        }
    });

    for i in 0..10 {
        let sender = sender.clone();
        tokio::spawn(async move {
            sender
                .send(Command::Print(format!("Hello, world! {}", i)))
                .await
                .unwrap();
        });
    }
    sender.send(Command::Quit).await.unwrap();
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
}

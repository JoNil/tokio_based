BASED: BASED Async Single-threaded Execution Dispatcher

The purpose of tokio_based is to allow you to easily spin up a single threaded tokio runtime. That will be shut down and all tasks dropped when the join handle is dropped.

```rust
struct ExampleServer {
    join_handle: tokio_based::JoinHandle,
}

impl ExampleServer {
    fn new() -> ExampleServer {
        let join_handle = tokio_based::spawn(|run| async move {
            let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

            while run.load(Ordering::SeqCst) {
                let (mut socket, _) = listener.accept().await.unwrap();

                tokio::spawn({
                    let run = run.clone();
                    async move {
                        let mut buf = [0; 1024];

                        // In a loop, read data from the socket and write the data back.
                        while run.load(Ordering::SeqCst) {
                            let n = match socket.read(&mut buf).await {
                                // socket closed
                                Ok(0) => return,
                                Ok(n) => n,
                                Err(e) => {
                                    eprintln!("failed to read from socket; err = {:?}", e);
                                    return;
                                }
                            };

                            // Write the data back
                            if let Err(e) = socket.write_all(&buf[0..n]).await {
                                eprintln!("failed to write to socket; err = {:?}", e);
                                return;
                            }
                        }
                    }
                });
            }
        });

        ExampleServer { join_handle }
    }
}

```

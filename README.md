BASED: BASED Async Single-threaded Execution Dispatcher

```rust
pub struct A {
    based: Based,
}

impl A {
    pub fn new() -> A {
        let based = Based::new(|run| async move {

            // Some async operation

            while run.load(Ordering::SeqCst) {
                // That does not return
            }
        });

        A { based }
    }
}
```

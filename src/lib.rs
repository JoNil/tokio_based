use std::{
    future::Future,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
};
use tokio::sync::oneshot;

struct Based {
    run: Arc<AtomicBool>,
    runtime: Option<(oneshot::Sender<()>, JoinHandle<()>)>,
}

impl Based {
    pub fn new<
        Fut: Future<Output = ()> + Send + 'static,
        F: FnOnce(Arc<AtomicBool>) -> Fut + Send + 'static,
    >(
        fut: F,
    ) -> Self {
        let (exit_sender, exit_receiver) = oneshot::channel();
        let run = Arc::new(AtomicBool::new(true));

        let runtime = Some((
            exit_sender,
            thread::spawn({
                let run = run.clone();
                move || {
                    let runtime = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .unwrap();

                    runtime.spawn(async move {
                        let run = run.clone();
                        fut(run).await;
                    });

                    runtime.block_on(async move {
                        let _ = exit_receiver.await;
                    });
                }
            }),
        ));

        Self { run, runtime }
    }
}

impl Drop for Based {
    fn drop(&mut self) {
        self.run.store(false, Ordering::SeqCst);
        if let Some((exit_sender, thread)) = self.runtime.take() {
            exit_sender.send(()).ok();
            thread.join().ok();
        }
    }
}

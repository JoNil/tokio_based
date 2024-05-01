//! # BASED: BASED Async Single-threaded Execution Dispatcher
//!
//! The purpose of tokio_based is to allow you to easily spin up a single threaded tokio runtime. That will be shut down and all tasks dropped when the join handle is dropped.
//!
use std::{
    future::Future,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};
use tokio::sync::oneshot;

/// When dropped the runtime will be shutdown and all tasks dropped
pub struct JoinHandle {
    run: Arc<AtomicBool>,
    runtime: Option<(oneshot::Sender<()>, thread::JoinHandle<()>)>,
}

/// Spawn takes a closure with a run parameter and runs the future returned by the closure
///
/// Example
/// ```
/// let join_handle = tokio_based::spawn(|run| async move {
///     // Do something in the async runtime
/// });
/// ```
pub fn spawn<
    Fut: Future<Output = ()> + Send + 'static,
    F: FnOnce(Arc<AtomicBool>) -> Fut + Send + 'static,
>(
    fut: F,
) -> JoinHandle {
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

    JoinHandle { run, runtime }
}

impl Drop for JoinHandle {
    fn drop(&mut self) {
        self.run.store(false, Ordering::SeqCst);
        if let Some((exit_sender, thread)) = self.runtime.take() {
            exit_sender.send(()).ok();
            thread.join().ok();
        }
    }
}

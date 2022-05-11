use std::thread;

/// A task is a long-running activity meant to run on another thread.
pub trait Task {
    type Result;
    type OnCompleteParams;
    fn run_task(self) -> anyhow::Result<Self::Result>;
    fn on_complete(r: anyhow::Result<Self::Result>, p: Self::OnCompleteParams);
    fn spawn(self, p: Self::OnCompleteParams)
    where
        Self: Sized + Send + 'static,
        Self::OnCompleteParams: Sized + Send + 'static,
    {
        thread::spawn(move || Self::on_complete(self.run_task(), p));
    }
}

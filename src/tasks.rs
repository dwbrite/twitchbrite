use std::thread;

/// A task is a long-running activity meant to run on another thread.
pub trait Task {
    type Result;
    type Parameters;
    fn run_task(&mut self) -> anyhow::Result<Self::Result>;
    fn handle_result(r: anyhow::Result<Self::Result>, p: Self::Parameters);
    fn spawn_task(mut self, p: Self::Parameters)
    where
        Self: Sized + Send + 'static,
        Self::Parameters: Sized + Send + 'static,
    {
        thread::spawn(move || Self::handle_result(self.run_task(), p));
    }
}

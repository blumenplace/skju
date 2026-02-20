pub trait Timer {
    fn wait_ms(&mut self, ms: u64) -> impl Future<Output = ()>;
}

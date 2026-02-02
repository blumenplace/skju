use embassy_time::Timer as EmbassyTimer;
use skju_peripherals::timer::Timer;

pub struct TimerHandler;

impl Timer for TimerHandler {
    async fn wait_ms(&mut self, ms: u64) {
        EmbassyTimer::after_millis(ms).await;
    }
}

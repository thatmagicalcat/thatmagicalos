use core::{
    pin::Pin,
    sync::atomic::{AtomicU64, Ordering},
    task::{Context, Poll, Waker},
    time::Duration,
};

use alloc::{collections::BTreeMap, vec::Vec};
use futures_util::task::AtomicWaker;
use spin::Mutex;

use crate::{apic, hpet::HPET};

type Nanoseconds = u64;

pub static TIMERS: Mutex<BTreeMap<Nanoseconds, Vec<Waker>>> = Mutex::new(BTreeMap::new());
pub static TIMER_WAKER: AtomicWaker = AtomicWaker::new();

fn get_min_timestamp() -> Option<u64> {
    TIMERS.lock().first_key_value().map(|(tick, _)| *tick)
}

fn duration_to_ticks(
    duration: Duration,
    freq_hz: u64,
) -> Result<u32, <u128 as TryInto<u32>>::Error> {
    let nanos = duration.as_nanos(); // Returns u128
    let ticks = (nanos * freq_hz as u128) / 1_000_000_000;

    ticks.try_into()
}

fn set_timer(ticks: u64) {
    apic::set_timer(
        apic::DivideConfig::DIVIDE_BY_1,
        ticks as u32,
        apic::LvtTimerMode::ONESHOT,
    );
}

fn stop_timer() {
    apic::set_timer(
        apic::DivideConfig::DIVIDE_BY_1,
        0,
        apic::LvtTimerMode::ONESHOT,
    );
}

/// There should only exist a single instance of this struct at a time
/// which is awaited, otherwise, the LAPIC timer will be reconfigured
/// by each instance which may lead to unexpected behavior
struct WaitForApic {
    target_nanos: u64,
    timer_started: bool,
}

impl WaitForApic {
    const fn new(target_nanos: u64) -> Self {
        Self {
            target_nanos,
            timer_started: false,
        }
    }
}

impl Future for WaitForApic {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let current_timestamp = HPET.get().unwrap().uptime_nanos();

        // fast path
        if self.target_nanos != 0 && current_timestamp >= self.target_nanos {
            return Poll::Ready(());
        }

        TIMER_WAKER.register(cx.waker());

        if !self.timer_started {
            self.as_mut().timer_started = true;

            if self.target_nanos == 0 {
                log::trace!("No timers registered, im gonna sleep... waiting for timer task to wake me");
                return Poll::Pending;
            }

            let ticks = duration_to_ticks(
                Duration::from_nanos(self.target_nanos - current_timestamp),
                apic::get_timer_frequency(),
            )
            .expect("Duration too long to convert to ticks") as _;

            set_timer(ticks);

            if ticks == 0 {
                TIMER_WAKER.wake();
            }

            return Poll::Pending;
        }

        // polled from the Sleep future after the timer has been set
        // so we stop the timer
        stop_timer();
        Poll::Ready(())
    }
}

pub async fn sleep(duration: Duration) {
    Sleep::new(duration).await
}

struct Sleep {
    timestamp_nanos: u64,
    registered: bool,
}

impl Sleep {
    fn new(duration: Duration) -> Self {
        Self {
            timestamp_nanos: HPET.get().unwrap().uptime_nanos() + duration.as_nanos() as u64,
            registered: false,
        }
    }
}

impl Future for Sleep {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let current_timestamp = HPET.get().unwrap().uptime_nanos();
        if current_timestamp >= self.timestamp_nanos {
            return Poll::Ready(());
        }

        if !self.registered {
            let is_earliest = TIMERS
                .lock()
                .first_key_value()
                .map(|(earliest_timestamp, _)| *earliest_timestamp > self.timestamp_nanos)
                .unwrap_or(true); // only timer, so it's the earliest

            TIMERS
                .lock()
                .entry(self.timestamp_nanos)
                .or_default()
                .push(cx.waker().clone());
            self.as_mut().registered = true;

            if is_earliest {
                TIMER_WAKER.wake();
            }
        }

        Poll::Pending
    }
}

pub async fn timer_dispatch() {
    log::info!("Starting timer dispatch loop");

    loop {
        // there is a chance that a significant amount of time has passed while running the wakers
        loop {
            let current_timestamp = HPET.get().unwrap().uptime_nanos();
            let mut timers = TIMERS.lock();

            if let Some((&next_timestamp, _)) = timers.first_key_value()
                && current_timestamp >= next_timestamp
            {
                let (_, wakers) = timers.pop_first().unwrap();
                drop(timers);

                for waker in wakers {
                    waker.wake();
                }

                continue;
            }

            break; // no more expired timers
        }

        let next_timestamp = get_min_timestamp().unwrap_or(0);
        WaitForApic {
            target_nanos: next_timestamp,
            timer_started: false,
        }
        .await;
    }
}

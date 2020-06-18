use std::mem;
use std::sync::{Arc, Mutex, Once};
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct StartTime {
    start: Arc<Mutex<Option<Instant>>>,
}

impl StartTime {
    pub(crate) fn singleton() -> StartTime {
        // Initialize it to a null value
        static mut START_TIME_SINGLETON: *const StartTime = 0 as *const StartTime;
        static START_TIME_ONCE: Once = Once::new();

        unsafe {
            START_TIME_ONCE.call_once(|| {
                // Make it
                let singleton = StartTime {
                    start: Arc::new(Mutex::new(None)),
                };

                // Put it in the heap so it can outlive this call
                START_TIME_SINGLETON = mem::transmute(Box::new(singleton));
            });

            // Now we give out a copy of the data that is safe to use concurrently.
            (*START_TIME_SINGLETON).clone()
        }
    }

    pub fn get(&self) -> bool {
        let running = self.start.lock().unwrap();
        let data = running.clone();
        match data {
            None => false,
            Some(v) => {
                if Instant::now().duration_since(v) >= Duration::from_secs(3) {
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn set(&mut self, time: Instant) {
        let mut state = self.start.lock().expect("Could not lock mutex");
        mem::replace(&mut *state, Some(time.clone()));
    }
}

use std::mem;
use std::sync::{Arc, Mutex, Once};

#[derive(Clone, Debug)]
pub struct AnalyzerState {
    running: Arc<Mutex<bool>>,
}

impl AnalyzerState {
    pub(crate) fn singleton() -> Self {
        // Initialize it to a null value
        static mut ANALYZER_STATE_SINGLETON: *const AnalyzerState = 0 as *const AnalyzerState;
        static ANALYZER_STATE_ONCE: Once = Once::new();

        unsafe {
            ANALYZER_STATE_ONCE.call_once(|| {
                // Make it
                let singleton = AnalyzerState {
                    running: Arc::new(Mutex::new(false)),
                };

                // Put it in the heap so it can outlive this call
                ANALYZER_STATE_SINGLETON = mem::transmute(Box::new(singleton));
            });

            // Now we give out a copy of the data that is safe to use concurrently.
            (*ANALYZER_STATE_SINGLETON).clone()
        }
    }

    pub fn get(&self) -> bool {
        let running = self.running.lock().unwrap();
        let data = running.clone();
        data
    }

    pub fn set(&mut self, new_value: bool) {
        let mut state = self.running.lock().expect("Could not lock mutex");
        mem::replace(&mut *state, new_value.clone());
    }
}

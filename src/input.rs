

use std::time::{Duration, Instant};
#[derive(PartialEq, Eq, Clone, Copy,Debug)]
enum KeyState {NPRESS, PRESS, DAS}

#[derive(Clone, Copy)]
struct MotionConfig {
    das_delay: Duration,
    arr: Duration,
}


#[derive(Clone, Copy)]
pub struct MotionState {
    key_state : KeyState,
    last_time : Option<Instant>,
    config: MotionConfig,
}

impl MotionState {
    pub fn new(das_delay:u64, arr:u64) -> Self {
        Self {
            key_state : KeyState::NPRESS,
            last_time: None,
            config: MotionConfig {das_delay:Duration::from_millis(das_delay), arr:Duration::from_millis(arr)}
        }
    }
    pub fn reset_last(&mut self, time:Instant) {
        self.last_time = Some(time);
    }

    pub fn update(&mut self, is_pressed:bool, current_time: Instant) -> bool
    {
        if !is_pressed {
            self.key_state = KeyState::NPRESS;
            self.last_time = None;
            return false;
        }

        match self.key_state {
            KeyState::NPRESS => {
                self.key_state = KeyState::PRESS;
                self.last_time = Some(current_time);
                true
            },
            KeyState::PRESS => {
                if let Some(last_time) = self.last_time {
                    if current_time.duration_since(last_time) > self.config.das_delay {
                        self.last_time = Some(current_time);
                        self.key_state = KeyState::DAS;
                        return true;
                    }
                }
                false
            },
            KeyState::DAS => {
                if let Some(last_time) = self.last_time {
                    if current_time.duration_since(last_time) > self.config.arr {
                        self.last_time = Some(current_time);
                        return true;
                    }
                }
                false
            }}
    }
}

pub struct ConstMotion {
    impl_motion : MotionState,
}

impl ConstMotion {
    pub fn new(delay:u64) -> Self {
        Self {
            impl_motion: MotionState { key_state: KeyState::PRESS, last_time: None, config: MotionConfig {das_delay: Duration::from_millis(delay), arr:Duration::from_millis(delay)}}
        }
    }
    pub fn reset(&mut self, time:Instant) {
        self.impl_motion.reset_last(time);
    }
    pub fn update(&mut self, current_time: Instant) -> bool {
        self.impl_motion.update(true, current_time)
    }
}

pub struct LockMgr {
    lock_delay: Duration,
    lock_start_time: Option<Instant>,
}

impl LockMgr {
    pub fn new(delay:u64) -> Self {
        Self {
            lock_delay: Duration::from_millis(delay),
            lock_start_time:None
        }
    }
    pub fn reset(&mut self) {
        self.lock_start_time = None;
    }

    pub fn start_if_not(&mut self, now:Instant) -> Option<Instant> {
        self.lock_start_time = self.lock_start_time.or(Some(now));
        self.lock_start_time
    }

    pub fn lock(&self, now:Instant) -> bool 
    {
        if let Some(start) = self.lock_start_time {
            if now.duration_since(start) > self.lock_delay {
                return true;
            }
        }
        return false;
    }
}

// --- 單元測試區塊 ---
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Instant, Duration};

    #[test]
    fn test_das_trigger_logic() {
        let mut motion = MotionState::new(100, 50);
        let start_time = Instant::now();

        // first press, should trigger
        assert!(motion.update(true, start_time));
        assert_eq!(motion.key_state, KeyState::PRESS);
        assert_eq!(motion.last_time, Some(start_time));

        // keep press, 50 < 100, do not trigger
        let time_50ms = start_time + Duration::from_millis(50);
        assert!(!motion.update(true, time_50ms));
        assert_eq!(motion.last_time, Some(start_time));

        // keep press, 101 > 100, triggter and set to DAS mode
        let time_101ms = start_time + Duration::from_millis(101);
        assert!(motion.update(true, time_101ms));
        assert_eq!(motion.key_state, KeyState::DAS);
        assert_eq!(motion.last_time, Some(time_101ms));

        // keep press, 51 > 50(ARR delay)
        let time_152ms = time_101ms + Duration::from_millis(51);
        assert!(motion.update(true, time_152ms));
        assert_eq!(motion.key_state, KeyState::DAS);
        assert_eq!(motion.last_time, Some(time_152ms));

    }

    #[test]
    fn test_release_reset() {
        let mut motion = MotionState::new(100, 50);
        let now = Instant::now();
        motion.update(true, now);
        motion.update(false, now);
        assert_eq!(motion.key_state, KeyState::NPRESS);
        assert!(motion.last_time.is_none());
    }
}
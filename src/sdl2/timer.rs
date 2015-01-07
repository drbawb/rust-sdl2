use std::mem;
use std::raw;
use std::thunk::Thunk;
use libc::{uint32_t, c_void};
#[cfg(test)] use std::sync::{Arc, Mutex};

pub use sys::timer as ll;

pub fn get_ticks() -> uint {
    unsafe { ll::SDL_GetTicks() as uint }
}

pub fn get_performance_counter() -> u64 {
    unsafe { ll::SDL_GetPerformanceCounter() }
}

pub fn get_performance_frequency() -> u64 {
    unsafe { ll::SDL_GetPerformanceFrequency() }
}

pub fn delay(ms: uint) {
    unsafe { ll::SDL_Delay(ms as u32) }
}

/// Represents a repeatable timer callback which owns its environment
pub type TimerThunk = Option<Thunk<(), NextInterval>>;

/// The next state for a timer:
///    * `uint` milliseconds duration to the next tick
///    * `TimerThunk` an invokable timer...
pub struct NextInterval((uint, TimerThunk));

pub struct Timer<'a> {
    delay: uint,
    raw: ll::SDL_TimerID,
    closure: TimerThunk,
}

impl<'a> Timer<'a> {
    pub fn new(delay: uint, callback: TimerThunk) -> Timer<'a> {
        Timer { delay: delay, raw: 0, closure: callback }
    }

    pub fn start(&mut self) {
        unsafe {
            let timer_id = ll::SDL_AddTimer(
                    self.delay as u32, 
                    Some(c_timer_callback as
                        extern "C" fn (
                            _interval: uint32_t, 
                            param: *const c_void
                        ) -> uint32_t), 
                    mem::transmute(&self.closure)
                );
            self.raw = timer_id;
        }
    }

    pub fn remove(&mut self) -> bool {
        let ret = unsafe { ll::SDL_RemoveTimer(self.raw) };
        if self.raw != 0 {
            self.raw = 0
        }
        ret == 1
    }
}

#[unsafe_destructor]
impl<'a> Drop for Timer<'a> {
    fn drop(&mut self) {
        let ret = unsafe { ll::SDL_RemoveTimer(self.raw) };
        if ret != 1 {
            println!("error dropping timer {}, maybe already removed.", self.raw);
        }
    }
}

extern "C" fn c_timer_callback(_interval: uint32_t, param: *const c_void) -> uint32_t {
    let timer_cb: &mut TimerThunk = unsafe { mem::transmute(param) };
   
    match timer_cb.take() {
        Some(f) => {
            let NextInterval((time, next_f)) = f.invoke(());
            *timer_cb = next_f;

            time as uint32_t
        },

        None => unreachable!("um..."),
    }
}

#[cfg(test)]
/// A recursive, capture-by-move periodic timer...
///
/// Wrapped in an `Option<T>` so that it can be taken by value
/// in the C callback...
fn gen_timer(st: Arc<Mutex<u32>>) -> TimerThunk {
    Some(Thunk::new(move|| {
        { // borrow `st
            let mut num = st.lock().unwrap();
            *num += 1;
        }

        NextInterval((10, gen_timer(st)))
    }))
}

#[test]
fn test_timer_1() {
    let local_num: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
    let timer_num = local_num.clone();
    {
        let mut timer = Timer::new(10, gen_timer(timer_num));
        timer.start();
        delay(100);
        let num = local_num.lock().unwrap();
        assert!(*num == 9);
    }

    // Check that timer has stopped
    delay(100);
    let num = local_num.lock().unwrap();
    assert!(*num == 9);
}


#[cfg(test)]
fn gen_timer_2() -> TimerThunk {
    Some(Thunk::new(move|| {
        NextInterval( (0, gen_timer_2()) )
    }))
}

#[test]
fn test_timer_2() {
    // Check that the closure lives long enough outside the block where
    // the timer was started.
    let _ = {
        let mut timer = Timer::new(1000, gen_timer_2());
        timer.start();
        timer
    };
    delay(200);
    delay(200);
    delay(200);
    delay(200);
    delay(200);
    delay(200);
}


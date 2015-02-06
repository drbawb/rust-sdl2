use libc::{uint32_t, c_void};
use std::mem;
use sys::timer as ll;

pub fn get_ticks() -> u32 {
    unsafe { ll::SDL_GetTicks() }
}

pub fn get_performance_counter() -> u64 {
    unsafe { ll::SDL_GetPerformanceCounter() }
}

pub fn get_performance_frequency() -> u64 {
    unsafe { ll::SDL_GetPerformanceFrequency() }
}

pub fn delay(ms: u32) {
    unsafe { ll::SDL_Delay(ms) }
}

#[unstable = "Unstable because of move to unboxed closures and `box` syntax"]
pub struct Timer<'a> {
    _delay: u32,
    raw: ll::SDL_TimerID,
}

impl<'a> Timer<'a> {
    /// Constructs a new timer using the boxed closure `callback`.
    /// The timer is started immediately, it will be cancelled either:
    ///   * when the timer is dropped
    ///   * or when the callback returns a non-positive continuation interval
    pub fn new(delay: u32, callback: &'a &Fn() -> u32) -> Timer<'a> {
        unsafe {
            let timer_id = ll::SDL_AddTimer(delay,
                                            Some(c_timer_callback),
                                            mem::transmute_copy(&callback));

            Timer {
                _delay: delay,
                raw: timer_id,
            }
        }
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

extern "C" fn c_timer_callback(_interval: u32, param: *const c_void) -> uint32_t {
    unsafe {
        let f: &&Fn() -> u32 = mem::transmute(param);
        (*f)() as uint32_t
    }
}

#[test]
fn test_timer_runs_multiple_times() {
    use std::sync::{Arc, Mutex};

    let local_num = Arc::new(Mutex::new(0));
    let timer_num = local_num.clone();

	let closure = move|| {
        // increment up to 10 times (0 -> 9)
        // tick again in 100ms after each increment
        //
        let mut num = timer_num.lock().unwrap();
        if *num < 9 {
            *num += 1;
            100
        } else { 0 }
    };

    let timer = Timer::new(100, &(&closure as &Fn() -> u32));

    delay(1200);                         // tick the timer at least 10 times w/ 200ms of "buffer"
    let num = local_num.lock().unwrap(); // read the number back
    assert_eq!(*num, 9);                 // it should have incremented at least 10 times...
}

#[test]
fn test_timer_runs_at_least_once() {
    use std::sync::{Arc, Mutex};

    let local_flag = Arc::new(Mutex::new(false));
    let timer_flag = local_flag.clone();

	let closure = || {
        let mut flag = timer_flag.lock().unwrap();
        *flag = true; 0
    };

    let timer = Timer::new(500, &(&closure  as &Fn() -> u32));

    delay(700);
    let flag = local_flag.lock().unwrap();
    assert_eq!(*flag, true);
}

#[cfg(test)]
mod wat {
	use super::{delay, Timer};

	fn foo<'a>() -> Timer<'a> {
		let x = 100;
		let closure = || { x };
		Timer::new(100, &(&closure as & Fn() -> u32))
	}

	#[test]
	fn wat_test() {
		let timer = foo();
		delay(300);
		assert!(true);
	}
}

/*#[test]
fn test_timer_can_be_recreated() {
    use std::sync::{Arc, Mutex};

    let local_num = Arc::new(Mutex::new(0));
    let timer_num = local_num.clone();

    // run the timer once and reclaim its closure
    let mut timer_1 = Timer::new(100, Box::new(move|| {
        let mut num = timer_num.lock().unwrap();
        *num += 1; // increment the number
        0          // do not run timer again
    }));

    // reclaim closure after timer runs
    delay(200);
    let closure = timer_1.into_inner();

    // create a second timer and increment again
    let timer_2 = Timer::new(100, closure);
    delay(200);

    // check that timer was incremented twice
    let num = local_num.lock().unwrap();
    assert_eq!(*num, 2);
}*/


use std::time::Instant;

pub struct Utils;

impl Utils {
    pub fn log_time(from: Instant, msg: &str) {
        let elapse_time = from.elapsed();
        println!("[Execution time] {} took - {:?}", msg, elapse_time);
    }
}

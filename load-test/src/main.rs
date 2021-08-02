use std::fmt;
use std::env;

struct UsageError {
    message: String,
}

impl fmt::Debug for UsageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "load-test: {}", self.message)
    }
}

fn main() -> Result<(), UsageError> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("usage: load-test url");
        return Err(UsageError { message: "not enough arguments".to_string() });
    } else {
        thread_loop(&args[1].to_owned());
    }
    Ok(())
}

fn thread_loop(hostname: &str) {
    let mut thread_local_request_counter = 0;
    loop {
        // Intentionally don't use an Agent; make as many separate
        // connections as possible.
        let _ = ureq::get(hostname).call();
        thread_local_request_counter += 1;
        if thread_local_request_counter % 100 == 0 {
            println!("made {} requests so far", thread_local_request_counter);
        }
    }
}

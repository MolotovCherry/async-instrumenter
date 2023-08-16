# Async Instrumenter

This Rust library offers a useful future which allows you to time exactly how long any particular future took to execute.

Usage couldn't be simpler:

```rs
async fn sleep() {
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
}

// debug log our `sleep()` future with a custom log message
// will only print debug log if `debug_assertions` are enabled
//
// elapsed arg must be used
dbg_instrument!("sleep() took {elapsed:?}", sleep()).await;

// same as above, except with a predefined macro provided log message
dbg_instrument!(sleep()).await;

// will always print a debug log message regardless of `debug_assertions` status
//
// elapsed arg must be used
instrument!("{elapsed:?}", sleep()).await;

// same as above, except with a predefined macro provided log message
instrument!(sleep()).await;

// we can also manually create an instrumenting future if we require custom behavior
// or access to the elapsed data
let res = InstrumentFuture::new(sleep()).await;
println!("took {:?} with result {:?}", res.elapsed, res.result);
```

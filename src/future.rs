use std::sync::{Arc, Mutex, Condvar};
use std::result::Result;
use std::error::Error;
use std::boxed::Box;
use std::fmt::{Display, Formatter};
use std::fmt;

#[derive(Clone)]
pub struct Future<T,E> {
    r: Arc<fval<T,E>>,
}

struct fval<T,E> {
    result: Mutex<Option<Arc<Result<T,E>>>>,
    cond: Condvar,
}

impl <T,E> Future<T,E> {
    pub fn new() -> Future<T, E> {
        return Future { 
            r: Arc::new(
                fval {
                    result: Mutex::new(None),
                    cond: Condvar::new()
                }
            )
        };
    }


    /// Blocks until a Result is available, then returns it
    fn get(&self) -> Arc<Result<T, E>> {
        let mut res = self.r.result.lock().unwrap();
        while res.is_none() {
            res = self.r.cond.wait(res).unwrap();
        }
        return res.as_ref().expect("impossible").clone();
    }

   /// sets the result of this future, at which point all calls to get()
   /// will return an Arc'd reference to our result
   fn set(&self, result: Result<T, E>) {
        let mut res = self.r.result.lock().unwrap();
        if res.is_some() {
            panic!("Trying to set an already set future value!")
        }
        *res = Some(Arc::new(result));
        self.r.cond.notify_all();
    }

    /// Returns promptly whether done or not.  May block very briefly to check.
    fn isDone(&self) -> bool {
        let res = self.r.result.lock().unwrap();
        return match *res {
            Some(_) => true,
            None => false,
        };
    }
}

#[cfg(test)]
mod tests {
        use super::*;
        use std::thread;

        #[test]
        fn future_send_recv() {
            let send: Future<&str, &str> = Future::new();
            let recv: Future<&str, &str> = Future::new();

            let remoteRecv = send.clone();
            let remoteSend = recv.clone();
            // spawn thread to forward from send to recv
            // prepends "FWD: " to received msg
            thread::spawn(move || {

                let myRecv = remoteRecv.get();
                let ref toSend = match *myRecv {
                    Ok(mut s) => s,
                    Err(s) => panic!(s),
                };
                remoteSend.set(Ok(&toSend));
            });

            send.set(Ok("sent!"));

            let recv = recv.get();
            match *recv {
                Ok(s) => assert_eq!(s, "sent!"),
                Err(s) => panic!(s),
            }


            
        }
}


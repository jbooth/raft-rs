use std::net::TcpStream;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::io;
use std::result::Result;
use future;

pub struct Request<'a> {
    args: &'a[&'a[u8]],
}

pub struct PeerConn {
    // socket
    // reqno
    // map of responses to be fulfilled
    // peer service
    // launched thread listens 
    // config
        // timeout
        
}

impl PeerConn {
    pub fn new() -> PeerConn {
        return PeerConn{}
    }

    pub fn req(&self, args: *const[*const[u8]]) -> future::Future<Box<[u8]>, ConnError> {
        // check error flag
        // build response promise
        // store in map
        // make iovec of args
        // call writev
        // handle error
            // set error flag and return
        return Result::Err(ConnError::PREV_CLOSED)
    }

    fn close() {
        // set error flag to CLOSED
    }

    fn daemon() {
        loop {
            // check error flag and forward to handle error
 
            // read from socket

            // handle timeout
                // check pending resp times
                // if any exceeded actual max timeout, set error flag
                // else reloop

            // if resp, handle
            // if cmd, handle, if error on compute or write, set error flag


            // handle error
                // set error flag
                // iterate all pending Resp and forward error
                // teardown and reconnect loop
                    // should this ever terminate?
        }
    }
}

enum ConnError {
    PREV_CLOSED,
    IOERROR(io::Error),
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}

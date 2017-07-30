
use std::net::TcpStream;
use std::io;
use std::fmt;
use std::error;
use std::result::Result;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::RwLock;
use std::sync::Arc;
use std::time::Duration;
use chashmap::CHashMap;

pub struct PeerResponse {
    recv: Receiver<Result<Box<[u8]>, ConnError>>,
}

struct pendingPeerResponse {
    send: Sender<Result<Box<[u8]>, ConnError>>,
}

fn responsePair() -> (PeerResponse, pendingPeerResponse) {
    let (tx, rx) = channel();
    return (PeerResponse { recv: rx }, pendingPeerResponse { send: tx });
}

pub struct PeerConn {
    // socket
    s: TcpStream,
    // reqno
    reqno: AtomicUsize,
    // map of responses to be fulfilled
    pendingResp: CHashMap<usize, pendingPeerResponse>,
    // this is set if we get into a bad state and returned to future requests
    errState: RwLock<Option<ConnError>>,
    // scratch buffer for reading
    buff: Vec<u8>,
    // peer service

    // launched thread listens 
    // config
        // timeout
}

impl PeerConn {
    // wraps a new stream in a PeerConn
    // this stream should not have anything in incoming or outgoing buffers prior to wrapping
    pub fn new(
        conn: TcpStream,
        timeout: Option<Duration>,
        buffSize: Option<usize>,
    ) -> Result<PeerConn, io::Error> {
        // default timeout 10s
        let actualTimeout = timeout.unwrap_or(Duration::new(10, 0));
        // default buffer 128kb
        let actualBuffSize = buffSize.unwrap_or(128 * 1024);

        conn.set_read_timeout(Some(actualTimeout))?;
        conn.set_write_timeout(Some(actualTimeout))?;

        return Ok(PeerConn {
            s: conn,
            reqno: AtomicUsize::new(1),
            pendingResp: CHashMap::new(),
            errState: RwLock::new(None),
            buff: Vec::with_capacity(actualBuffSize),
        });
    }

    pub fn req(&mut self, args: &[&[u8]]) -> Result<PeerResponse, ConnError> {
        // check error flag
        self.checkErrState()?;
        // build response promise, store in map to be fulfilled
        let mut reqId = self.reqno.fetch_add(1, Ordering::SeqCst);
        // 0 reqId is reserved for reqs that don't want a resp
        while reqId == 0 {
            reqId = self.reqno.fetch_add(1, Ordering::SeqCst);
        }
        let (res, serverRes) = responsePair();
        let prev = self.pendingResp.insert(reqId, serverRes);
        if prev.is_some() {
            // this should only happen if we're forgetting to clean up resps
            // reqno can overflow but we shouldn't have billions/trillions of pending
            // reqs for a single connection
            panic!("pending responses aren't being cleaned up!");
        }
        // build header buffer
        // 2 byte numArgs, 4 bytes for argument length
        // max 65k args, max arg size 4GB
        // assemble final array and write
        let buffSize = 2 + (4 * args.len());
        // small allocation, could optimize onto stack or threadlocal somehow
        let mut cursor = io::Cursor::new(Vec::with_capacity(buffSize));
        use byteorder::{BigEndian, WriteBytesExt};
        cursor.write_u16::<BigEndian>(args.len() as u16).unwrap();
        for arg in args {
            cursor.write_u32::<BigEndian>(arg.len() as u32).unwrap();
        }
        let headerBuff = cursor.into_inner();
        {
            // assemble final &[&[u8]] for send
            // small allocation, could optimize
            let mut toWrite: &mut [&[u8]] = &mut (Vec::with_capacity(args.len() + 1));
            toWrite[0] = &headerBuff;
            toWrite[1..].copy_from_slice(args);

            use vecio::Rawv;
            return match self.s.writev(&toWrite) {
                Ok(_) => return Ok(res),
                // on error, mark so we shut down and die
                Err(e) => return Err(self.setIoError(e)),
            };

        }
        return Ok(res);
    }

    fn checkErrState(&self) -> Result<(), ConnError> {
        match *self.errState.read().unwrap() {
            Some(ref e) => return Err(e.clone()),
            None => return Ok(()),
        }
    }

    fn setErrState(&mut self, e: ConnError) -> ConnError {
        let mut errState = self.errState.write().unwrap();
        *errState = Some(e.clone());
        return e;
    }

    fn setIoError(&mut self, e: io::Error) -> ConnError {
        return self.setErrState(ConnError::IOERR(Arc::new(e)));
    }

    // marks this connection as closed
    // all new requests and pending responses will get an error
    pub fn close(&mut self) {
        self.setErrState(ConnError::CLOSED);
    }

    // background thread which
    //
    // 1)  monitors socket for inbound requests and responses
    //      a)  requests are forwarded to handler we were constructed with
    //      b)  responses are forwarded to their pending future
    // 2)  handles termination conditions, forwarding error to all pending responses
    fn daemon(&mut self) {
        'main: loop {
            // check error flag and forward to handle error
            if (self.checkErrState().is_err()) {
                break 'main;
            }

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
        // clean up pending futures, close socket and die
        let err: Result<Box<[u8]>, ConnError>  = Err(ConnError::CLOSED); //self.checkErrState();
        
        for (_reqno, resp) in (&mut self.pendingResp).into_iter().by_ref() {
            match resp.send.send(err.clone()) {
                        Ok(_) => (),
                        Err(e) => (), // TODO log
            }
        }
    }
}

struct msgHeader {
    argLenths: Vec<u32>,
    msgType: u8,
}


#[derive(Debug, Clone)]
enum ConnError {
    CLOSED,
    IOERR(Arc<io::Error>),
}

impl fmt::Display for ConnError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CLOSED => write!(f, "Previously closed"),
            //IOERR(e) => e.Display(f),
        }
    }
}

impl error::Error for ConnError {
    fn description(&self) -> &str {
        match self {
            CLOSED => &"Previously Closed",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match self {
            CLOSED => None,
        }
    }
}



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}

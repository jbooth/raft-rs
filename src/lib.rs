mod conn;
mod future;

struct server {
    c: conn::PeerConn,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}

mod conn;

extern crate chashmap;
extern crate vecio;
extern crate byteorder;

struct logEntryHead {
    term: u64,
    leaderId: u64,
    prevLogIdx: u64,
    prevLogTerm: u64,
    leaderCommitIdx: u64,
}

trait RaftApp {
    fn Apply();
}
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}

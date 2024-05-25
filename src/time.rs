use std::time::Duration;

pub fn timestamp() -> Duration {
    use std::time::SystemTime;
    let earlier = SystemTime::UNIX_EPOCH;
    let msg = "SystemTime before UNIX EPOCH!";

    SystemTime::now().duration_since(earlier).expect(msg)
}

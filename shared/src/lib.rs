use chrono::{DateTime, Utc};

#[cfg(not(feature = "test_features"))]
pub fn utc_now() -> DateTime<Utc> {
    Utc::now()
}

#[cfg(feature = "test_features")]
pub fn utc_now() -> DateTime<Utc> {
    test::now()
}

#[cfg(feature = "test_features")]
pub mod test {
    use std::cell::Cell;

    use chrono::{DateTime, Utc};

    thread_local! {
        static NOW: Cell<Option<DateTime<Utc>>> = const { Cell::new(None) };
    }

    pub fn now() -> DateTime<Utc> {
        NOW.with(|timestamp| {
            match timestamp.get() {
                Some(now) => return now,
                None => {
                    let now = Utc::now();
                    timestamp.set(Some(now.clone()));
                    now
                }
            }
        })
    }

    pub fn set_now(now: &DateTime<Utc>) {
        NOW.set(Some(now.clone()));
    }
}
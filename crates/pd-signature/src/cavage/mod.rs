use std::time::SystemTime;

pub mod header;
pub mod sig;

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[inline]
pub(crate) fn is_subset<I>(left: &[I], right: &[I]) -> bool
where
    I: PartialEq,
{
    if left.len() <= right.len() {
        left.iter().all(|item| right.contains(item))
    } else {
        false
    }
}

// Mockable time
#[inline]
pub(crate) fn get_current_time() -> SystemTime {
    #[cfg(any(test, feature = "mock-time-test"))]
    {
        MOCK_TIME.with(|t| t.borrow().unwrap_or_else(SystemTime::now))
    }
    #[cfg(not(any(test, feature = "mock-time-test")))]
    {
        SystemTime::now()
    }
}

#[cfg(any(test, feature = "mock-time-test"))]
thread_local! {
    pub static MOCK_TIME: std::cell::RefCell<Option<SystemTime>> = const { std::cell::RefCell::new(None) };
}

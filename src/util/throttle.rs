use std::{
    ops::Deref,
    time::{Duration, Instant},
};

use anyhow::Result;

/// A value that is only refreshed after a certain amount of time has passed.
pub struct Throttle<T> {
    last_update: Option<Instant>,
    value: T,
}

impl<T> Throttle<T> {
    /// Creates a new `Throttle` with the given value.
    /// The value will be considered fresh.
    #[allow(dead_code)]
    pub fn new(value: T) -> Throttle<T> {
        Throttle {
            last_update: Some(Instant::now()),
            value,
        }
    }

    /// Refreshes the value if it is older than the given `max_age`.
    pub fn try_refresh<U>(&mut self, max_age: Duration, try_refresh: U) -> Result<()>
    where
        U: FnOnce(&mut T) -> Result<()>,
    {
        let should_update = match self.last_update {
            None => true,
            Some(last_update) => last_update.elapsed() >= max_age,
        };
        if should_update {
            self.last_update = Some(Instant::now());
            try_refresh(&mut self.value)?;
        }
        Ok(())
    }
}

impl<T: Default> Default for Throttle<T> {
    /// Creates a new `Throttle` with the default value for the inner value.
    /// The value will be considered stale.
    fn default() -> Throttle<T> {
        Throttle {
            last_update: None,
            value: Default::default(),
        }
    }
}

impl<T> Deref for Throttle<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

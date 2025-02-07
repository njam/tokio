use futures::{self, Future};
use std::cell::{Cell, RefCell};
use std::error::Error;
use std::fmt;
use std::marker::PhantomData;
use std::prelude::v1::*;

thread_local!(static ENTERED: Cell<bool> = Cell::new(false));

/// Represents an executor context.
///
/// For more details, see [`enter` documentation](fn.enter.html)
pub struct Enter {
    _p: PhantomData<RefCell<()>>,
}

/// An error returned by `enter` if an execution scope has already been
/// entered.
pub struct EnterError {
    _a: (),
}

impl fmt::Debug for EnterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EnterError")
            .field("reason", &self.description())
            .finish()
    }
}

impl fmt::Display for EnterError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            fmt,
            "attempted to run an executor while another executor is already running"
        )
    }
}

impl Error for EnterError {}

/// Marks the current thread as being within the dynamic extent of an
/// executor.
///
/// Executor implementations should call this function before blocking the
/// thread. If `None` is returned, the executor should fail by panicking or
/// taking some other action without blocking the current thread. This prevents
/// deadlocks due to multiple executors competing for the same thread.
///
/// # Error
///
/// Returns an error if the current thread is already marked
pub fn enter() -> Result<Enter, EnterError> {
    ENTERED.with(|c| {
        if c.get() {
            Err(EnterError { _a: () })
        } else {
            c.set(true);

            Ok(Enter { _p: PhantomData })
        }
    })
}

impl Enter {
    /// Blocks the thread on the specified future, returning the value with
    /// which that future completes.
    pub fn block_on<F: Future>(&mut self, f: F) -> Result<F::Item, F::Error> {
        futures::executor::spawn(f).wait_future()
    }
}

impl fmt::Debug for Enter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Enter").finish()
    }
}

impl Drop for Enter {
    fn drop(&mut self) {
        ENTERED.with(|c| {
            assert!(c.get());
            c.set(false);
        });
    }
}

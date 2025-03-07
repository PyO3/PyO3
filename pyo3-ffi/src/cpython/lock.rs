use std::sync::atomic::AtomicU8;

#[repr(transparent)]
#[derive(Debug)]
pub struct PyMutex {
    pub(crate) _bits: AtomicU8,
}

impl PyMutex {
    pub const fn new() -> PyMutex {
        PyMutex {
            _bits: AtomicU8::new(0),
        }
    }
}

impl Default for PyMutex {
    fn default() -> Self {
        Self::new()
    }
}

extern "C" {
    pub fn PyMutex_Lock(m: *mut PyMutex);
    pub fn PyMutex_Unlock(m: *mut PyMutex);
}

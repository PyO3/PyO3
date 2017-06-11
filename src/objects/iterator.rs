// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use ffi;
use pointers::PyPtr;
use python::{Python, ToPyPointer, IntoPyPointer};
use objects::PyObject;
use err::{PyErr, PyResult, PyDowncastError};

/// A python iterator object.
///
/// Unlike other python objects, this class includes a `Python<'p>` token
/// so that `PyIterator` can implement the rust `Iterator` trait.
pub struct PyIterator<'p>(PyPtr, Python<'p>);


impl <'p> PyIterator<'p> {
    /// Constructs a `PyIterator` from a Python iterator object.
    pub fn from_object<T>(py: Python<'p>, obj: T)
                          -> Result<PyIterator<'p>, PyDowncastError<'p>>
        where T: IntoPyPointer
    {
        unsafe {
            let ptr = obj.into_ptr();
            if ffi::PyIter_Check(ptr) != 0 {
                Ok(PyIterator(PyPtr::from_borrowed_ptr(ptr), py))
            } else {
                ffi::Py_DECREF(ptr);
                Err(PyDowncastError(py, None))
            }
        }
    }
}

impl <'p> Iterator for PyIterator<'p> {
    type Item = PyResult<PyObject>;

    /// Retrieves the next item from an iterator.
    /// Returns `None` when the iterator is exhausted.
    /// If an exception occurs, returns `Some(Err(..))`.
    /// Further `next()` calls after an exception occurs are likely
    /// to repeatedly result in the same exception.
    fn next(&mut self) -> Option<PyResult<PyObject>> {
        match unsafe { PyObject::from_owned_ptr_or_opt(
            self.1, ffi::PyIter_Next(self.0.as_ptr())) } {
            Some(obj) => Some(Ok(obj)),
            None => {
                if PyErr::occurred(self.1) {
                    Some(Err(PyErr::fetch(self.1)))
                } else {
                    None
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use python::{Python};
    use conversion::ToPyObject;
    use objectprotocol::ObjectProtocol;

    #[test]
    fn vec_iter() {
        let gil_guard = Python::acquire_gil();
        let py = gil_guard.python();
        let obj = vec![10, 20].to_object(py);
        let mut it = obj.iter(py).unwrap();
        assert_eq!(10, it.next().unwrap().unwrap().extract(py).unwrap());
        assert_eq!(20, it.next().unwrap().unwrap().extract(py).unwrap());
        assert!(it.next().is_none());
    }
}

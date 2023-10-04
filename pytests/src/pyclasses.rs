use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::iter::IterNextOutput;
use pyo3::prelude::*;
use pyo3::types::PyType;

#[pyclass]
struct EmptyClass {}

#[pymethods]
impl EmptyClass {
    #[new]
    fn new() -> Self {
        EmptyClass {}
    }
}

/// This is for demonstrating how to return a value from __next__
#[pyclass]
#[derive(Default)]
struct PyClassIter {
    count: usize,
}

#[pymethods]
impl PyClassIter {
    #[new]
    pub fn new() -> Self {
        Default::default()
    }

    fn __next__(&mut self) -> IterNextOutput<usize, &'static str> {
        if self.count < 5 {
            self.count += 1;
            IterNextOutput::Yield(self.count)
        } else {
            IterNextOutput::Return("Ended")
        }
    }
}

/// Demonstrates a base class which can operate on the relevant subclass in its constructor.
#[pyclass(subclass)]
#[derive(Clone, Debug)]
struct AssertingBaseClass;

#[pymethods]
impl AssertingBaseClass {
    #[new]
    #[classmethod]
    fn new(cls: &PyType, expected_type: &PyType) -> PyResult<Self> {
        if !cls.is(expected_type) {
            return Err(PyValueError::new_err(format!(
                "{:?} != {:?}",
                cls, expected_type
            )));
        }
        Ok(Self)
    }
}

#[pyclass]
struct PyClassDel {
    count: Option<usize>,
}

#[pymethods]
impl PyClassDel {
    #[new]
    pub fn new() -> Self {
        PyClassDel { count: Some(0) }
    }

    fn __del__(&mut self) {
        self.count = None;
    }
}

#[pyclass]
struct PyClassDelError {}

#[pymethods]
impl PyClassDelError {
    #[new]
    pub fn new() -> Self {
        PyClassDelError {}
    }

    fn __del__(&mut self) -> PyResult<()> {
        Err(PyRuntimeError::new_err("__del__ error"))
    }
}

#[pymodule]
pub fn pyclasses(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<EmptyClass>()?;
    m.add_class::<PyClassIter>()?;
    m.add_class::<AssertingBaseClass>()?;
    m.add_class::<PyClassDel>()?;
    m.add_class::<PyClassDelError>()?;
    Ok(())
}

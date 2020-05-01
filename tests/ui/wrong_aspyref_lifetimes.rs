use pyo3::type_marker::Dict;
use pyo3::{types::PyDict, AsPyRef, Py, PyNativeType, Python};

fn main() {
    let gil = Python::acquire_gil();
    let dict: Py<Dict> = PyDict::new(gil.python()).into();
    let dict: &PyDict = dict.as_ref(gil.python());
    drop(gil);

    let _py: Python = dict.py(); // Obtain a Python<'p> without GIL.
}

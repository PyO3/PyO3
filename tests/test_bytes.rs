#![cfg(feature = "macros")]

use pyo3::prelude::*;
use pyo3::types::PyBytes;

#[path = "../src/tests/common.rs"]
mod common;

#[pyfunction]
fn bytes_pybytes_conversion(bytes: &[u8]) -> &[u8] {
    bytes
}

#[test]
fn test_pybytes_bytes_conversion() {
    Python::with_gil(|py| {
        let f = wrap_pyfunction_bound!(bytes_pybytes_conversion)(py).unwrap();
        py_assert!(py, f, "f(b'Hello World') == b'Hello World'");
    });
}

#[pyfunction]
fn bytes_vec_conversion(py: Python<'_>, bytes: Vec<u8>) -> Bound<'_, PyBytes> {
    PyBytes::new_bound(py, bytes.as_slice())
}

#[test]
fn test_pybytes_vec_conversion() {
    Python::with_gil(|py| {
        let f = wrap_pyfunction_bound!(bytes_vec_conversion)(py).unwrap();
        py_assert!(py, f, "f(b'Hello World') == b'Hello World'");
    });
}

#[test]
fn test_bytearray_vec_conversion() {
    Python::with_gil(|py| {
        let f = wrap_pyfunction_bound!(bytes_vec_conversion)(py).unwrap();
        py_assert!(py, f, "f(bytearray(b'Hello World')) == b'Hello World'");
    });
}

#[test]
fn test_py_as_bytes() {
    let pyobj: pyo3::Py<pyo3::types::PyBytes> =
        Python::with_gil(|py| pyo3::types::PyBytes::new_bound(py, b"abc").unbind());

    let data = Python::with_gil(|py| pyobj.as_bytes(py));

    assert_eq!(data, b"abc");

    Python::with_gil(move |_py| drop(pyobj));
}

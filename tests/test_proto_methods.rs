#![cfg(feature = "macros")]

use pyo3::exceptions::{PyIndexError, PyValueError};
use pyo3::types::{PyDict, PyList, PyMapping, PySequence, PySlice, PyType};
use pyo3::{exceptions::PyAttributeError, prelude::*};
use pyo3::{ffi, py_run, AsPyPointer, PyCell};
use std::{isize, iter};

mod common;

#[pyclass]
struct EmptyClass;

#[pyclass]
struct ExampleClass {
    #[pyo3(get, set)]
    value: i32,
    _custom_attr: Option<i32>,
}

#[pymethods]
impl ExampleClass {
    fn __getattr__(&self, py: Python, attr: &str) -> PyResult<PyObject> {
        if attr == "special_custom_attr" {
            Ok(self._custom_attr.into_py(py))
        } else {
            Err(PyAttributeError::new_err(attr.to_string()))
        }
    }

    fn __setattr__(&mut self, attr: &str, value: &PyAny) -> PyResult<()> {
        if attr == "special_custom_attr" {
            self._custom_attr = Some(value.extract()?);
            Ok(())
        } else {
            Err(PyAttributeError::new_err(attr.to_string()))
        }
    }

    fn __delattr__(&mut self, attr: &str) -> PyResult<()> {
        if attr == "special_custom_attr" {
            self._custom_attr = None;
            Ok(())
        } else {
            Err(PyAttributeError::new_err(attr.to_string()))
        }
    }

    fn __str__(&self) -> String {
        self.value.to_string()
    }

    fn __repr__(&self) -> String {
        format!("ExampleClass(value={})", self.value)
    }

    fn __hash__(&self) -> u64 {
        let i64_value: i64 = self.value.into();
        i64_value as u64
    }

    fn __bool__(&self) -> bool {
        self.value != 0
    }
}

fn make_example(py: Python) -> &PyCell<ExampleClass> {
    Py::new(
        py,
        ExampleClass {
            value: 5,
            _custom_attr: Some(20),
        },
    )
    .unwrap()
    .into_ref(py)
}

#[test]
fn test_getattr() {
    Python::with_gil(|py| {
        let example_py = make_example(py);
        assert_eq!(
            example_py
                .getattr("value")
                .unwrap()
                .extract::<i32>()
                .unwrap(),
            5,
        );
        assert_eq!(
            example_py
                .getattr("special_custom_attr")
                .unwrap()
                .extract::<i32>()
                .unwrap(),
            20,
        );
        assert!(example_py
            .getattr("other_attr")
            .unwrap_err()
            .is_instance_of::<PyAttributeError>(py));
    })
}

#[test]
fn test_setattr() {
    Python::with_gil(|py| {
        let example_py = make_example(py);
        example_py.setattr("special_custom_attr", 15).unwrap();
        assert_eq!(
            example_py
                .getattr("special_custom_attr")
                .unwrap()
                .extract::<i32>()
                .unwrap(),
            15,
        );
    })
}

#[test]
fn test_delattr() {
    Python::with_gil(|py| {
        let example_py = make_example(py);
        example_py.delattr("special_custom_attr").unwrap();
        assert!(example_py.getattr("special_custom_attr").unwrap().is_none());
    })
}

#[test]
fn test_str() {
    Python::with_gil(|py| {
        let example_py = make_example(py);
        assert_eq!(example_py.str().unwrap().to_str().unwrap(), "5");
    })
}

#[test]
fn test_repr() {
    Python::with_gil(|py| {
        let example_py = make_example(py);
        assert_eq!(
            example_py.repr().unwrap().to_str().unwrap(),
            "ExampleClass(value=5)"
        );
    })
}

#[test]
fn test_hash() {
    Python::with_gil(|py| {
        let example_py = make_example(py);
        assert_eq!(example_py.hash().unwrap(), 5);
    })
}

#[test]
fn test_bool() {
    Python::with_gil(|py| {
        let example_py = make_example(py);
        assert!(example_py.is_true().unwrap());
        example_py.borrow_mut().value = 0;
        assert!(!example_py.is_true().unwrap());
    })
}

#[pyclass]
pub struct LenOverflow;

#[pymethods]
impl LenOverflow {
    fn __len__(&self) -> usize {
        (isize::MAX as usize) + 1
    }
}

#[test]
fn len_overflow() {
    Python::with_gil(|py| {
        let inst = Py::new(py, LenOverflow).unwrap();
        py_expect_exception!(py, inst, "len(inst)", PyOverflowError);
    });
}

#[pyclass]
pub struct MappingWithSeqDefaults {
    values: Py<PyDict>,
}

#[pymethods]
impl MappingWithSeqDefaults {
    fn __len__(&self, py: Python) -> usize {
        self.values.as_ref(py).len()
    }

    fn __getitem__<'a>(&'a self, key: &'a PyAny) -> PyResult<&'a PyAny> {
        let any: &PyAny = self.values.as_ref(key.py()).as_ref();
        any.get_item(key)
    }

    fn __setitem__(&self, key: &PyAny, value: &PyAny) -> PyResult<()> {
        self.values.as_ref(key.py()).set_item(key, value)
    }

    fn __delitem__(&self, key: &PyAny) -> PyResult<()> {
        self.values.as_ref(key.py()).del_item(key)
    }
}

#[test]
fn mapping_with_seq_defaults() {
    Python::with_gil(|py| {
        let inst = Py::new(
            py,
            MappingWithSeqDefaults {
                values: PyDict::new(py).into(),
            },
        )
        .unwrap();

        //
        let mapping: &PyMapping = inst.as_ref(py).downcast().unwrap();
        let sequence: &PySequence = inst.as_ref(py).downcast().unwrap();

        py_assert!(py, inst, "len(inst) == 0");

        py_run!(py, inst, "inst['foo'] = 'foo'");
        py_assert!(py, inst, "inst['foo'] == 'foo'");
        py_run!(py, inst, "del inst['foo']");
        py_expect_exception!(py, inst, "inst['foo']", PyKeyError);

        // Default iteration will go through __getseqitem__, which then defaults to call __getitem__
        // which fails with a KeyError
        py_expect_exception!(py, inst, "[*inst] == []", PyKeyError, "0");

        // check mapping protocol
        assert_eq!(mapping.len().unwrap(), 0);

        mapping.set_item(0, 5).unwrap();
        assert_eq!(mapping.len().unwrap(), 1);

        assert_eq!(mapping.get_item(0).unwrap().extract::<u8>().unwrap(), 5);

        mapping.del_item(0).unwrap();
        assert_eq!(mapping.len().unwrap(), 0);

        // check sequence protocol
        assert_eq!(sequence.len().unwrap(), 0);

        sequence.set_item(0, 5).unwrap();
        assert_eq!(sequence.len().unwrap(), 1);

        assert_eq!(sequence.get_item(0).unwrap().extract::<u8>().unwrap(), 5);
        sequence.del_item(0).unwrap();

        assert_eq!(sequence.len().unwrap(), 0);
    });
}

#[pyclass]
pub struct MappingAndSeq {
    values: Py<PyDict>,
    last_access: String,
}

#[pymethods]
impl MappingAndSeq {
    fn __len__(&mut self, py: Python) -> usize {
        self.last_access = "__len__()".into();
        self.values.as_ref(py).len()
    }

    fn __getitem__<'a>(&'a mut self, key: &'a PyAny) -> PyResult<&'a PyAny> {
        self.last_access = format!("__getitem__({:?})", key);
        let any: &PyAny = self.values.as_ref(key.py()).as_ref();
        any.get_item(key)
    }

    fn __setitem__(&mut self, key: &PyAny, value: &PyAny) -> PyResult<()> {
        self.last_access = format!("__setitem__({:?}, {:?})", key, value);
        self.values.as_ref(key.py()).set_item(key, value)
    }

    fn __delitem__(&mut self, key: &PyAny) -> PyResult<()> {
        self.last_access = format!("__delitem__({:?})", key);
        self.values.as_ref(key.py()).del_item(key)
    }

    fn __seqlen__(&mut self, py: Python) -> usize {
        self.last_access = "__seqlen__()".into();
        self.values.as_ref(py).len()
    }

    fn __getseqitem__<'a>(&'a mut self, py: Python<'a>, index: isize) -> PyResult<&PyAny> {
        self.last_access = format!("__getseqitem__({:?})", index);
        self.values
            .as_ref(py)
            .get_item(index)
            .ok_or_else(|| PyIndexError::new_err(index))
    }

    fn __setseqitem__(&mut self, index: isize, value: &PyAny) -> PyResult<()> {
        self.last_access = format!("__setseqitem__({:?}, {:?})", index, value);
        self.values.as_ref(value.py()).set_item(index, value)
    }

    fn __delseqitem__(&mut self, py: Python, index: isize) -> PyResult<()> {
        self.last_access = format!("__delseqitem__({:?})", index);
        self.values.as_ref(py).del_item(index)
    }
}

#[test]
fn mapping_and_seq() {
    Python::with_gil(|py| {
        let inst = Py::new(
            py,
            MappingAndSeq {
                values: PyDict::new(py).into(),
                last_access: String::new(),
            },
        )
        .unwrap();

        macro_rules! assert_last_access {
            ($string:literal) => {
                assert_eq!(inst.borrow(py).last_access, $string);
            };
        }

        let mapping: &PyMapping = inst.as_ref(py).downcast().unwrap();
        let sequence: &PySequence = inst.as_ref(py).downcast().unwrap();

        // Python len() goes through __seqlen__ first
        py_assert!(py, inst, "len(inst) == 0");
        assert_last_access!("__seqlen__()");

        // Python indexing prefers mapping protocol to sequence protocol
        py_run!(py, inst, "inst['foo'] = 'foo'");
        assert_last_access!("__setitem__('foo', 'foo')");
        py_run!(py, inst, "inst[-5] = -5");
        assert_last_access!("__setitem__(-5, -5)");

        py_assert!(py, inst, "inst['foo'] == 'foo'");
        assert_last_access!("__getitem__('foo')");
        py_assert!(py, inst, "inst[-5] == -5");
        assert_last_access!("__getitem__(-5)");

        py_run!(py, inst, "del inst['foo']");
        assert_last_access!("__delitem__('foo')");
        py_run!(py, inst, "del inst[-5]");
        assert_last_access!("__delitem__(-5)");

        py_expect_exception!(py, inst, "inst['foo']", PyKeyError);
        assert_last_access!("__getitem__('foo')");

        // Default iteration will go through __getseqitem__
        py_assert!(py, inst, "[*inst] == []");
        assert_last_access!("__getseqitem__(0)");
        py_run!(py, inst, "inst[0] = 'hi'");
        assert_last_access!("__setitem__(0, 'hi')");
        py_assert!(py, inst, "[*inst] == ['hi']");
        assert_last_access!("__getseqitem__(1)");
        py_run!(py, inst, "del inst[0]");
        assert_last_access!("__delitem__(0)");

        // check mapping protocol
        assert_eq!(mapping.len().unwrap(), 0);
        assert_last_access!("__len__()");

        mapping.set_item(0, 5).unwrap();
        assert_last_access!("__setitem__(0, 5)");

        assert_eq!(mapping.len().unwrap(), 1);
        assert_last_access!("__len__()");

        assert_eq!(mapping.get_item(0).unwrap().extract::<u8>().unwrap(), 5);
        assert_last_access!("__getitem__(0)");

        mapping.del_item(0).unwrap();
        assert_last_access!("__delitem__(0)");

        assert_eq!(mapping.len().unwrap(), 0);
        assert_last_access!("__len__()");

        // check sequence protocol
        assert_eq!(sequence.len().unwrap(), 0);
        assert_last_access!("__seqlen__()");

        sequence.set_item(0, 5).unwrap();
        assert_last_access!("__setseqitem__(0, 5)");

        assert_eq!(sequence.len().unwrap(), 1);
        assert_last_access!("__seqlen__()");

        assert_eq!(sequence.get_item(0).unwrap().extract::<u8>().unwrap(), 5);
        assert_last_access!("__getseqitem__(0)");

        sequence.del_item(0).unwrap();
        assert_last_access!("__delseqitem__(0)");

        assert_eq!(sequence.len().unwrap(), 0);
        assert_last_access!("__seqlen__()");

        // FIXME: add an example & guide for sequence
        // FIXME: add an example & guide for mapping
    });
}

#[pyclass(true_mapping)]
pub struct MappingOnly {
    values: Py<PyDict>,
}

#[pymethods]
impl MappingOnly {
    fn __len__(&self, py: Python) -> usize {
        self.values.as_ref(py).len()
    }

    fn __getitem__<'a>(&'a self, key: &'a PyAny) -> PyResult<&'a PyAny> {
        let any: &PyAny = self.values.as_ref(key.py()).as_ref();
        any.get_item(key)
    }

    fn __setitem__(&self, key: &PyAny, value: &PyAny) -> PyResult<()> {
        self.values.as_ref(key.py()).set_item(key, value)
    }

    fn __delitem__(&self, key: &PyAny) -> PyResult<()> {
        self.values.as_ref(key.py()).del_item(key)
    }
}

#[test]
fn mapping_only() {
    Python::with_gil(|py| {
        let inst = Py::new(
            py,
            MappingOnly {
                values: PyDict::new(py).into(),
            },
        )
        .unwrap();

        let mapping: &PyMapping = inst.as_ref(py).downcast().unwrap();
        assert!(inst.as_ref(py).downcast::<PySequence>().is_err());

        py_assert!(py, inst, "len(inst) == 0");

        unsafe {
            assert_eq!(ffi::PyObject_Size(inst.as_ptr()), 0);
            assert_eq!(ffi::PyMapping_Size(inst.as_ptr()), 0);
            // Not a sequence; ffi call should fail
            assert_eq!(ffi::PySequence_Size(inst.as_ptr()), -1);
            let _ = PyErr::fetch(py);
        }

        py_run!(py, inst, "inst['foo'] = 'foo'");
        py_assert!(py, inst, "inst['foo'] == 'foo'");
        py_run!(py, inst, "del inst['foo']");

        py_expect_exception!(py, inst, "inst['foo']", PyKeyError);

        // Not a sequence => no default iteration
        py_expect_exception!(
            py,
            inst,
            "[*inst]",
            PyTypeError,
            "'builtins.MappingOnly' object is not iterable"
        );

        // check mapping protocol
        assert_eq!(mapping.len().unwrap(), 0);

        mapping.set_item(0, 5).unwrap();
        assert_eq!(mapping.len().unwrap(), 1);

        assert_eq!(mapping.get_item(0).unwrap().extract::<u8>().unwrap(), 5);
        mapping.del_item(0).unwrap();

        assert_eq!(mapping.len().unwrap(), 0);
    });
}

#[pyclass]
pub struct SequenceOnly {
    values: Vec<PyObject>,
}

#[pymethods]
impl SequenceOnly {
    fn __seqlen__(&self) -> usize {
        self.values.len()
    }

    fn __getseqitem__(&self, index: isize) -> PyResult<PyObject> {
        let uindex = self.usize_index(index)?;
        self.values
            .get(uindex)
            .map(Clone::clone)
            .ok_or_else(|| PyIndexError::new_err("sequence index out of range"))
    }

    fn __setseqitem__(&mut self, index: isize, value: PyObject) -> PyResult<()> {
        let uindex = self.usize_index(index)?;
        self.values
            .get_mut(uindex)
            .map(|place| *place = value)
            .ok_or_else(|| PyIndexError::new_err("sequence index out of range"))
    }

    fn __delseqitem__(&mut self, index: isize) -> PyResult<()> {
        let uindex = self.usize_index(index)?;
        if uindex >= self.values.len() {
            Err(PyIndexError::new_err("sequence index out of range"))
        } else {
            self.values.remove(uindex);
            Ok(())
        }
    }

    fn append(&mut self, value: PyObject) {
        self.values.push(value);
    }
}

impl SequenceOnly {
    fn usize_index(&self, index: isize) -> PyResult<usize> {
        if index < 0 {
            // NB don't need to subtract index from length because CPython already does this
            // for us because __seqlen__ is defined.
            Err(PyIndexError::new_err("sequence index out of range"))
        } else {
            Ok(index as usize)
        }
    }
}

#[test]
fn sequence_only() {
    Python::with_gil(|py| {
        let inst = Py::new(py, SequenceOnly { values: vec![] }).unwrap();

        let sequence: &PySequence = inst.as_ref(py).downcast().unwrap();

        py_assert!(py, inst, "len(inst) == 0");

        unsafe {
            assert_eq!(ffi::PyObject_Size(inst.as_ptr()), 0);
            // Not a mapping; ffi call should fail
            assert_eq!(ffi::PyMapping_Size(inst.as_ptr()), -1);
            let _ = PyErr::fetch(py);
            assert_eq!(ffi::PySequence_Size(inst.as_ptr()), 0);
        }

        py_expect_exception!(py, inst, "inst[0]", PyIndexError);
        py_run!(py, inst, "inst.append('foo')");

        py_assert!(py, inst, "inst[0] == 'foo'");
        py_assert!(py, inst, "inst[-1] == 'foo'");

        py_expect_exception!(py, inst, "inst[1]", PyIndexError);
        py_expect_exception!(py, inst, "inst[-2]", PyIndexError);

        py_assert!(py, inst, "[*inst] == ['foo']");

        py_run!(py, inst, "del inst[0]");

        py_expect_exception!(
            py,
            inst,
            "inst['foo']",
            PyTypeError,
            "sequence index must be integer, not 'str'"
        );

        // slicing needs to be implemented through __getitem__ (not __getseqitem__)
        py_expect_exception!(
            py,
            inst,
            "inst[0:2]",
            PyTypeError,
            "sequence index must be integer, not 'slice'"
        );

        // check sequence protocol
        assert_eq!(sequence.len().unwrap(), 0);

        py_run!(py, inst, "inst.append(0)");
        sequence.set_item(0, 5).unwrap();
        assert_eq!(sequence.len().unwrap(), 1);

        assert_eq!(sequence.get_item(0).unwrap().extract::<u8>().unwrap(), 5);
        sequence.del_item(0).unwrap();

        assert_eq!(sequence.len().unwrap(), 0);
    });
}

#[pyclass]
struct Iterator {
    iter: Box<dyn iter::Iterator<Item = i32> + Send>,
}

#[pymethods]
impl Iterator {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<Self>) -> Option<i32> {
        slf.iter.next()
    }
}

#[test]
fn iterator() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let inst = Py::new(
        py,
        Iterator {
            iter: Box::new(5..8),
        },
    )
    .unwrap();
    py_assert!(py, inst, "iter(inst) is inst");
    py_assert!(py, inst, "list(inst) == [5, 6, 7]");
}

#[pyclass]
struct Callable;

#[pymethods]
impl Callable {
    fn __call__(&self, arg: i32) -> i32 {
        arg * 6
    }
}

#[pyclass]
struct NotCallable;

#[test]
fn callable() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = Py::new(py, Callable).unwrap();
    py_assert!(py, c, "callable(c)");
    py_assert!(py, c, "c(7) == 42");

    let nc = Py::new(py, NotCallable).unwrap();
    py_assert!(py, nc, "not callable(nc)");
}

#[allow(deprecated)]
mod deprecated {
    use super::*;

    #[pyclass]
    struct Callable;

    #[pymethods]
    impl Callable {
        #[__call__]
        fn __call__(&self, arg: i32) -> i32 {
            arg * 6
        }
    }

    #[test]
    fn callable() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let c = Py::new(py, Callable).unwrap();
        py_assert!(py, c, "callable(c)");
        py_assert!(py, c, "c(7) == 42");
    }
}

#[pyclass]
#[derive(Debug)]
struct SetItem {
    key: i32,
    val: i32,
}

#[pymethods]
impl SetItem {
    fn __setitem__(&mut self, key: i32, val: i32) {
        self.key = key;
        self.val = val;
    }
}

#[test]
fn setitem() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = PyCell::new(py, SetItem { key: 0, val: 0 }).unwrap();
    py_run!(py, c, "c[1] = 2");
    {
        let c = c.borrow();
        assert_eq!(c.key, 1);
        assert_eq!(c.val, 2);
    }
    py_expect_exception!(py, c, "del c[1]", PyNotImplementedError);
}

#[pyclass]
struct DelItem {
    key: i32,
}

#[pymethods]
impl DelItem {
    fn __delitem__(&mut self, key: i32) {
        self.key = key;
    }
}

#[test]
fn delitem() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = PyCell::new(py, DelItem { key: 0 }).unwrap();
    py_run!(py, c, "del c[1]");
    {
        let c = c.borrow();
        assert_eq!(c.key, 1);
    }
    py_expect_exception!(py, c, "c[1] = 2", PyNotImplementedError);
}

#[pyclass]
struct SetDelItem {
    val: Option<i32>,
}

#[pymethods]
impl SetDelItem {
    fn __setitem__(&mut self, _key: i32, val: i32) {
        self.val = Some(val);
    }

    fn __delitem__(&mut self, _key: i32) {
        self.val = None;
    }
}

#[test]
fn setdelitem() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = PyCell::new(py, SetDelItem { val: None }).unwrap();
    py_run!(py, c, "c[1] = 2");
    {
        let c = c.borrow();
        assert_eq!(c.val, Some(2));
    }
    py_run!(py, c, "del c[1]");
    let c = c.borrow();
    assert_eq!(c.val, None);
}

#[pyclass]
struct Contains {}

#[pymethods]
impl Contains {
    fn __contains__(&self, item: i32) -> bool {
        item >= 0
    }
}

#[test]
fn contains() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = Py::new(py, Contains {}).unwrap();
    py_run!(py, c, "assert 1 in c");
    py_run!(py, c, "assert -1 not in c");
    py_expect_exception!(py, c, "assert 'wrong type' not in c", PyTypeError);
}

#[pyclass]
struct GetItem {}

#[pymethods]
impl GetItem {
    fn __getitem__(&self, idx: &PyAny) -> PyResult<&'static str> {
        if let Ok(slice) = idx.cast_as::<PySlice>() {
            let indices = slice.indices(1000)?;
            if indices.start == 100 && indices.stop == 200 && indices.step == 1 {
                return Ok("slice");
            }
        } else if let Ok(idx) = idx.extract::<isize>() {
            if idx == 1 {
                return Ok("int");
            }
        }
        Err(PyValueError::new_err("error"))
    }
}

#[test]
fn test_getitem() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let ob = Py::new(py, GetItem {}).unwrap();

    py_assert!(py, ob, "ob[1] == 'int'");
    py_assert!(py, ob, "ob[100:200:1] == 'slice'");
}

#[pyclass]
struct ClassWithGetAttr {
    #[pyo3(get, set)]
    data: u32,
}

#[pymethods]
impl ClassWithGetAttr {
    fn __getattr__(&self, _name: &str) -> u32 {
        self.data * 2
    }
}

#[test]
fn getattr_doesnt_override_member() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let inst = PyCell::new(py, ClassWithGetAttr { data: 4 }).unwrap();
    py_assert!(py, inst, "inst.data == 4");
    py_assert!(py, inst, "inst.a == 8");
}

/// Wraps a Python future and yield it once.
#[pyclass]
struct OnceFuture {
    future: PyObject,
    polled: bool,
}

#[pymethods]
impl OnceFuture {
    #[new]
    fn new(future: PyObject) -> Self {
        OnceFuture {
            future,
            polled: false,
        }
    }

    fn __await__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }
    fn __next__(mut slf: PyRefMut<Self>) -> Option<PyObject> {
        if !slf.polled {
            slf.polled = true;
            Some(slf.future.clone())
        } else {
            None
        }
    }
}

#[test]
fn test_await() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let once = py.get_type::<OnceFuture>();
    let source = pyo3::indoc::indoc!(
        r#"
import asyncio
import sys

async def main():
    res = await Once(await asyncio.sleep(0.1))
    return res
# For an odd error similar to https://bugs.python.org/issue38563
if sys.platform == "win32" and sys.version_info >= (3, 8, 0):
    asyncio.set_event_loop_policy(asyncio.WindowsSelectorEventLoopPolicy())
# get_event_loop can raise an error: https://github.com/PyO3/pyo3/pull/961#issuecomment-645238579
loop = asyncio.new_event_loop()
asyncio.set_event_loop(loop)
assert loop.run_until_complete(main()) is None
loop.close()
"#
    );
    let globals = PyModule::import(py, "__main__").unwrap().dict();
    globals.set_item("Once", once).unwrap();
    py.run(source, Some(globals), None)
        .map_err(|e| e.print(py))
        .unwrap();
}

/// Increment the count when `__get__` is called.
#[pyclass]
struct DescrCounter {
    #[pyo3(get)]
    count: usize,
}

#[pymethods]
impl DescrCounter {
    #[new]
    fn new() -> Self {
        DescrCounter { count: 0 }
    }
    /// Each access will increase the count
    fn __get__<'a>(
        mut slf: PyRefMut<'a, Self>,
        _instance: &PyAny,
        _owner: Option<&PyType>,
    ) -> PyRefMut<'a, Self> {
        slf.count += 1;
        slf
    }
    /// Allow assigning a new counter to the descriptor, copying the count across
    fn __set__(&self, _instance: &PyAny, new_value: &mut Self) {
        new_value.count = self.count;
    }
    /// Delete to reset the counter
    fn __delete__(&mut self, _instance: &PyAny) {
        self.count = 0;
    }
}

#[test]
fn descr_getset() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let counter = py.get_type::<DescrCounter>();
    let source = pyo3::indoc::indoc!(
        r#"
class Class:
    counter = Counter()

# access via type
counter = Class.counter
assert counter.count == 1

# access with instance directly
assert Counter.__get__(counter, Class()).count == 2

# access via instance
c = Class()
assert c.counter.count == 3

# __set__
c.counter = Counter()
assert c.counter.count == 4

# __delete__
del c.counter
assert c.counter.count == 1
"#
    );
    let globals = PyModule::import(py, "__main__").unwrap().dict();
    globals.set_item("Counter", counter).unwrap();
    py.run(source, Some(globals), None)
        .map_err(|e| e.print(py))
        .unwrap();
}

#[pyclass]
struct NotHashable;

#[pymethods]
impl NotHashable {
    #[classattr]
    const __hash__: Option<PyObject> = None;
}

#[test]
fn test_hash_opt_out() {
    // By default Python provides a hash implementation, which can be disabled by setting __hash__
    // to None.
    Python::with_gil(|py| {
        let empty = Py::new(py, EmptyClass).unwrap();
        py_assert!(py, empty, "hash(empty) is not None");

        let not_hashable = Py::new(py, NotHashable).unwrap();
        py_expect_exception!(py, not_hashable, "hash(not_hashable)", PyTypeError);
    })
}

/// Class with __iter__ gets default contains from CPython.
#[pyclass]
struct DefaultedContains;

#[pymethods]
impl DefaultedContains {
    fn __iter__(&self, py: Python) -> PyObject {
        PyList::new(py, &["a", "b", "c"])
            .as_ref()
            .iter()
            .unwrap()
            .into()
    }
}

#[pyclass]
struct NoContains;

#[pymethods]
impl NoContains {
    fn __iter__(&self, py: Python) -> PyObject {
        PyList::new(py, &["a", "b", "c"])
            .as_ref()
            .iter()
            .unwrap()
            .into()
    }

    // Equivalent to the opt-out const form in NotHashable above, just more verbose, to confirm this
    // also works.
    #[classattr]
    fn __contains__() -> Option<PyObject> {
        None
    }
}

#[test]
fn test_contains_opt_out() {
    Python::with_gil(|py| {
        let defaulted_contains = Py::new(py, DefaultedContains).unwrap();
        py_assert!(py, defaulted_contains, "'a' in defaulted_contains");

        let no_contains = Py::new(py, NoContains).unwrap();
        py_expect_exception!(py, no_contains, "'a' in no_contains", PyTypeError);
    })
}

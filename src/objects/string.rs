// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::types::{Any, PyBytes, Str};
use crate::{
    ffi,
    objects::{PyAny, PyNativeObject},
    owned::PyOwned,
    AsPyPointer, FromPyObject, IntoPy, Py, PyErr, PyNativeType, PyObject, PyResult, PyTryFrom,
    Python, ToPyObject,
};
use std::borrow::Cow;
use std::os::raw::c_char;
use std::str;

/// Represents a Python `str` (a Unicode string object).
///
/// This type is immutable.
#[repr(transparent)]
pub struct PyStr<'py>(Py<Str>, Python<'py>);

pyo3_native_object!(PyStr<'py>, Str, 'py);

impl<'py> PyStr<'py> {
    /// Creates a new Python string object.
    ///
    /// Panics if out of memory.
    pub fn new(py: Python<'py>, s: &str) -> PyOwned<'py, Str> {
        let ptr = s.as_ptr() as *const c_char;
        let len = s.len() as ffi::Py_ssize_t;
        unsafe { PyOwned::from_owned_ptr_or_panic(py, ffi::PyUnicode_FromStringAndSize(ptr, len)) }
    }

    pub fn from_object(src: &PyAny<'py>, encoding: &str, errors: &str) -> PyOwned<'py, Str> {
        unsafe {
            PyOwned::from_owned_ptr_or_panic(
                src.py(),
                ffi::PyUnicode_FromEncodedObject(
                    src.as_ptr(),
                    encoding.as_ptr() as *const c_char,
                    errors.as_ptr() as *const c_char,
                ),
            )
        }
    }

    /// Gets the Python string as a byte slice.
    ///
    /// Returns a `UnicodeEncodeError` if the input is not valid unicode
    /// (containing unpaired surrogates).
    #[inline]
    pub fn to_str(&self) -> PyResult<&str> {
        #[cfg(not(Py_LIMITED_API))]
        unsafe {
            let mut size: ffi::Py_ssize_t = 0;
            let data = ffi::PyUnicode_AsUTF8AndSize(self.as_ptr(), &mut size) as *const u8;
            if data.is_null() {
                Err(PyErr::fetch(self.py()))
            } else {
                let slice = std::slice::from_raw_parts(data, size as usize);
                Ok(std::str::from_utf8_unchecked(slice))
            }
        }
        #[cfg(Py_LIMITED_API)]
        unsafe {
            let data = ffi::PyUnicode_AsUTF8String(self.as_ptr());
            if data.is_null() {
                Err(PyErr::fetch(self.py()))
            } else {
                let bytes = self.py().from_owned_ptr::<PyBytes>(data);
                Ok(std::str::from_utf8_unchecked(bytes.as_bytes()))
            }
        }
    }

    /// Converts the `PyStr` into a Rust string.
    ///
    /// Unpaired surrogates invalid UTF-8 sequences are
    /// replaced with `U+FFFD REPLACEMENT CHARACTER`.
    pub fn to_string_lossy(&self) -> Cow<str> {
        match self.to_str() {
            Ok(s) => Cow::Borrowed(s),
            Err(_) => {
                let bytes: PyOwned<PyBytes> = unsafe {
                    PyOwned::from_owned_ptr_or_panic(
                        self.py(),
                        ffi::PyUnicode_AsEncodedString(
                            self.as_ptr(),
                            b"utf-8\0" as *const _ as _,
                            b"surrogatepass\0" as *const _ as _,
                        ),
                    )
                };
                String::from_utf8_lossy(bytes.as_bytes())
            }
        }
    }
}

/// Converts a Rust `str` to a Python object.
/// See `PyStr::new` for details on the conversion.
impl ToPyObject for str {
    #[inline]
    fn to_object(&self, py: Python) -> PyObject {
        PyStr::new(py, self).into()
    }
}

impl<'a> IntoPy<PyObject> for &'a str {
    #[inline]
    fn into_py(self, py: Python) -> PyObject {
        PyStr::new(py, self).into()
    }
}

/// Converts a Rust `Cow<str>` to a Python object.
/// See `PyStr::new` for details on the conversion.
impl<'a> ToPyObject for Cow<'a, str> {
    #[inline]
    fn to_object(&self, py: Python) -> PyObject {
        PyStr::new(py, self).into()
    }
}

/// Converts a Rust `String` to a Python object.
/// See `PyStr::new` for details on the conversion.
impl ToPyObject for String {
    #[inline]
    fn to_object(&self, py: Python) -> PyObject {
        PyStr::new(py, self).into()
    }
}

impl ToPyObject for char {
    fn to_object(&self, py: Python) -> PyObject {
        self.into_py(py)
    }
}

impl IntoPy<PyObject> for char {
    fn into_py(self, py: Python) -> PyObject {
        let mut bytes = [0u8; 4];
        PyStr::new(py, self.encode_utf8(&mut bytes)).into()
    }
}

impl IntoPy<PyObject> for String {
    fn into_py(self, py: Python) -> PyObject {
        PyStr::new(py, &self).into()
    }
}

impl<'a> IntoPy<PyObject> for &'a String {
    #[inline]
    fn into_py(self, py: Python) -> PyObject {
        PyStr::new(py, self).into()
    }
}

/// Allows extracting strings from Python objects.
/// Accepts Python `str` and `unicode` objects.
impl<'source> FromPyObject<'source> for &'source str {
    fn extract(ob: &'source Any) -> PyResult<Self> {
        <PyStr as PyTryFrom>::try_from(ob)?.to_str()
    }
}

/// Allows extracting strings from Python objects.
/// Accepts Python `str` and `unicode` objects.
impl FromPyObject<'_> for String {
    fn extract(obj: &Any) -> PyResult<Self> {
        <PyStr as PyTryFrom>::try_from(obj)?
            .to_str()
            .map(ToOwned::to_owned)
    }
}

impl FromPyObject<'_> for char {
    fn extract(obj: &Any) -> PyResult<Self> {
        let s = PyStr::try_from(obj)?.to_str()?;
        let mut iter = s.chars();
        if let (Some(ch), None) = (iter.next(), iter.next()) {
            Ok(ch)
        } else {
            Err(crate::exceptions::PyValueError::new_err(
                "expected a string of length 1",
            ))
        }
    }
}

#[cfg(test)]
mod test {
    use super::PyStr;
    use crate::Python;
    use crate::{FromPyObject, PyObject, PyTryFrom, ToPyObject};

    #[test]
    fn test_non_bmp() {
        Python::with_gil(|py| {
            let s = "\u{1F30F}";
            let py_string = s.to_object(py);
            assert_eq!(s, py_string.extract::<String>(py).unwrap());
        })
    }

    #[test]
    fn test_extract_str() {
        Python::with_gil(|py| {
            let s = "Hello Python";
            let py_string = s.to_object(py);

            let s2: &str = FromPyObject::extract(py_string.as_ref(py)).unwrap();
            assert_eq!(s, s2);
        })
    }

    #[test]
    fn test_extract_char() {
        Python::with_gil(|py| {
            let ch = '😃';
            let py_string = ch.to_object(py);
            let ch2: char = FromPyObject::extract(py_string.as_ref(py)).unwrap();
            assert_eq!(ch, ch2);
        })
    }

    #[test]
    fn test_extract_char_err() {
        Python::with_gil(|py| {
            let s = "Hello Python";
            let py_string = s.to_object(py);
            let err: crate::PyResult<char> = FromPyObject::extract(py_string.as_ref(py));
            assert!(err
                .unwrap_err()
                .to_string()
                .contains("expected a string of length 1"));
        })
    }

    #[test]
    fn test_to_str_ascii() {
        Python::with_gil(|py| {
            let s = "ascii 🐈";
            let obj: PyObject = PyStr::new(py, s).into();
            let py_string = <PyStr as PyTryFrom>::try_from(obj.as_ref(py)).unwrap();
            assert_eq!(s, py_string.to_str().unwrap());
        })
    }

    #[test]
    fn test_to_str_surrogate() {
        Python::with_gil(|py| {
            let obj: PyObject = py.eval(r#"'\ud800'"#, None, None).unwrap().into();
            let py_string = <PyStr as PyTryFrom>::try_from(obj.as_ref(py)).unwrap();
            assert!(py_string.to_str().is_err());
        })
    }

    #[test]
    fn test_to_str_unicode() {
        Python::with_gil(|py| {
            let s = "哈哈🐈";
            let obj: PyObject = PyStr::new(py, s).into();
            let py_string = <PyStr as PyTryFrom>::try_from(obj.as_ref(py)).unwrap();
            assert_eq!(s, py_string.to_str().unwrap());
        })
    }

    #[test]
    fn test_to_string_lossy() {
        Python::with_gil(|py| {
            let obj: PyObject = py
                .eval(r#"'🐈 Hello \ud800World'"#, None, None)
                .unwrap()
                .into();
            let py_string = <PyStr as PyTryFrom>::try_from(obj.as_ref(py)).unwrap();
            assert_eq!(py_string.to_string_lossy(), "🐈 Hello ���World");
        })
    }

    #[test]
    fn test_debug_string() {
        Python::with_gil(|py| {
            let v = "Hello\n".to_object(py);
            let s = <PyStr as PyTryFrom>::try_from(v.as_ref(py)).unwrap();
            assert_eq!(format!("{:?}", s), "'Hello\\n'");
        })
    }

    #[test]
    fn test_display_string() {
        Python::with_gil(|py| {
            let v = "Hello\n".to_object(py);
            let s = <PyStr as PyTryFrom>::try_from(v.as_ref(py)).unwrap();
            assert_eq!(format!("{}", s), "Hello\n");
        })
    }
}

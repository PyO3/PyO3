use crate::err::{self, PyResult};
use crate::instance::Borrowed;
use crate::types::any::PyAnyMethods;
use crate::{ffi, Bound, PyAny, PyNativeType, PyTypeInfo, Python};
use std::borrow::Cow;
#[cfg(not(any(Py_LIMITED_API, PyPy)))]
use std::ffi::CStr;

/// Represents a reference to a Python `type object`.
#[repr(transparent)]
pub struct PyType(PyAny);

pyobject_native_type_core!(PyType, pyobject_native_static_type_object!(ffi::PyType_Type), #checkfunction=ffi::PyType_Check);

impl PyType {
    /// Deprecated form of [`PyType::new_bound`].
    #[inline]
    #[cfg_attr(
        not(feature = "gil-refs"),
        deprecated(
            since = "0.21.0",
            note = "`new` will be replaced by `new_bound` in a future PyO3 version"
        )
    )]
    pub fn new<T: PyTypeInfo>(py: Python<'_>) -> &PyType {
        T::type_object_bound(py).into_gil_ref()
    }

    /// Creates a new type object.
    #[inline]
    pub fn new_bound<T: PyTypeInfo>(py: Python<'_>) -> Bound<'_, PyType> {
        T::type_object_bound(py)
    }

    /// Retrieves the underlying FFI pointer associated with this Python object.
    #[inline]
    pub fn as_type_ptr(&self) -> *mut ffi::PyTypeObject {
        self.as_borrowed().as_type_ptr()
    }

    /// Deprecated form of [`PyType::from_type_ptr_bound`].
    ///
    /// # Safety
    ///
    /// - The pointer must a valid non-null reference to a `PyTypeObject`.
    #[inline]
    #[cfg_attr(
        not(feature = "gil-refs"),
        deprecated(
            since = "0.21.0",
            note = "`from_type_ptr` will be replaced by `from_type_ptr_bound` in a future PyO3 version"
        )
    )]
    pub unsafe fn from_type_ptr(py: Python<'_>, p: *mut ffi::PyTypeObject) -> &PyType {
        Self::from_type_ptr_bound(py, &p).to_owned().into_gil_ref()
    }

    /// Converts the given FFI pointer into `&Bound<PyType>`, to use in safe code.
    ///
    /// # Safety
    /// - The pointer must be non-null.
    /// - The pointer must be valid for the entire of the lifetime 'a for which the reference is used,
    ///   as with `std::slice::from_raw_parts`.
    #[inline]
    pub unsafe fn from_type_ptr_bound<'a, 'py>(
        py: Python<'py>,
        p: &'a *mut ffi::PyTypeObject,
    ) -> &'a Bound<'py, PyType> {
        let object_ptr = &*(p as *const *mut ffi::PyTypeObject as *const *mut ffi::PyObject);
        Bound::ref_from_ptr(py, object_ptr).downcast_unchecked()
    }

    /// Gets the [qualified name](https://docs.python.org/3/glossary.html#term-qualified-name) of the `PyType`.
    pub fn qualname(&self) -> PyResult<String> {
        self.as_borrowed().qualname()
    }

    /// Gets the full name, which includes the module, of the `PyType`.
    pub fn name(&self) -> PyResult<Cow<'_, str>> {
        self.as_borrowed().name()
    }

    /// Checks whether `self` is a subclass of `other`.
    ///
    /// Equivalent to the Python expression `issubclass(self, other)`.
    pub fn is_subclass(&self, other: &PyAny) -> PyResult<bool> {
        self.as_borrowed().is_subclass(&other.as_borrowed())
    }

    /// Checks whether `self` is a subclass of type `T`.
    ///
    /// Equivalent to the Python expression `issubclass(self, T)`, if the type
    /// `T` is known at compile time.
    pub fn is_subclass_of<T>(&self) -> PyResult<bool>
    where
        T: PyTypeInfo,
    {
        self.as_borrowed().is_subclass_of::<T>()
    }
}

/// Implementation of functionality for [`PyType`].
///
/// These methods are defined for the `Bound<'py, PyType>` smart pointer, so to use method call
/// syntax these methods are separated into a trait, because stable Rust does not yet support
/// `arbitrary_self_types`.
#[doc(alias = "PyType")]
pub trait PyTypeMethods<'py> {
    /// Retrieves the underlying FFI pointer associated with this Python object.
    fn as_type_ptr(&self) -> *mut ffi::PyTypeObject;

    /// Gets the full name, which includes the module, of the `PyType`.
    fn name(&self) -> PyResult<Cow<'_, str>>;

    /// Gets the [qualified name](https://docs.python.org/3/glossary.html#term-qualified-name) of the `PyType`.
    fn qualname(&self) -> PyResult<String>;

    /// Checks whether `self` is a subclass of `other`.
    ///
    /// Equivalent to the Python expression `issubclass(self, other)`.
    fn is_subclass(&self, other: &Bound<'_, PyAny>) -> PyResult<bool>;

    /// Checks whether `self` is a subclass of type `T`.
    ///
    /// Equivalent to the Python expression `issubclass(self, T)`, if the type
    /// `T` is known at compile time.
    fn is_subclass_of<T>(&self) -> PyResult<bool>
    where
        T: PyTypeInfo;
}

impl<'py> PyTypeMethods<'py> for Bound<'py, PyType> {
    /// Retrieves the underlying FFI pointer associated with this Python object.
    #[inline]
    fn as_type_ptr(&self) -> *mut ffi::PyTypeObject {
        self.as_ptr() as *mut ffi::PyTypeObject
    }

    /// Gets the name of the `PyType`.
    fn name(&self) -> PyResult<Cow<'_, str>> {
        Borrowed::from(self).name()
    }

    fn qualname(&self) -> PyResult<String> {
        #[cfg(any(Py_LIMITED_API, PyPy, not(Py_3_11)))]
        let name = self.getattr(intern!(self.py(), "__qualname__"))?.extract();

        #[cfg(not(any(Py_LIMITED_API, PyPy, not(Py_3_11))))]
        let name = {
            use crate::ffi_ptr_ext::FfiPtrExt;
            let obj = unsafe {
                ffi::PyType_GetQualName(self.as_type_ptr()).assume_owned_or_err(self.py())?
            };

            obj.extract()
        };

        name
    }

    /// Checks whether `self` is a subclass of `other`.
    ///
    /// Equivalent to the Python expression `issubclass(self, other)`.
    fn is_subclass(&self, other: &Bound<'_, PyAny>) -> PyResult<bool> {
        let result = unsafe { ffi::PyObject_IsSubclass(self.as_ptr(), other.as_ptr()) };
        err::error_on_minusone(self.py(), result)?;
        Ok(result == 1)
    }

    /// Checks whether `self` is a subclass of type `T`.
    ///
    /// Equivalent to the Python expression `issubclass(self, T)`, if the type
    /// `T` is known at compile time.
    fn is_subclass_of<T>(&self) -> PyResult<bool>
    where
        T: PyTypeInfo,
    {
        self.is_subclass(&T::type_object_bound(self.py()))
    }
}

impl<'a> Borrowed<'a, '_, PyType> {
    fn name(self) -> PyResult<Cow<'a, str>> {
        #[cfg(not(any(Py_LIMITED_API, PyPy)))]
        {
            let ptr = self.as_type_ptr();

            let name = unsafe { CStr::from_ptr((*ptr).tp_name) }.to_str()?;

            #[cfg(Py_3_10)]
            if unsafe { ffi::PyType_HasFeature(ptr, ffi::Py_TPFLAGS_IMMUTABLETYPE) } != 0 {
                return Ok(Cow::Borrowed(name));
            }

            Ok(Cow::Owned(name.to_owned()))
        }

        #[cfg(any(Py_LIMITED_API, PyPy))]
        {
            let module = self.getattr(intern!(self.py(), "__module__"))?;

            #[cfg(not(Py_3_11))]
            let name = self.getattr(intern!(self.py(), "__name__"))?;

            #[cfg(Py_3_11)]
            let name = {
                use crate::ffi_ptr_ext::FfiPtrExt;
                unsafe { ffi::PyType_GetName(self.as_type_ptr()).assume_owned_or_err(self.py())? }
            };

            Ok(Cow::Owned(format!("{}.{}", module, name)))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{PyBool, PyLong};
    use crate::Python;

    #[test]
    fn test_type_is_subclass() {
        Python::with_gil(|py| {
            let bool_type = py.get_type::<PyBool>();
            let long_type = py.get_type::<PyLong>();
            assert!(bool_type.is_subclass(long_type).unwrap());
        });
    }

    #[test]
    fn test_type_is_subclass_of() {
        Python::with_gil(|py| {
            assert!(py.get_type::<PyBool>().is_subclass_of::<PyLong>().unwrap());
        });
    }
}

use crate::conversion::IntoPyObject;
#[cfg(feature = "experimental-inspect")]
use crate::inspect::types::TypeInfo;
use crate::types::list::new_from_iter;
use crate::{Bound, IntoPy, PyAny, PyErr, PyObject, Python, ToPyObject};

impl<T> ToPyObject for [T]
where
    T: ToPyObject,
{
    fn to_object(&self, py: Python<'_>) -> PyObject {
        let mut iter = self.iter().map(|e| e.to_object(py));
        let list = new_from_iter(py, &mut iter);
        list.into()
    }
}

impl<T> ToPyObject for Vec<T>
where
    T: ToPyObject,
{
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.as_slice().to_object(py)
    }
}

impl<T> IntoPy<PyObject> for Vec<T>
where
    T: IntoPy<PyObject>,
{
    fn into_py(self, py: Python<'_>) -> PyObject {
        let mut iter = self.into_iter().map(|e| e.into_py(py));
        let list = new_from_iter(py, &mut iter);
        list.into()
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        TypeInfo::list_of(T::type_output())
    }
}

impl<'py, T> IntoPyObject<'py> for Vec<T>
where
    T: IntoPyObject<'py>,
    PyErr: From<T::Error>,
{
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    /// Turns [`Vec<u8>`] into [`PyBytes`], all other `T`s will be turned into a [`PyList`]
    ///
    /// [`PyBytes`]: crate::types::PyBytes
    /// [`PyList`]: crate::types::PyList
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        T::iter_into_pyobject(self, py, crate::conversion::private::Token)
    }
}

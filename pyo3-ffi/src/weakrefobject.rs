use crate::object::*;
use std::os::raw::c_int;

#[cfg(all(not(PyPy), Py_LIMITED_API))]
opaque_struct!(PyWeakReference);

#[cfg(all(not(PyPy), not(Py_LIMITED_API)))]
pub struct PyWeakReference {
    pub ob_base: PyObject,
    pub wr_object: *mut PyObject,
    pub wr_callback: *mut PyObject,
    pub hash: crate::Py_hash_t,
    pub wr_prev: *mut PyWeakReference,
    pub wr_next: *mut PyWeakReference,
}

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    pub static mut _PyWeakref_RefType: PyTypeObject;
    pub static mut _PyWeakref_ProxyType: PyTypeObject;
    pub static mut _PyWeakref_CallableProxyType: PyTypeObject;

    #[cfg(PyPy)]
    #[link_name = "PyPyWeakref_CheckRef"]
    pub fn PyWeakref_CheckRef(op: *mut PyObject) -> c_int;

    #[cfg(PyPy)]
    #[link_name = "PyPyWeakref_CheckRefExact"]
    pub fn PyWeakref_CheckRefExact(op: *mut PyObject) -> c_int;

    #[cfg(PyPy)]
    #[link_name = "PyPyWeakref_CheckProxy"]
    pub fn PyWeakref_CheckProxy(op: *mut PyObject) -> c_int;
}

#[inline]
#[cfg(not(PyPy))]
pub unsafe fn PyWeakref_CheckRef(op: *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, addr_of_mut_shim!(_PyWeakref_RefType))
}

#[inline]
#[cfg(not(PyPy))]
pub unsafe fn PyWeakref_CheckRefExact(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == addr_of_mut_shim!(_PyWeakref_RefType)) as c_int
}

#[inline]
#[cfg(not(PyPy))]
pub unsafe fn PyWeakref_CheckProxy(op: *mut PyObject) -> c_int {
    ((Py_TYPE(op) == addr_of_mut_shim!(_PyWeakref_ProxyType))
        || (Py_TYPE(op) == addr_of_mut_shim!(_PyWeakref_CallableProxyType))) as c_int
}

#[inline]
pub unsafe fn PyWeakref_Check(op: *mut PyObject) -> c_int {
    (PyWeakref_CheckRef(op) != 0 || PyWeakref_CheckProxy(op) != 0) as c_int
}

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyWeakref_NewRef")]
    pub fn PyWeakref_NewRef(ob: *mut PyObject, callback: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyWeakref_NewProxy")]
    pub fn PyWeakref_NewProxy(ob: *mut PyObject, callback: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyWeakref_GetObject")]
    pub fn PyWeakref_GetObject(_ref: *mut PyObject) -> *mut PyObject;
}

use libc::size_t;
use std::os::raw::{c_int, c_void};

use crate::object::*;
use crate::pyport::Py_ssize_t;

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyObject_Malloc")]
    pub fn PyObject_Malloc(size: size_t) -> *mut c_void;
    #[cfg_attr(PyPy, link_name = "PyPyObject_Calloc")]
    pub fn PyObject_Calloc(nelem: size_t, elsize: size_t) -> *mut c_void;
    #[cfg_attr(PyPy, link_name = "PyPyObject_Realloc")]
    pub fn PyObject_Realloc(ptr: *mut c_void, new_size: size_t) -> *mut c_void;
    #[cfg_attr(PyPy, link_name = "PyPyObject_Free")]
    pub fn PyObject_Free(ptr: *mut c_void);

    // skipped PyObject_MALLOC
    // skipped PyObject_REALLOC
    // skipped PyObject_FREE
    // skipped PyObject_Del
    // skipped PyObject_DEL

    #[cfg_attr(PyPy, link_name = "PyPyObject_Init")]
    pub fn PyObject_Init(arg1: *mut PyObject, arg2: *mut PyTypeObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyObject_InitVar")]
    pub fn PyObject_InitVar(
        arg1: *mut PyVarObject,
        arg2: *mut PyTypeObject,
        arg3: Py_ssize_t,
    ) -> *mut PyVarObject;

    // skipped PyObject_INIT
    // skipped PyObject_INIT_VAR

    #[cfg_attr(PyPy, link_name = "_PyPyObject_New")]
    fn _PyObject_New(arg1: *mut PyTypeObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "_PyPyObject_NewVar")]
    fn _PyObject_NewVar(arg1: *mut PyTypeObject, arg2: Py_ssize_t) -> *mut PyVarObject;
}

#[inline(always)]
pub unsafe fn PyObject_New(t: *mut PyTypeObject) -> *mut PyObject {
    _PyObject_New(t)
}

#[inline(always)]
pub unsafe fn PyObject_NEW(t: *mut PyTypeObject) -> *mut PyObject {
    PyObject_New(t)
}

#[inline(always)]
pub unsafe fn PyObject_NewVar(t: *mut PyTypeObject, size: Py_ssize_t) -> *mut PyVarObject {
    _PyObject_NewVar(t, size)
}

#[inline(always)]
pub unsafe fn PyObject_NEW_VAR(t: *mut PyTypeObject, size: Py_ssize_t) -> *mut PyVarObject {
    PyObject_NewVar(t, size)
}

extern "C" {
    pub fn PyGC_Collect() -> Py_ssize_t;

    #[cfg(Py_3_10)]
    #[cfg_attr(PyPy, link_name = "PyPyGC_Enable")]
    pub fn PyGC_Enable() -> c_int;

    #[cfg(Py_3_10)]
    #[cfg_attr(PyPy, link_name = "PyPyGC_Disable")]
    pub fn PyGC_Disable() -> c_int;

    #[cfg(Py_3_10)]
    #[cfg_attr(PyPy, link_name = "PyPyGC_IsEnabled")]
    pub fn PyGC_IsEnabled() -> c_int;
}

#[inline]
pub unsafe fn PyType_IS_GC(t: *mut PyTypeObject) -> c_int {
    PyType_HasFeature(t, Py_TPFLAGS_HAVE_GC)
}

extern "C" {
    fn _PyObject_GC_Resize(arg1: *mut PyVarObject, arg2: Py_ssize_t) -> *mut PyVarObject;
}

#[inline(always)]
pub unsafe fn PyObject_GC_Resize<T>(op: *mut PyVarObject, n: Py_ssize_t) -> *mut T {
    _PyObject_GC_Resize(op, n).cast::<T>()
}
// skipped PyObject_GC_Resize
extern "C" {
    #[cfg_attr(PyPy, link_name = "_PyPyObject_GC_New")]
    fn _PyObject_GC_New(arg1: *mut PyTypeObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "_PyPyObject_GC_NewVar")]
    fn _PyObject_GC_NewVar(arg1: *mut PyTypeObject, arg2: Py_ssize_t) -> *mut PyVarObject;
    #[cfg(not(PyPy))]
    pub fn PyObject_GC_Track(arg1: *mut c_void);
    #[cfg(not(PyPy))]
    pub fn PyObject_GC_UnTrack(arg1: *mut c_void);
    #[cfg_attr(PyPy, link_name = "PyPyObject_GC_Del")]
    pub fn PyObject_GC_Del(arg1: *mut c_void);

}

#[inline(always)]
pub unsafe fn PyObject_GC_New<T>(typeobj: *mut PyTypeObject) -> *mut T {
    _PyObject_GC_New(typeobj).cast::<T>()
}

#[inline(always)]
pub unsafe fn PyObject_GC_NewVar<T>(typeobj: *mut PyTypeObject, n: Py_ssize_t) -> *mut T {
    _PyObject_GC_NewVar(typeobj, n).cast::<T>()
}

extern "C" {
    #[cfg(any(all(Py_3_9, not(PyPy)), Py_3_10))] // added in 3.9, or 3.10 on PyPy
    #[cfg_attr(PyPy, link_name = "PyPyObject_GC_IsTracked")]
    pub fn PyObject_GC_IsTracked(arg1: *mut PyObject) -> c_int;
    #[cfg(any(all(Py_3_9, not(PyPy)), Py_3_10))] // added in 3.9, or 3.10 on PyPy
    #[cfg_attr(PyPy, link_name = "PyPyObject_GC_IsFinalized")]
    pub fn PyObject_GC_IsFinalized(arg1: *mut PyObject) -> c_int;
}

// skipped Py_VISIT

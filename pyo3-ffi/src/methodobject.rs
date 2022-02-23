use crate::object::{PyObject, PyTypeObject, Py_TYPE};
#[cfg(Py_3_9)]
use crate::PyObject_TypeCheck;
use std::mem;
use std::os::raw::{c_char, c_int};

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyCFunction_Type")]
    pub static mut PyCFunction_Type: PyTypeObject;
}

#[cfg(Py_3_9)]
#[inline]
pub unsafe fn PyCFunction_CheckExact(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == addr_of_mut_shim!(PyCFunction_Type)) as c_int
}

#[cfg(Py_3_9)]
#[inline]
pub unsafe fn PyCFunction_Check(op: *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, addr_of_mut_shim!(PyCFunction_Type))
}

#[cfg(not(Py_3_9))]
#[inline]
pub unsafe fn PyCFunction_Check(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == addr_of_mut_shim!(PyCFunction_Type)) as c_int
}

pub type PyCFunction =
    unsafe extern "C" fn(slf: *mut PyObject, args: *mut PyObject) -> *mut PyObject;

#[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
pub type _PyCFunctionFast = unsafe extern "C" fn(
    slf: *mut PyObject,
    args: *mut *mut PyObject,
    nargs: crate::pyport::Py_ssize_t,
) -> *mut PyObject;

pub type PyCFunctionWithKeywords = unsafe extern "C" fn(
    slf: *mut PyObject,
    args: *mut PyObject,
    kwds: *mut PyObject,
) -> *mut PyObject;

#[cfg(not(Py_LIMITED_API))]
pub type _PyCFunctionFastWithKeywords = unsafe extern "C" fn(
    slf: *mut PyObject,
    args: *const *mut PyObject,
    nargs: crate::pyport::Py_ssize_t,
    kwnames: *mut PyObject,
) -> *mut PyObject;

#[cfg(all(Py_3_9, not(Py_LIMITED_API)))]
pub type PyCMethod = unsafe extern "C" fn(
    slf: *mut PyObject,
    defining_class: *mut PyTypeObject,
    args: *const *mut PyObject,
    nargs: crate::pyport::Py_ssize_t,
    kwnames: *mut PyObject,
) -> *mut PyObject;

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyCFunction_GetFunction")]
    pub fn PyCFunction_GetFunction(f: *mut PyObject) -> Option<PyCFunction>;
    pub fn PyCFunction_GetSelf(f: *mut PyObject) -> *mut PyObject;
    pub fn PyCFunction_GetFlags(f: *mut PyObject) -> c_int;
    #[cfg_attr(Py_3_9, deprecated(note = "Python 3.9"))]
    pub fn PyCFunction_Call(
        f: *mut PyObject,
        args: *mut PyObject,
        kwds: *mut PyObject,
    ) -> *mut PyObject;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyMethodDef {
    pub ml_name: *const c_char,
    pub ml_meth: PyMethodDefPointer,
    pub ml_flags: c_int,
    pub ml_doc: *const c_char,
}

/// Function types used to implement Python callables.
///
/// This function pointer must be accompanied by the correct [ml_flags](PyMethodDef::ml_flags),
/// otherwise the behavior is undefined.
///
/// See the [Python C API documentation][1] for more information.
///
/// [1]: https://docs.python.org/3/c-api/structures.html#implementing-functions-and-methods
#[repr(C)]
#[derive(Copy, Clone)]
pub union PyMethodDefPointer {
    /// This variant corresponds with [`METH_VARARGS`] *or* [`METH_NOARGS`] *or* [`METH_O`].
    pub PyCFunction: PyCFunction,

    /// This variant corresponds with [`METH_VARARGS`] | [`METH_KEYWORDS`].
    pub PyCFunctionWithKeywords: PyCFunctionWithKeywords,

    /// This variant corresponds with [`METH_FASTCALL`].
    #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
    pub _PyCFunctionFast: _PyCFunctionFast,

    /// This variant corresponds with [`METH_FASTCALL`] | [`METH_KEYWORDS`].
    #[cfg(not(Py_LIMITED_API))]
    pub _PyCFunctionFastWithKeywords: _PyCFunctionFastWithKeywords,

    /// This variant corresponds with [`METH_METHOD`] | [`METH_FASTCALL`] | [`METH_KEYWORDS`].
    #[cfg(all(Py_3_9, not(Py_LIMITED_API)))]
    pub PyCMethod: PyCMethod,
}

// TODO: This can be a const assert on Rust 1.57
const _: () =
    [()][mem::size_of::<PyMethodDefPointer>() - mem::size_of::<Option<extern "C" fn()>>()];

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyCFunction_New")]
    pub fn PyCFunction_New(ml: *mut PyMethodDef, slf: *mut PyObject) -> *mut PyObject;

    #[cfg_attr(PyPy, link_name = "PyPyCFunction_NewEx")]
    pub fn PyCFunction_NewEx(
        ml: *mut PyMethodDef,
        slf: *mut PyObject,
        module: *mut PyObject,
    ) -> *mut PyObject;
}

// skipped non-limited / 3.9 PyCMethod_New

/* Flag passed to newmethodobject */
pub const METH_VARARGS: c_int = 0x0001;
pub const METH_KEYWORDS: c_int = 0x0002;
/* METH_NOARGS and METH_O must not be combined with the flags above. */
pub const METH_NOARGS: c_int = 0x0004;
pub const METH_O: c_int = 0x0008;

/* METH_CLASS and METH_STATIC are a little different; these control
the construction of methods for a class.  These cannot be used for
functions in modules. */
pub const METH_CLASS: c_int = 0x0010;
pub const METH_STATIC: c_int = 0x0020;

/* METH_COEXIST allows a method to be entered eventhough a slot has
already filled the entry.  When defined, the flag allows a separate
method, "__contains__" for example, to coexist with a defined
slot like sq_contains. */

pub const METH_COEXIST: c_int = 0x0040;

/* METH_FASTCALL indicates the PEP 590 Vectorcall calling format. It may
be specified alone or with METH_KEYWORDS. */
#[cfg(not(Py_LIMITED_API))]
pub const METH_FASTCALL: c_int = 0x0080;

// skipped METH_STACKLESS

#[cfg(all(Py_3_9, not(Py_LIMITED_API)))]
pub const METH_METHOD: c_int = 0x0200;

extern "C" {
    #[cfg(not(Py_3_9))]
    pub fn PyCFunction_ClearFreeList() -> c_int;
}

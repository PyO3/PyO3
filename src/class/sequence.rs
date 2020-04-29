// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python Sequence Interface
//! Trait and support implementation for implementing sequence

use crate::conversion::{FromPyObject, IntoPy};
use crate::err::{PyErr, PyResult};
use crate::gil::GILPool;
use crate::objectprotocol::ObjectProtocol;
use crate::{callback, exceptions, ffi, run_callback, PyAny, PyCell, PyClass, PyObject};
use std::os::raw::c_int;

/// Sequence interface
#[allow(unused_variables)]
pub trait PySequenceProtocol<'p>: PyClass<'p> + Sized {
    fn __len__(&'p self) -> Self::Result
    where
        Self: PySequenceLenProtocol<'p>,
    {
        unimplemented!()
    }

    fn __getitem__(&'p self, idx: Self::Index) -> Self::Result
    where
        Self: PySequenceGetItemProtocol<'p>,
    {
        unimplemented!()
    }

    fn __setitem__(&'p mut self, idx: Self::Index, value: Self::Value) -> Self::Result
    where
        Self: PySequenceSetItemProtocol<'p>,
    {
        unimplemented!()
    }

    fn __delitem__(&'p mut self, idx: Self::Index) -> Self::Result
    where
        Self: PySequenceDelItemProtocol<'p>,
    {
        unimplemented!()
    }

    fn __contains__(&'p self, item: Self::Item) -> Self::Result
    where
        Self: PySequenceContainsProtocol<'p>,
    {
        unimplemented!()
    }

    fn __concat__(&'p self, other: Self::Other) -> Self::Result
    where
        Self: PySequenceConcatProtocol<'p>,
    {
        unimplemented!()
    }

    fn __repeat__(&'p self, count: Self::Index) -> Self::Result
    where
        Self: PySequenceRepeatProtocol<'p>,
    {
        unimplemented!()
    }

    fn __inplace_concat__(&'p mut self, other: Self::Other) -> Self::Result
    where
        Self: PySequenceInplaceConcatProtocol<'p>,
    {
        unimplemented!()
    }

    fn __inplace_repeat__(&'p mut self, count: Self::Index) -> Self::Result
    where
        Self: PySequenceInplaceRepeatProtocol<'p>,
    {
        unimplemented!()
    }
}

// The following are a bunch of marker traits used to detect
// the existance of a slotted method.

pub trait PySequenceLenProtocol<'p>: PySequenceProtocol<'p> {
    type Result: Into<PyResult<usize>>;
}

pub trait PySequenceGetItemProtocol<'p>: PySequenceProtocol<'p> {
    type Index: FromPyObject<'p, 'p> + From<isize>;
    type Success: IntoPy<PyObject>;
    type Result: Into<PyResult<Self::Success>>;
}

pub trait PySequenceSetItemProtocol<'p>: PySequenceProtocol<'p> {
    type Index: FromPyObject<'p, 'p> + From<isize>;
    type Value: FromPyObject<'p, 'p>;
    type Result: Into<PyResult<()>>;
}

pub trait PySequenceDelItemProtocol<'p>: PySequenceProtocol<'p> {
    type Index: FromPyObject<'p, 'p> + From<isize>;
    type Result: Into<PyResult<()>>;
}

pub trait PySequenceContainsProtocol<'p>: PySequenceProtocol<'p> {
    type Item: FromPyObject<'p, 'p>;
    type Result: Into<PyResult<bool>>;
}

pub trait PySequenceConcatProtocol<'p>: PySequenceProtocol<'p> {
    type Other: FromPyObject<'p, 'p>;
    type Success: IntoPy<PyObject>;
    type Result: Into<PyResult<Self::Success>>;
}

pub trait PySequenceRepeatProtocol<'p>: PySequenceProtocol<'p> {
    type Index: FromPyObject<'p, 'p> + From<isize>;
    type Success: IntoPy<PyObject>;
    type Result: Into<PyResult<Self::Success>>;
}

pub trait PySequenceInplaceConcatProtocol<'p>: PySequenceProtocol<'p> + IntoPy<PyObject> {
    type Other: FromPyObject<'p, 'p>;
    type Result: Into<PyResult<Self>>;
}

pub trait PySequenceInplaceRepeatProtocol<'p>: PySequenceProtocol<'p> + IntoPy<PyObject> {
    type Index: FromPyObject<'p, 'p> + From<isize>;
    type Result: Into<PyResult<Self>>;
}

#[doc(hidden)]
pub trait PySequenceProtocolImpl {
    fn tp_as_sequence() -> Option<ffi::PySequenceMethods>;
}

impl<T> PySequenceProtocolImpl for T {
    default fn tp_as_sequence() -> Option<ffi::PySequenceMethods> {
        None
    }
}

impl<'p, T> PySequenceProtocolImpl for T
where
    T: PySequenceProtocol<'p>,
{
    fn tp_as_sequence() -> Option<ffi::PySequenceMethods> {
        Some(ffi::PySequenceMethods {
            sq_length: Self::sq_length(),
            sq_concat: Self::sq_concat(),
            sq_repeat: Self::sq_repeat(),
            sq_item: Self::sq_item(),
            was_sq_slice: ::std::ptr::null_mut(),
            sq_ass_item: sq_ass_item_impl::sq_ass_item::<Self>(),
            was_sq_ass_slice: ::std::ptr::null_mut(),
            sq_contains: Self::sq_contains(),
            sq_inplace_concat: Self::sq_inplace_concat(),
            sq_inplace_repeat: Self::sq_inplace_repeat(),
        })
    }
}

trait PySequenceLenProtocolImpl {
    fn sq_length() -> Option<ffi::lenfunc>;
}

impl<'p, T> PySequenceLenProtocolImpl for T
where
    T: PySequenceProtocol<'p>,
{
    default fn sq_length() -> Option<ffi::lenfunc> {
        None
    }
}

impl<T> PySequenceLenProtocolImpl for T
where
    T: for<'p> PySequenceLenProtocol<'p>,
{
    fn sq_length() -> Option<ffi::lenfunc> {
        py_len_func!(PySequenceLenProtocol, T::__len__)
    }
}

trait PySequenceGetItemProtocolImpl {
    fn sq_item() -> Option<ffi::ssizeargfunc>;
}

impl<'p, T> PySequenceGetItemProtocolImpl for T
where
    T: PySequenceProtocol<'p>,
{
    default fn sq_item() -> Option<ffi::ssizeargfunc> {
        None
    }
}

impl<T> PySequenceGetItemProtocolImpl for T
where
    T: for<'p> PySequenceGetItemProtocol<'p>,
{
    fn sq_item() -> Option<ffi::ssizeargfunc> {
        py_ssizearg_func!(PySequenceGetItemProtocol, T::__getitem__)
    }
}

/// It can be possible to delete and set items (PySequenceSetItemProtocol and
/// PySequenceDelItemProtocol implemented), only to delete (PySequenceDelItemProtocol implemented)
/// or no deleting or setting is possible
mod sq_ass_item_impl {
    use super::*;

    /// ssizeobjargproc PySequenceMethods.sq_ass_item
    ///
    /// This function is used by PySequence_SetItem() and has the same signature. It is also used
    /// by PyObject_SetItem() and PyObject_DelItem(), after trying the item assignment and deletion
    /// via the mp_ass_subscript slot. This slot may be left to NULL if the object does not support
    /// item assignment and deletion.
    pub(super) fn sq_ass_item<'p, T>() -> Option<ffi::ssizeobjargproc>
    where
        T: PySequenceProtocol<'p>,
    {
        if let Some(del_set_item) = T::del_set_item() {
            Some(del_set_item)
        } else if let Some(del_item) = T::del_item() {
            Some(del_item)
        } else if let Some(set_item) = T::set_item() {
            Some(set_item)
        } else {
            None
        }
    }

    trait SetItem {
        fn set_item() -> Option<ffi::ssizeobjargproc>;
    }

    impl<'p, T> SetItem for T
    where
        T: PySequenceProtocol<'p>,
    {
        default fn set_item() -> Option<ffi::ssizeobjargproc> {
            None
        }
    }

    impl<T> SetItem for T
    where
        T: for<'p> PySequenceSetItemProtocol<'p>,
    {
        fn set_item() -> Option<ffi::ssizeobjargproc> {
            unsafe extern "C" fn wrap<T>(
                slf: *mut ffi::PyObject,
                key: ffi::Py_ssize_t,
                value: *mut ffi::PyObject,
            ) -> c_int
            where
                T: for<'p> PySequenceSetItemProtocol<'p>,
            {
                let pool = GILPool::new();
                let py = pool.python();
                run_callback(py, || {
                    let slf = py.from_borrowed_ptr::<PyCell<T>>(slf);

                    if value.is_null() {
                        return Err(PyErr::new::<exceptions::NotImplementedError, _>(format!(
                            "Item deletion is not supported by {:?}",
                            stringify!(T)
                        )));
                    }

                    let mut slf = slf.try_borrow_mut()?;
                    let value = py.from_borrowed_ptr::<PyAny>(value);
                    let value = value.extract()?;
                    let result = slf.__setitem__(key.into(), value).into();
                    callback::convert(py, result)
                })
            }
            Some(wrap::<T>)
        }
    }

    trait DelItem {
        fn del_item() -> Option<ffi::ssizeobjargproc>;
    }

    impl<'p, T> DelItem for T
    where
        T: PySequenceProtocol<'p>,
    {
        default fn del_item() -> Option<ffi::ssizeobjargproc> {
            None
        }
    }

    impl<T> DelItem for T
    where
        T: for<'p> PySequenceDelItemProtocol<'p>,
    {
        fn del_item() -> Option<ffi::ssizeobjargproc> {
            unsafe extern "C" fn wrap<T>(
                slf: *mut ffi::PyObject,
                key: ffi::Py_ssize_t,
                value: *mut ffi::PyObject,
            ) -> c_int
            where
                T: for<'p> PySequenceDelItemProtocol<'p>,
            {
                let pool = GILPool::new();
                let py = pool.python();
                run_callback(py, || {
                    let slf = py.from_borrowed_ptr::<PyCell<T>>(slf);

                    let result = if value.is_null() {
                        slf.borrow_mut().__delitem__(key.into()).into()
                    } else {
                        Err(PyErr::new::<exceptions::NotImplementedError, _>(format!(
                            "Item assignment not supported by {:?}",
                            stringify!(T)
                        )))
                    };

                    callback::convert(py, result)
                })
            }
            Some(wrap::<T>)
        }
    }

    trait DelSetItem {
        fn del_set_item() -> Option<ffi::ssizeobjargproc>;
    }

    impl<'p, T> DelSetItem for T
    where
        T: PySequenceProtocol<'p>,
    {
        default fn del_set_item() -> Option<ffi::ssizeobjargproc> {
            None
        }
    }

    impl<T> DelSetItem for T
    where
        T: for<'p> PySequenceSetItemProtocol<'p> + for<'p> PySequenceDelItemProtocol<'p>,
    {
        fn del_set_item() -> Option<ffi::ssizeobjargproc> {
            unsafe extern "C" fn wrap<T>(
                slf: *mut ffi::PyObject,
                key: ffi::Py_ssize_t,
                value: *mut ffi::PyObject,
            ) -> c_int
            where
                T: for<'p> PySequenceSetItemProtocol<'p> + for<'p> PySequenceDelItemProtocol<'p>,
            {
                let pool = GILPool::new();
                let py = pool.python();
                run_callback(py, || {
                    let slf = py.from_borrowed_ptr::<PyCell<T>>(slf);

                    let result = if value.is_null() {
                        call_mut!(slf, __delitem__; key.into())
                    } else {
                        let value = py.from_borrowed_ptr::<PyAny>(value);
                        let mut slf_ = slf.try_borrow_mut()?;
                        let value = value.extract()?;
                        slf_.__setitem__(key.into(), value).into()
                    };
                    callback::convert(py, result)
                })
            }
            Some(wrap::<T>)
        }
    }
}

trait PySequenceContainsProtocolImpl {
    fn sq_contains() -> Option<ffi::objobjproc>;
}

impl<'p, T> PySequenceContainsProtocolImpl for T
where
    T: PySequenceProtocol<'p>,
{
    default fn sq_contains() -> Option<ffi::objobjproc> {
        None
    }
}

impl<T> PySequenceContainsProtocolImpl for T
where
    T: for<'p> PySequenceContainsProtocol<'p>,
{
    fn sq_contains() -> Option<ffi::objobjproc> {
        py_binary_func!(PySequenceContainsProtocol, T::__contains__, c_int)
    }
}

trait PySequenceConcatProtocolImpl {
    fn sq_concat() -> Option<ffi::binaryfunc>;
}

impl<'p, T> PySequenceConcatProtocolImpl for T
where
    T: PySequenceProtocol<'p>,
{
    default fn sq_concat() -> Option<ffi::binaryfunc> {
        None
    }
}

impl<T> PySequenceConcatProtocolImpl for T
where
    T: for<'p> PySequenceConcatProtocol<'p>,
{
    fn sq_concat() -> Option<ffi::binaryfunc> {
        py_binary_func!(PySequenceConcatProtocol, T::__concat__)
    }
}

trait PySequenceRepeatProtocolImpl {
    fn sq_repeat() -> Option<ffi::ssizeargfunc>;
}

impl<'p, T> PySequenceRepeatProtocolImpl for T
where
    T: PySequenceProtocol<'p>,
{
    default fn sq_repeat() -> Option<ffi::ssizeargfunc> {
        None
    }
}

impl<T> PySequenceRepeatProtocolImpl for T
where
    T: for<'p> PySequenceRepeatProtocol<'p>,
{
    fn sq_repeat() -> Option<ffi::ssizeargfunc> {
        py_ssizearg_func!(PySequenceRepeatProtocol, T::__repeat__)
    }
}

trait PySequenceInplaceConcatProtocolImpl {
    fn sq_inplace_concat() -> Option<ffi::binaryfunc>;
}

impl<'p, T> PySequenceInplaceConcatProtocolImpl for T
where
    T: PySequenceProtocol<'p>,
{
    default fn sq_inplace_concat() -> Option<ffi::binaryfunc> {
        None
    }
}

impl<T> PySequenceInplaceConcatProtocolImpl for T
where
    T: for<'p> PySequenceInplaceConcatProtocol<'p>,
{
    fn sq_inplace_concat() -> Option<ffi::binaryfunc> {
        py_binary_func!(
            PySequenceInplaceConcatProtocol,
            T::__inplace_concat__,
            *mut ffi::PyObject,
            call_mut
        )
    }
}

trait PySequenceInplaceRepeatProtocolImpl {
    fn sq_inplace_repeat() -> Option<ffi::ssizeargfunc>;
}

impl<'p, T> PySequenceInplaceRepeatProtocolImpl for T
where
    T: PySequenceProtocol<'p>,
{
    default fn sq_inplace_repeat() -> Option<ffi::ssizeargfunc> {
        None
    }
}

impl<T> PySequenceInplaceRepeatProtocolImpl for T
where
    T: for<'p> PySequenceInplaceRepeatProtocol<'p>,
{
    fn sq_inplace_repeat() -> Option<ffi::ssizeargfunc> {
        py_ssizearg_func!(
            PySequenceInplaceRepeatProtocol,
            T::__inplace_repeat__,
            call_mut
        )
    }
}

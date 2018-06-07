// Copyright (c) 2017-present PyO3 Project and Contributors

#[macro_use] mod exc_impl;

pub use self::typeobject::PyType;
pub use self::module::PyModule;
pub use self::iterator::PyIterator;
pub use self::boolobject::PyBool;
pub use self::bytearray::PyByteArray;
pub use self::tuple::PyTuple;
pub use self::dict::PyDict;
pub use self::list::PyList;
pub use self::floatob::PyFloat;
pub use self::sequence::PySequence;
pub use self::slice::{PySlice, PySliceIndices};
pub use self::set::{PySet, PyFrozenSet};
pub use self::stringdata::PyStringData;

#[cfg(Py_3)]
pub use self::string::{PyBytes, PyString, PyUnicode};

#[cfg(not(Py_3))]
pub use self::string2::{PyBytes, PyString, PyUnicode};

#[cfg(Py_3)]
pub use self::num3::PyLong;
#[cfg(Py_3)]
pub use self::num3::PyLong as PyInt;

#[cfg(not(Py_3))]
pub use self::num2::{PyInt, PyLong};


/// Implements a typesafe conversions throught [FromPyObject], given a typecheck function as second
/// parameter
#[macro_export]
macro_rules! pyobject_downcast(
    ($name: ident, $checkfunction: ident) => (
        impl<'a> $crate::FromPyObject<'a> for &'a $name
        {
            /// Extracts `Self` from the source `PyObject`.
            #[cfg_attr(feature = "cargo-clippy", allow(useless_transmute))]
            fn extract(ob: &'a $crate::PyObjectRef) -> $crate::PyResult<Self>
            {
                unsafe {
                    if $crate::ffi::$checkfunction(ob.as_ptr()) != 0 {
                        Ok($crate::std::mem::transmute(ob))
                    } else {
                        Err($crate::PyDowncastError.into())
                    }
                }
            }
        }
    );
);

#[macro_export]
macro_rules! pyobject_native_type_named(
    ($name: ident) => {
        impl $crate::PyNativeType for $name {}

        impl $crate::std::convert::AsRef<$crate::PyObjectRef> for $name {
            #[cfg_attr(feature = "cargo-clippy", allow(useless_transmute))]
            fn as_ref(&self) -> &$crate::PyObjectRef {
                unsafe{$crate::std::mem::transmute(self)}
            }
        }

        impl $crate::PyObjectWithToken for $name {
            #[inline(always)]
            fn py(&self) -> $crate::Python {
                unsafe { $crate::Python::assume_gil_acquired() }
            }
        }

        impl $crate::python::ToPyPointer for $name {
            /// Gets the underlying FFI pointer, returns a borrowed pointer.
            #[inline]
            fn as_ptr(&self) -> *mut $crate::ffi::PyObject {
                self.0.as_ptr()
            }
        }

        impl PartialEq for $name {
            #[inline]
            fn eq(&self, o: &$name) -> bool {
                self.as_ptr() == o.as_ptr()
            }
        }
    };
);

#[macro_export]
macro_rules! pyobject_native_type(
    ($name: ident, $typeobject: ident, $checkfunction: ident) => {
        pyobject_native_type_named!($name);
        pyobject_native_type_convert!($name, $typeobject, $checkfunction);
        pyobject_downcast!($name, $checkfunction);

        impl<'a> $crate::std::convert::From<&'a $name> for &'a $crate::PyObjectRef {
            fn from(ob: &'a $name) -> Self {
                unsafe{$crate::std::mem::transmute(ob)}
            }
        }
    };
);

#[macro_export]
macro_rules! pyobject_native_type_convert(
    ($name: ident, $typeobject: ident, $checkfunction: ident) => {
        impl $crate::typeob::PyTypeInfo for $name {
            type Type = ();
            type BaseType = $crate::PyObjectRef;

            const NAME: &'static str = stringify!($name);
            const SIZE: usize = $crate::std::mem::size_of::<$crate::ffi::PyObject>();
            const OFFSET: isize = 0;

            #[inline]
            unsafe fn type_object() -> &'static mut $crate::ffi::PyTypeObject {
                &mut $crate::ffi::$typeobject
            }

            #[cfg_attr(feature = "cargo-clippy", allow(not_unsafe_ptr_arg_deref))]
            fn is_instance(ptr: *mut $crate::ffi::PyObject) -> bool {
                #[allow(unused_unsafe)]
                unsafe { $crate::ffi::$checkfunction(ptr) > 0 }
            }
        }

        impl $crate::typeob::PyTypeObject for $name {
            #[inline(always)]
            fn init_type() {}

            #[inline]
            fn type_object() -> $crate::Py<$crate::PyType> {
                $crate::PyType::new::<$name>()
            }
        }

        impl $crate::ToPyObject for $name
        {
            #[inline]
            fn to_object(&self, py: $crate::Python) -> $crate::PyObject {
                unsafe {$crate::PyObject::from_borrowed_ptr(py, self.0.as_ptr())}
            }
        }

        impl $crate::ToBorrowedObject for $name
        {
            #[inline]
            fn with_borrowed_ptr<F, R>(&self, _py: $crate::Python, f: F) -> R
                where F: FnOnce(*mut $crate::ffi::PyObject) -> R
            {
                f(self.0.as_ptr())
            }
        }

        impl $crate::std::fmt::Debug for $name {
            fn fmt(&self, f: &mut $crate::std::fmt::Formatter)
                   -> Result<(), $crate::std::fmt::Error>
            {
                use $crate::ObjectProtocol;
                let s = try!(self.repr().map_err(|_| $crate::std::fmt::Error));
                f.write_str(&s.to_string_lossy())
            }
        }

        impl $crate::std::fmt::Display for $name {
            fn fmt(&self, f: &mut $crate::std::fmt::Formatter)
                   -> Result<(), $crate::std::fmt::Error>
            {
                use $crate::ObjectProtocol;
                let s = try!(self.str().map_err(|_| $crate::std::fmt::Error));
                f.write_str(&s.to_string_lossy())
            }
        }
    };
);

/// Implements [FromPyObject] and (currently deactivated until it's stabillized) a
/// [TryFrom](std::convert::TryFrom) implementation, given a function body that performs the actual
/// conversion
#[macro_export]
macro_rules! pyobject_extract(
    ($obj:ident to $t:ty => $body: block) => {
        impl<'source> $crate::FromPyObject<'source> for $t
        {
            fn extract($obj: &'source $crate::PyObjectRef) -> $crate::PyResult<Self>
            {
                $body
            }
        }

        #[cfg(feature = "try_from")]
        impl<'source> $crate::std::convert::TryFrom<&'source $crate::PyObjectRef> for $t
        {
            type Error = $crate::PyErr;

            fn try_from($obj: &$crate::PyObjectRef) -> Result<Self, $crate::PyErr>
            {
                $body
            }
        }
    }
);


use python::ToPyPointer;

/// Represents general python instance.
pub struct PyObjectRef(::PyObject);
pyobject_native_type_named!(PyObjectRef);
pyobject_native_type_convert!(PyObjectRef, PyBaseObject_Type, PyObject_Check);
pyobject_downcast!(PyObjectRef, PyObject_Check);

mod typeobject;
mod module;
mod dict;
mod iterator;
mod boolobject;
mod bytearray;
mod tuple;
mod list;
mod floatob;
mod sequence;
mod slice;
mod stringdata;
mod stringutils;
mod set;
pub mod exc;

#[macro_use]
mod num_common;

#[cfg(Py_3)]
mod num3;

#[cfg(not(Py_3))]
mod num2;

#[cfg(Py_3)]
mod string;

#[cfg(not(Py_3))]
mod string2;

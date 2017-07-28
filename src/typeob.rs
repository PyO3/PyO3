// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python type object information

use std;
use std::mem;
use std::ffi::{CStr, CString};
use std::collections::HashMap;

use {ffi, class, pythonrun};
use err::{PyErr, PyResult};
use instance::Py;
use python::Python;
use objects::PyType;
use class::methods::PyMethodDefType;


/// Python type information.
pub trait PyTypeInfo {
    /// Type of objects to store in PyObject struct
    type Type;

    /// Class name
    const NAME: &'static str;

    /// Class doc string
    const DESCRIPTION: &'static str = "\0";

    /// Size of the rust PyObject structure (PyObject + rust structure)
    const SIZE: usize;

    /// `Type` instance offset inside PyObject structure
    const OFFSET: isize;

    /// Type flags (ie PY_TYPE_FLAG_GC, PY_TYPE_FLAG_WEAKREF)
    const FLAGS: usize = 0;

    /// Base class
    type BaseType: PyTypeInfo;

    /// PyTypeObject instance for this type
    unsafe fn type_object() -> &'static mut ffi::PyTypeObject;

    /// Check `*mut ffi::PyObject` if it is the same type
    fn is_instance(ptr: *mut ffi::PyObject) -> bool;

}

/// type object supports python GC
pub const PY_TYPE_FLAG_GC: usize = 1<<0;

/// Type object supports python weak references
pub const PY_TYPE_FLAG_WEAKREF: usize = 1<<1;

/// Type object can be used as the base type of another type
pub const PY_TYPE_FLAG_BASETYPE: usize = 1<<2;

/// The instances of this type have a dictionary containing instance variables
pub const PY_TYPE_FLAG_DICT: usize = 1<<3;


impl<'a, T: ?Sized> PyTypeInfo for &'a T where T: PyTypeInfo {
    type Type = T::Type;
    type BaseType = T::BaseType;
    const NAME: &'static str = T::NAME;
    const DESCRIPTION: &'static str = T::DESCRIPTION;
    const SIZE: usize = T::SIZE;
    const OFFSET: isize = T::OFFSET;
    const FLAGS: usize = T::FLAGS;

    #[inline]
    default unsafe fn type_object() -> &'static mut ffi::PyTypeObject {
        <T as PyTypeInfo>::type_object()
    }

    #[inline]
    default fn is_instance(ptr: *mut ffi::PyObject) -> bool {
        <T as PyTypeInfo>::is_instance(ptr)
    }

}

/// A Python object allocator that is usable as a base type for #[class]
pub trait PyObjectAlloc<T> {

    /// Allocates a new object (usually by calling ty->tp_alloc),
    /// and initializes it using value.
    unsafe fn alloc(py: Python, value: T) -> PyResult<*mut ffi::PyObject>;

    /// Calls the rust destructor for the object and frees the memory
    /// (usually by calling ptr->ob_type->tp_free).
    /// This function is used as tp_dealloc implementation.
    unsafe fn dealloc(py: Python, obj: *mut ffi::PyObject);
}

impl<T> PyObjectAlloc<T> for T where T : PyTypeInfo {

    default unsafe fn alloc(_py: Python, value: T) -> PyResult<*mut ffi::PyObject> {
        // TODO: remove this
        T::init_type();

        let obj = ffi::PyType_GenericAlloc(T::type_object(), 0);

        let ptr = (obj as *mut u8).offset(T::OFFSET) as *mut T;
        std::ptr::write(ptr, value);

        Ok(obj)
    }

    #[cfg(Py_3)]
    default unsafe fn dealloc(_py: Python, obj: *mut ffi::PyObject) {
        let ptr = (obj as *mut u8).offset(T::OFFSET) as *mut T;
        std::ptr::drop_in_place(ptr);

        if ffi::PyObject_CallFinalizerFromDealloc(obj) < 0 {
            return
        }

        let ty = ffi::Py_TYPE(obj);
        if ffi::PyType_IS_GC(ty) != 0 {
            ffi::PyObject_GC_Del(obj as *mut ::c_void);
        } else {
            ffi::PyObject_Free(obj as *mut ::c_void);
        }

        // For heap types, PyType_GenericAlloc calls INCREF on the type objects,
        // so we need to call DECREF here:
        if ffi::PyType_HasFeature(ty, ffi::Py_TPFLAGS_HEAPTYPE) != 0 {
            ffi::Py_DECREF(ty as *mut ffi::PyObject);
        }
    }

    #[cfg(not(Py_3))]
    default unsafe fn dealloc(_py: Python, obj: *mut ffi::PyObject) {
        let ptr = (obj as *mut u8).offset(T::OFFSET) as *mut T;
        std::ptr::drop_in_place(ptr);

        let ty = ffi::Py_TYPE(obj);
        if ffi::PyType_IS_GC(ty) != 0 {
            ffi::PyObject_GC_Del(obj as *mut ::c_void);
        } else {
            ffi::PyObject_Free(obj as *mut ::c_void);
        }

        // For heap types, PyType_GenericAlloc calls INCREF on the type objects,
        // so we need to call DECREF here:
        if ffi::PyType_HasFeature(ty, ffi::Py_TPFLAGS_HEAPTYPE) != 0 {
            ffi::Py_DECREF(ty as *mut ffi::PyObject);
        }
    }
}

/// Trait implemented by Python object types that have a corresponding type object.
pub trait PyTypeObject {

    /// Initialize type object
    fn init_type();

    /// Retrieves the type object for this Python object type.
    fn type_object() -> Py<PyType>;

}

impl<T> PyTypeObject for T where T: PyObjectAlloc<T> + PyTypeInfo {

    #[inline]
    default fn init_type() {
        unsafe {
            if ((*<T>::type_object()).tp_flags & ffi::Py_TPFLAGS_READY) == 0 {
                // automatically initialize the class on-demand
                let gil = Python::acquire_gil();
                let py = gil.python();

                initialize_type::<T>(py, None).expect(
                    format!("An error occurred while initializing class {}", T::NAME).as_ref());
            }
        }
    }

    #[inline]
    default fn type_object() -> Py<PyType> {
        <T as PyTypeObject>::init_type();
        PyType::new::<T>()
    }
}


/// Register new type in python object system.
pub fn initialize_type<'p, T>(py: Python<'p>, module_name: Option<&str>) -> PyResult<()>
    where T: PyObjectAlloc<T> + PyTypeInfo
{
    // type name
    let name = match module_name {
        Some(module_name) => CString::new(format!("{}.{}", module_name, T::NAME)),
        None => CString::new(T::NAME)
    };
    let name = name.expect(
        "Module name/type name must not contain NUL byte").into_raw();

    let type_object: &mut ffi::PyTypeObject = unsafe{&mut *T::type_object()};
    let base_type_object: &mut ffi::PyTypeObject = unsafe{
        &mut *<T::BaseType as PyTypeInfo>::type_object()};

    type_object.tp_name = name;
    type_object.tp_doc = T::DESCRIPTION.as_ptr() as *const _;
    type_object.tp_base = base_type_object;

    // dealloc
    type_object.tp_dealloc = Some(tp_dealloc_callback::<T>);

    // type size
    type_object.tp_basicsize = <T as PyTypeInfo>::SIZE as ffi::Py_ssize_t;

    let mut offset = T::SIZE;
    // weakref support (check py3cls::py_class::impl_class)
    if T::FLAGS & PY_TYPE_FLAG_WEAKREF != 0 {
        offset -= std::mem::size_of::<*const ffi::PyObject>();
        type_object.tp_weaklistoffset = offset as isize;
    }

    // __dict__ support
    if T::FLAGS & PY_TYPE_FLAG_DICT != 0 {
        offset -= std::mem::size_of::<*const ffi::PyObject>();
        type_object.tp_dictoffset = offset as isize;
    }

    // GC support
    <T as class::gc::PyGCProtocolImpl>::update_type_object(type_object);

    // descriptor protocol
    <T as class::descr::PyDescrProtocolImpl>::tp_as_descr(type_object);

    // iterator methods
    <T as class::iter::PyIterProtocolImpl>::tp_as_iter(type_object);

    // basic methods
    <T as class::basic::PyObjectProtocolImpl>::tp_as_object(type_object);

    // number methods
    if let Some(meth) = <T as class::number::PyNumberProtocolImpl>::tp_as_number() {
        type_object.tp_as_number = Box::into_raw(Box::new(meth));
    } else {
        type_object.tp_as_number = ::std::ptr::null_mut()
    }

    // mapping methods
    if let Some(meth) = <T as class::mapping::PyMappingProtocolImpl>::tp_as_mapping() {
        type_object.tp_as_mapping = Box::into_raw(Box::new(meth));
    } else {
        type_object.tp_as_mapping = ::std::ptr::null_mut()
    }

    // sequence methods
    if let Some(meth) = <T as class::sequence::PySequenceProtocolImpl>::tp_as_sequence() {
        type_object.tp_as_sequence = Box::into_raw(Box::new(meth));
    } else {
        type_object.tp_as_sequence = ::std::ptr::null_mut()
    }

    // async methods
    async_methods::<T>(type_object);

    // buffer protocol
    if let Some(meth) = <T as class::buffer::PyBufferProtocolImpl>::tp_as_buffer() {
        type_object.tp_as_buffer = Box::into_raw(Box::new(meth));
    } else {
        type_object.tp_as_buffer = ::std::ptr::null_mut()
    }

    // normal methods
    let (new, init, call, mut methods) = py_class_method_defs::<T>()?;
    if !methods.is_empty() {
        methods.push(ffi::PyMethodDef_INIT);
        type_object.tp_methods = methods.as_mut_ptr();
        mem::forget(methods);
    }

    match (new, init) {
        (None, Some(_)) => {
            panic!("{}.__new__ method is required if __init__ method defined", T::NAME);
        }
        _ => ()
    }

    // __new__ method
    type_object.tp_new = new;
    // __init__ method
    type_object.tp_init = init;
    // __call__ method
    type_object.tp_call = call;

    // properties
    let mut props = py_class_properties::<T>();
    if !props.is_empty() {
        props.push(ffi::PyGetSetDef_INIT);
        type_object.tp_getset = props.as_mut_ptr();
        mem::forget(props);
    }

    // set type flags
    py_class_flags::<T>(type_object);
    if type_object.tp_base != unsafe{std::mem::transmute(&ffi::PyBaseObject_Type)} {
        type_object.tp_flags = type_object.tp_flags | ffi::Py_TPFLAGS_HEAPTYPE
    }

    // register type object
    unsafe {
        if ffi::PyType_Ready(type_object) == 0 {
            Ok(())
        } else {
            Err(PyErr::fetch(py))
        }
    }
}

#[cfg(Py_3)]
fn async_methods<T>(type_info: &mut ffi::PyTypeObject) {
    if let Some(meth) = <T as class::async::PyAsyncProtocolImpl>::tp_as_async() {
        type_info.tp_as_async = Box::into_raw(Box::new(meth));
    } else {
        type_info.tp_as_async = ::std::ptr::null_mut()
    }
}

#[cfg(not(Py_3))]
fn async_methods<T>(_type_info: &mut ffi::PyTypeObject) {}

unsafe extern "C" fn tp_dealloc_callback<T>(obj: *mut ffi::PyObject)
    where T: PyObjectAlloc<T>
{
    debug!("DEALLOC: {:?} - {:?}", obj,
           CStr::from_ptr((*(*obj).ob_type).tp_name).to_string_lossy());
    let _pool = pythonrun::GILPool::new_no_pointers();
    let py = Python::assume_gil_acquired();
    <T as PyObjectAlloc<T>>::dealloc(py, obj)
}

#[cfg(Py_3)]
fn py_class_flags<T: PyTypeInfo>(type_object: &mut ffi::PyTypeObject) {
    if type_object.tp_traverse != None || type_object.tp_clear != None ||
        T::FLAGS & PY_TYPE_FLAG_GC != 0
    {
        type_object.tp_flags = ffi::Py_TPFLAGS_DEFAULT |  ffi::Py_TPFLAGS_HAVE_GC;
    } else {
        type_object.tp_flags = ffi::Py_TPFLAGS_DEFAULT;
    }
    if T::FLAGS & PY_TYPE_FLAG_BASETYPE !=  0 {
        type_object.tp_flags |= ffi::Py_TPFLAGS_BASETYPE;
    }
}

#[cfg(not(Py_3))]
fn py_class_flags<T: PyTypeInfo>(type_object: &mut ffi::PyTypeObject) {
    if type_object.tp_traverse != None || type_object.tp_clear != None ||
        T::FLAGS & PY_TYPE_FLAG_GC != 0
    {
        type_object.tp_flags = ffi::Py_TPFLAGS_DEFAULT | ffi::Py_TPFLAGS_CHECKTYPES | ffi::Py_TPFLAGS_HAVE_GC;
    } else {
        type_object.tp_flags = ffi::Py_TPFLAGS_DEFAULT | ffi::Py_TPFLAGS_CHECKTYPES;
    }
    if !type_object.tp_as_buffer.is_null() {
        type_object.tp_flags = type_object.tp_flags | ffi::Py_TPFLAGS_HAVE_NEWBUFFER;
    }
    if T::FLAGS & PY_TYPE_FLAG_BASETYPE !=  0 {
        type_object.tp_flags |= ffi::Py_TPFLAGS_BASETYPE;
    }
}

fn py_class_method_defs<T>() -> PyResult<(Option<ffi::newfunc>,
                                          Option<ffi::initproc>,
                                          Option<ffi::PyCFunctionWithKeywords>,
                                          Vec<ffi::PyMethodDef>)>
{
    let mut defs = Vec::new();
    let mut call = None;
    let mut new = None;
    let mut init = None;

    for def in <T as class::methods::PyMethodsProtocolImpl>::py_methods() {
        match *def {
            PyMethodDefType::New(ref def) => {
                if let class::methods::PyMethodType::PyNewFunc(meth) = def.ml_meth {
                    new = Some(meth)
                }
            },
            PyMethodDefType::Call(ref def) => {
                if let class::methods::PyMethodType::PyCFunctionWithKeywords(meth) = def.ml_meth {
                    call = Some(meth)
                } else {
                    panic!("Method type is not supoorted by tp_call slot")
                }
            }
            PyMethodDefType::Init(ref def) => {
                if let class::methods::PyMethodType::PyInitFunc(meth) = def.ml_meth {
                    init = Some(meth)
                } else {
                    panic!("Method type is not supoorted by tp_init slot")
                }
            }
            PyMethodDefType::Method(ref def) => {
                defs.push(def.as_method_def());
            }
            PyMethodDefType::Class(ref def) => {
                defs.push(def.as_method_def());
            }
            PyMethodDefType::Static(ref def) => {
                defs.push(def.as_method_def());
            }
            _ => (),
        }
    }

    for def in <T as class::basic::PyObjectProtocolImpl>::methods() {
        defs.push(def.as_method_def());
    }
    for def in <T as class::context::PyContextProtocolImpl>::methods() {
        defs.push(def.as_method_def());
    }
    for def in <T as class::mapping::PyMappingProtocolImpl>::methods() {
        defs.push(def.as_method_def());
    }
    for def in <T as class::number::PyNumberProtocolImpl>::methods() {
        defs.push(def.as_method_def());
    }
    for def in <T as class::descr::PyDescrProtocolImpl>::methods() {
        defs.push(def.as_method_def());
    }

    py_class_async_methods::<T>(&mut defs);

    Ok((new, init, call, defs))
}

#[cfg(Py_3)]
fn py_class_async_methods<T>(defs: &mut Vec<ffi::PyMethodDef>) {
    for def in <T as class::async::PyAsyncProtocolImpl>::methods() {
        defs.push(def.as_method_def());
    }
}

#[cfg(not(Py_3))]
fn py_class_async_methods<T>(_defs: &mut Vec<ffi::PyMethodDef>) {}

fn py_class_properties<T>() -> Vec<ffi::PyGetSetDef> {
    let mut defs = HashMap::new();

    for def in <T as class::methods::PyMethodsProtocolImpl>::py_methods() {
        match *def {
            PyMethodDefType::Getter(ref getter) => {
                let name = getter.name.to_string();
                if !defs.contains_key(&name) {
                    let _ = defs.insert(name.clone(), ffi::PyGetSetDef_INIT);
                }
                let def = defs.get_mut(&name).expect("Failed to call get_mut");
                getter.copy_to(def);
            },
            PyMethodDefType::Setter(ref setter) => {
                let name = setter.name.to_string();
                if !defs.contains_key(&name) {
                    let _ = defs.insert(name.clone(), ffi::PyGetSetDef_INIT);
                }
                let def = defs.get_mut(&name).expect("Failed to call get_mut");
                setter.copy_to(def);
            },
            _ => (),
        }
    }

    defs.values().cloned().collect()
}

#![allow(missing_docs)]
//! Crate-private implementation of PyClassObject

use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::mem::{ManuallyDrop, MaybeUninit};
use std::ptr::addr_of_mut;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::impl_::pyclass::{
    PyClassBaseType, PyClassDict, PyClassImpl, PyClassThreadChecker, PyClassWeakRef, PyObjectOffset,
};
use crate::internal::get_slot::TP_FREE;
use crate::type_object::{PyLayout, PySizedLayout};
use crate::types::PyType;
use crate::{ffi, PyClass, PyTypeInfo, Python};

#[cfg(not(Py_LIMITED_API))]
use crate::types::PyTypeMethods;

use super::{PyBorrowError, PyBorrowMutError};

pub trait PyClassMutability {
    // The storage for this inheritance layer. Only the first mutable class in
    // an inheritance hierarchy needs to store the borrow flag.
    type Storage: PyClassBorrowChecker;
    // The borrow flag needed to implement this class' mutability. Empty until
    // the first mutable class, at which point it is BorrowChecker and will be
    // for all subclasses.
    type Checker: PyClassBorrowChecker;
    type ImmutableChild: PyClassMutability;
    type MutableChild: PyClassMutability;
}

pub struct ImmutableClass(());
pub struct MutableClass(());
pub struct ExtendsMutableAncestor<M: PyClassMutability>(PhantomData<M>);

impl PyClassMutability for ImmutableClass {
    type Storage = EmptySlot;
    type Checker = EmptySlot;
    type ImmutableChild = ImmutableClass;
    type MutableChild = MutableClass;
}

impl PyClassMutability for MutableClass {
    type Storage = BorrowChecker;
    type Checker = BorrowChecker;
    type ImmutableChild = ExtendsMutableAncestor<ImmutableClass>;
    type MutableChild = ExtendsMutableAncestor<MutableClass>;
}

impl<M: PyClassMutability> PyClassMutability for ExtendsMutableAncestor<M> {
    type Storage = EmptySlot;
    type Checker = BorrowChecker;
    type ImmutableChild = ExtendsMutableAncestor<ImmutableClass>;
    type MutableChild = ExtendsMutableAncestor<MutableClass>;
}

#[derive(Debug)]
struct BorrowFlag(AtomicUsize);

impl BorrowFlag {
    pub(crate) const UNUSED: usize = 0;
    const HAS_MUTABLE_BORROW: usize = usize::MAX;
    fn increment(&self) -> Result<(), PyBorrowError> {
        let mut value = self.0.load(Ordering::Relaxed);
        loop {
            if value == BorrowFlag::HAS_MUTABLE_BORROW {
                return Err(PyBorrowError { _private: () });
            }
            match self.0.compare_exchange(
                // only increment if the value hasn't changed since the
                // last atomic load
                value,
                value + 1,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(..) => {
                    // value has been successfully incremented, we need an acquire fence
                    // so that data this borrow flag protects can be read safely in this thread
                    std::sync::atomic::fence(Ordering::Acquire);
                    break Ok(());
                }
                Err(changed_value) => {
                    // value changed under us, need to try again
                    value = changed_value;
                }
            }
        }
    }
    fn decrement(&self) {
        // impossible to get into a bad state from here so relaxed
        // ordering is fine, the decrement only needs to eventually
        // be visible
        self.0.fetch_sub(1, Ordering::Relaxed);
    }
}

pub struct EmptySlot(());
pub struct BorrowChecker(BorrowFlag);

pub trait PyClassBorrowChecker {
    /// Initial value for self
    fn new() -> Self;

    /// Increments immutable borrow count, if possible
    fn try_borrow(&self) -> Result<(), PyBorrowError>;

    /// Decrements immutable borrow count
    fn release_borrow(&self);
    /// Increments mutable borrow count, if possible
    fn try_borrow_mut(&self) -> Result<(), PyBorrowMutError>;
    /// Decremements mutable borrow count
    fn release_borrow_mut(&self);
}

impl PyClassBorrowChecker for EmptySlot {
    #[inline]
    fn new() -> Self {
        EmptySlot(())
    }

    #[inline]
    fn try_borrow(&self) -> Result<(), PyBorrowError> {
        Ok(())
    }

    #[inline]
    fn release_borrow(&self) {}

    #[inline]
    fn try_borrow_mut(&self) -> Result<(), PyBorrowMutError> {
        unreachable!()
    }

    #[inline]
    fn release_borrow_mut(&self) {
        unreachable!()
    }
}

impl PyClassBorrowChecker for BorrowChecker {
    #[inline]
    fn new() -> Self {
        Self(BorrowFlag(AtomicUsize::new(BorrowFlag::UNUSED)))
    }

    fn try_borrow(&self) -> Result<(), PyBorrowError> {
        self.0.increment()
    }

    fn release_borrow(&self) {
        self.0.decrement();
    }

    fn try_borrow_mut(&self) -> Result<(), PyBorrowMutError> {
        let flag = &self.0;
        match flag.0.compare_exchange(
            // only allowed to transition to mutable borrow if the reference is
            // currently unused
            BorrowFlag::UNUSED,
            BorrowFlag::HAS_MUTABLE_BORROW,
            // On success, reading the flag and updating its state are an atomic
            // operation
            Ordering::AcqRel,
            // It doesn't matter precisely when the failure gets turned
            // into an error
            Ordering::Relaxed,
        ) {
            Ok(..) => Ok(()),
            Err(..) => Err(PyBorrowMutError { _private: () }),
        }
    }

    fn release_borrow_mut(&self) {
        self.0 .0.store(BorrowFlag::UNUSED, Ordering::Release)
    }
}

pub trait GetBorrowChecker<T: PyClassImpl> {
    fn borrow_checker(
        class_object: &T::Layout,
    ) -> &<T::PyClassMutability as PyClassMutability>::Checker;
}

impl<T: PyClassImpl<PyClassMutability = Self>> GetBorrowChecker<T> for MutableClass {
    fn borrow_checker(class_object: &T::Layout) -> &BorrowChecker {
        &class_object.contents().borrow_checker
    }
}

impl<T: PyClassImpl<PyClassMutability = Self>> GetBorrowChecker<T> for ImmutableClass {
    fn borrow_checker(class_object: &T::Layout) -> &EmptySlot {
        &class_object.contents().borrow_checker
    }
}

impl<T: PyClassImpl<PyClassMutability = Self>, M: PyClassMutability> GetBorrowChecker<T>
    for ExtendsMutableAncestor<M>
where
    T::BaseType: PyClassImpl + PyClassBaseType<LayoutAsBase = <T::BaseType as PyClassImpl>::Layout>,
    <T::BaseType as PyClassImpl>::PyClassMutability: PyClassMutability<Checker = BorrowChecker>,
{
    fn borrow_checker(class_object: &T::Layout) -> &BorrowChecker {
        <<T::BaseType as PyClassImpl>::PyClassMutability as GetBorrowChecker<T::BaseType>>::borrow_checker(class_object.ob_base())
    }
}

/// Base layout of PyClassObject.
#[doc(hidden)]
#[repr(C)]
pub struct PyClassObjectBase<T> {
    ob_base: T,
}

unsafe impl<T, U> PyLayout<T> for PyClassObjectBase<U> where U: PySizedLayout<T> {}

/// Base layout of PyClassObject.
#[doc(hidden)]
#[repr(C)]
pub struct PyVariableClassObjectBase {
    ob_base: ffi::PyVarObject,
}

unsafe impl<T> PyLayout<T> for PyVariableClassObjectBase {}

impl<T: PyTypeInfo> PyClassObjectLayout<T> for PyVariableClassObjectBase {
    fn ensure_threadsafe(&self) {}
    fn check_threadsafe(&self) -> Result<(), PyBorrowError> {
        Ok(())
    }
    unsafe fn tp_dealloc(py: Python<'_>, slf: *mut ffi::PyObject) {
        tp_dealloc(py, slf, T::type_object_raw(py));
    }
}

#[doc(hidden)]
pub trait PyClassObjectLayout<T>: PyLayout<T> {
    fn ensure_threadsafe(&self);
    fn check_threadsafe(&self) -> Result<(), PyBorrowError>;
    /// Implementation of tp_dealloc.
    /// # Safety
    /// - slf must be a valid pointer to an instance of a T or a subclass.
    /// - slf must not be used after this call (as it will be freed).
    unsafe fn tp_dealloc(py: Python<'_>, slf: *mut ffi::PyObject);
}

#[doc(hidden)]
pub trait InternalPyClassObjectLayout<T: PyClassImpl>: PyClassObjectLayout<T> {
    /// Obtain a pointer to the contents of an uninitialized PyObject of this type
    /// Safety: the provided object must have the layout that the implementation is expecting
    unsafe fn contents_uninitialised(
        obj: *mut ffi::PyObject,
    ) -> *mut MaybeUninit<PyClassObjectContents<T>>;

    fn get_ptr(&self) -> *mut T;

    fn contents(&self) -> &PyClassObjectContents<T>;

    fn contents_mut(&mut self) -> &mut PyClassObjectContents<T>;

    fn ob_base(&self) -> &<T::BaseType as PyClassBaseType>::LayoutAsBase;

    /// Used to set PyType_Spec::basicsize
    /// https://docs.python.org/3/c-api/type.html#c.PyType_Spec.basicsize
    fn basicsize() -> ffi::Py_ssize_t;

    /// Gets the offset of the contents from the start of the struct in bytes.
    fn contents_offset() -> PyObjectOffset;

    /// Gets the offset of the dictionary from the start of the struct in bytes.
    fn dict_offset() -> PyObjectOffset;

    /// Gets the offset of the weakref list from the start of the struct in bytes.
    fn weaklist_offset() -> PyObjectOffset;

    fn borrow_checker(&self) -> &<T::PyClassMutability as PyClassMutability>::Checker;
}

impl<T, U> PyClassObjectLayout<T> for PyClassObjectBase<U>
where
    U: PySizedLayout<T>,
    T: PyTypeInfo,
{
    fn ensure_threadsafe(&self) {}
    fn check_threadsafe(&self) -> Result<(), PyBorrowError> {
        Ok(())
    }
    unsafe fn tp_dealloc(py: Python<'_>, slf: *mut ffi::PyObject) {
        tp_dealloc(py, slf, T::type_object_raw(py));
    }
}

unsafe fn tp_dealloc(py: Python<'_>, obj: *mut ffi::PyObject, type_ptr: *mut ffi::PyTypeObject) {
    // FIXME: there is potentially subtle issues here if the base is overwritten
    // at runtime? To be investigated.
    let actual_type = PyType::from_borrowed_type_ptr(py, ffi::Py_TYPE(obj));

    // For `#[pyclass]` types which inherit from PyAny or PyType, we can just call tp_free
    let is_base_object = type_ptr == std::ptr::addr_of_mut!(ffi::PyBaseObject_Type);
    let is_metaclass = type_ptr == std::ptr::addr_of_mut!(ffi::PyType_Type);
    if is_base_object || is_metaclass {
        let tp_free = actual_type
            .get_slot(TP_FREE)
            .expect("base type should have tp_free");
        return tp_free(obj.cast());
    }

    // More complex native types (e.g. `extends=PyDict`) require calling the base's dealloc.
    #[cfg(not(Py_LIMITED_API))]
    {
        // FIXME: should this be using actual_type.tp_dealloc?
        if let Some(dealloc) = (*type_ptr).tp_dealloc {
            // Before CPython 3.11 BaseException_dealloc would use Py_GC_UNTRACK which
            // assumes the exception is currently GC tracked, so we have to re-track
            // before calling the dealloc so that it can safely call Py_GC_UNTRACK.
            #[cfg(not(any(Py_3_11, PyPy)))]
            if ffi::PyType_FastSubclass(type_ptr, ffi::Py_TPFLAGS_BASE_EXC_SUBCLASS) == 1 {
                ffi::PyObject_GC_Track(obj.cast());
            }
            dealloc(obj);
        } else {
            (*actual_type.as_type_ptr())
                .tp_free
                .expect("type missing tp_free")(obj.cast());
        }
    }

    #[cfg(Py_LIMITED_API)]
    unreachable!("subclassing native types is not possible with the `abi3` feature");
}

#[repr(C)]
pub(crate) struct PyClassObjectContents<T: PyClassImpl> {
    pub(crate) value: ManuallyDrop<UnsafeCell<T>>,
    pub(crate) borrow_checker: <T::PyClassMutability as PyClassMutability>::Storage,
    pub(crate) thread_checker: T::ThreadChecker,
    pub(crate) dict: T::Dict,
    pub(crate) weakref: T::WeakRef,
}

impl<T: PyClassImpl> PyClassObjectContents<T> {
    pub(crate) fn new(init: T) -> Self {
        PyClassObjectContents {
            value: ManuallyDrop::new(UnsafeCell::new(init)),
            borrow_checker: <T::PyClassMutability as PyClassMutability>::Storage::new(),
            thread_checker: T::ThreadChecker::new(),
            dict: T::Dict::INIT,
            weakref: T::WeakRef::INIT,
        }
    }

    unsafe fn dealloc(&mut self, py: Python<'_>, py_object: *mut ffi::PyObject) {
        if self.thread_checker.can_drop(py) {
            ManuallyDrop::drop(&mut self.value);
        }
        self.dict.clear_dict(py);
        self.weakref.clear_weakrefs(py_object, py);
    }
}

/// The layout of a PyClass with a known sized base class as a Python object
#[repr(C)]
pub struct PyStaticClassObject<T: PyClassImpl> {
    ob_base: <T::BaseType as PyClassBaseType>::LayoutAsBase,
    contents: PyClassObjectContents<T>,
}

impl<T: PyClassImpl> InternalPyClassObjectLayout<T> for PyStaticClassObject<T> {
    unsafe fn contents_uninitialised(
        obj: *mut ffi::PyObject,
    ) -> *mut MaybeUninit<PyClassObjectContents<T>> {
        #[repr(C)]
        struct PartiallyInitializedClassObject<T: PyClassImpl> {
            _ob_base: <T::BaseType as PyClassBaseType>::LayoutAsBase,
            contents: MaybeUninit<PyClassObjectContents<T>>,
        }
        let obj: *mut PartiallyInitializedClassObject<T> = obj.cast();
        addr_of_mut!((*obj).contents)
    }

    fn get_ptr(&self) -> *mut T {
        self.contents.value.get()
    }

    fn ob_base(&self) -> &<T::BaseType as PyClassBaseType>::LayoutAsBase {
        &self.ob_base
    }

    fn contents(&self) -> &PyClassObjectContents<T> {
        &self.contents
    }

    fn contents_mut(&mut self) -> &mut PyClassObjectContents<T> {
        &mut self.contents
    }

    /// used to set PyType_Spec::basicsize
    /// https://docs.python.org/3/c-api/type.html#c.PyType_Spec.basicsize
    fn basicsize() -> ffi::Py_ssize_t {
        let size = std::mem::size_of::<Self>();

        // Py_ssize_t may not be equal to isize on all platforms
        #[allow(clippy::useless_conversion)]
        size.try_into().expect("size should fit in Py_ssize_t")
    }

    /// Gets the offset of the contents from the start of the struct in bytes.
    fn contents_offset() -> PyObjectOffset {
        PyObjectOffset::Absolute(usize_to_py_ssize(memoffset::offset_of!(
            PyStaticClassObject<T>,
            contents
        )))
    }

    /// Gets the offset of the dictionary from the start of the struct in bytes.
    fn dict_offset() -> PyObjectOffset {
        use memoffset::offset_of;

        let offset = offset_of!(PyStaticClassObject<T>, contents)
            + offset_of!(PyClassObjectContents<T>, dict);

        PyObjectOffset::Absolute(usize_to_py_ssize(offset))
    }

    /// Gets the offset of the weakref list from the start of the struct in bytes.
    fn weaklist_offset() -> PyObjectOffset {
        use memoffset::offset_of;

        let offset = offset_of!(PyStaticClassObject<T>, contents)
            + offset_of!(PyClassObjectContents<T>, weakref);

        PyObjectOffset::Absolute(usize_to_py_ssize(offset))
    }

    fn borrow_checker(&self) -> &<T::PyClassMutability as PyClassMutability>::Checker {
        // Safety: T::Layout must be PyStaticClassObject<T>
        let slf: &T::Layout = unsafe { std::mem::transmute(self) };
        T::PyClassMutability::borrow_checker(slf)
    }
}

unsafe impl<T: PyClassImpl> PyLayout<T> for PyStaticClassObject<T> {}
impl<T: PyClass> PySizedLayout<T> for PyStaticClassObject<T> {}

impl<T: PyClassImpl> PyClassObjectLayout<T> for PyStaticClassObject<T>
where
    <T::BaseType as PyClassBaseType>::LayoutAsBase: PyClassObjectLayout<T::BaseType>,
{
    fn ensure_threadsafe(&self) {
        self.contents.thread_checker.ensure();
        self.ob_base.ensure_threadsafe();
    }
    fn check_threadsafe(&self) -> Result<(), PyBorrowError> {
        if !self.contents.thread_checker.check() {
            return Err(PyBorrowError { _private: () });
        }
        self.ob_base.check_threadsafe()
    }
    unsafe fn tp_dealloc(py: Python<'_>, slf: *mut ffi::PyObject) {
        // Safety: Python only calls tp_dealloc when no references to the object remain.
        let class_object = &mut *(slf.cast::<T::Layout>());
        class_object.contents_mut().dealloc(py, slf);
        <T::BaseType as PyClassBaseType>::LayoutAsBase::tp_dealloc(py, slf)
    }
}

#[repr(C)]
pub struct PyVariableClassObject<T: PyClassImpl> {
    ob_base: <T::BaseType as PyClassBaseType>::LayoutAsBase,
}

impl<T: PyClassImpl> PyVariableClassObject<T> {
    #[cfg(Py_3_12)]
    fn get_contents_of_obj(obj: *mut ffi::PyObject) -> *mut PyClassObjectContents<T> {
        // https://peps.python.org/pep-0697/
        let type_obj = unsafe { ffi::Py_TYPE(obj) };
        let pointer = unsafe { ffi::PyObject_GetTypeData(obj, type_obj) };
        pointer as *mut PyClassObjectContents<T>
    }

    #[cfg(Py_3_12)]
    fn get_contents_ptr(&self) -> *mut PyClassObjectContents<T> {
        Self::get_contents_of_obj(self as *const PyVariableClassObject<T> as *mut ffi::PyObject)
    }
}

#[cfg(Py_3_12)]
impl<T: PyClassImpl> InternalPyClassObjectLayout<T> for PyVariableClassObject<T> {
    unsafe fn contents_uninitialised(
        obj: *mut ffi::PyObject,
    ) -> *mut MaybeUninit<PyClassObjectContents<T>> {
        Self::get_contents_of_obj(obj) as *mut MaybeUninit<PyClassObjectContents<T>>
    }

    fn get_ptr(&self) -> *mut T {
        self.contents().value.get()
    }

    fn ob_base(&self) -> &<T::BaseType as PyClassBaseType>::LayoutAsBase {
        &self.ob_base
    }

    fn contents(&self) -> &PyClassObjectContents<T> {
        unsafe { (self.get_contents_ptr() as *const PyClassObjectContents<T>).as_ref() }
            .expect("should be able to cast PyClassObjectContents pointer")
    }

    fn contents_mut(&mut self) -> &mut PyClassObjectContents<T> {
        unsafe { self.get_contents_ptr().as_mut() }
            .expect("should be able to cast PyClassObjectContents pointer")
    }

    /// used to set PyType_Spec::basicsize
    /// https://docs.python.org/3/c-api/type.html#c.PyType_Spec.basicsize
    fn basicsize() -> ffi::Py_ssize_t {
        let size = std::mem::size_of::<PyClassObjectContents<T>>();
        // negative to indicate 'extra' space that cpython will allocate for us
        -usize_to_py_ssize(size)
    }

    /// Gets the offset of the contents from the start of the struct in bytes.
    fn contents_offset() -> PyObjectOffset {
        PyObjectOffset::Relative(0)
    }

    /// Gets the offset of the dictionary from the start of the struct in bytes.
    fn dict_offset() -> PyObjectOffset {
        PyObjectOffset::Relative(usize_to_py_ssize(memoffset::offset_of!(
            PyClassObjectContents<T>,
            dict
        )))
    }

    /// Gets the offset of the weakref list from the start of the struct in bytes.
    fn weaklist_offset() -> PyObjectOffset {
        PyObjectOffset::Relative(usize_to_py_ssize(memoffset::offset_of!(
            PyClassObjectContents<T>,
            weakref
        )))
    }

    fn borrow_checker(&self) -> &<T::PyClassMutability as PyClassMutability>::Checker {
        // Safety: T::Layout must be PyStaticClassObject<T>
        let slf: &T::Layout = unsafe { std::mem::transmute(self) };
        T::PyClassMutability::borrow_checker(slf)
    }
}

unsafe impl<T: PyClassImpl> PyLayout<T> for PyVariableClassObject<T> {}

impl<T: PyClassImpl> PyClassObjectLayout<T> for PyVariableClassObject<T>
where
    <T::BaseType as PyClassBaseType>::LayoutAsBase: PyClassObjectLayout<T::BaseType>,
{
    fn ensure_threadsafe(&self) {
        self.contents().thread_checker.ensure();
        self.ob_base.ensure_threadsafe();
    }
    fn check_threadsafe(&self) -> Result<(), PyBorrowError> {
        if !self.contents().thread_checker.check() {
            return Err(PyBorrowError { _private: () });
        }
        self.ob_base.check_threadsafe()
    }
    unsafe fn tp_dealloc(py: Python<'_>, slf: *mut ffi::PyObject) {
        // Safety: Python only calls tp_dealloc when no references to the object remain.
        let class_object = &mut *(slf.cast::<T::Layout>());
        class_object.contents_mut().dealloc(py, slf);
        <T::BaseType as PyClassBaseType>::LayoutAsBase::tp_dealloc(py, slf)
    }
}

/// Py_ssize_t may not be equal to isize on all platforms
fn usize_to_py_ssize(value: usize) -> ffi::Py_ssize_t {
    #[allow(clippy::useless_conversion)]
    value.try_into().expect("value should fit in Py_ssize_t")
}

#[cfg(test)]
#[cfg(feature = "macros")]
mod tests {
    use super::*;

    use crate::prelude::*;
    use crate::pyclass::boolean_struct::{False, True};

    #[pyclass(crate = "crate", subclass)]
    struct MutableBase;

    #[pyclass(crate = "crate", extends = MutableBase, subclass)]
    struct MutableChildOfMutableBase;

    #[pyclass(crate = "crate", extends = MutableBase, frozen, subclass)]
    struct ImmutableChildOfMutableBase;

    #[pyclass(crate = "crate", extends = MutableChildOfMutableBase)]
    struct MutableChildOfMutableChildOfMutableBase;

    #[pyclass(crate = "crate", extends = ImmutableChildOfMutableBase)]
    struct MutableChildOfImmutableChildOfMutableBase;

    #[pyclass(crate = "crate", extends = MutableChildOfMutableBase, frozen)]
    struct ImmutableChildOfMutableChildOfMutableBase;

    #[pyclass(crate = "crate", extends = ImmutableChildOfMutableBase, frozen)]
    struct ImmutableChildOfImmutableChildOfMutableBase;

    #[pyclass(crate = "crate", frozen, subclass)]
    struct ImmutableBase;

    #[pyclass(crate = "crate", extends = ImmutableBase, subclass)]
    struct MutableChildOfImmutableBase;

    #[pyclass(crate = "crate", extends = ImmutableBase, frozen, subclass)]
    struct ImmutableChildOfImmutableBase;

    #[pyclass(crate = "crate", extends = MutableChildOfImmutableBase)]
    struct MutableChildOfMutableChildOfImmutableBase;

    #[pyclass(crate = "crate", extends = ImmutableChildOfImmutableBase)]
    struct MutableChildOfImmutableChildOfImmutableBase;

    #[pyclass(crate = "crate", extends = MutableChildOfImmutableBase, frozen)]
    struct ImmutableChildOfMutableChildOfImmutableBase;

    #[pyclass(crate = "crate", extends = ImmutableChildOfImmutableBase, frozen)]
    struct ImmutableChildOfImmutableChildOfImmutableBase;

    #[pyclass(crate = "crate", subclass)]
    struct BaseWithData(#[allow(unused)] u64);

    #[pyclass(crate = "crate", extends = BaseWithData)]
    struct ChildWithData(#[allow(unused)] u64);

    #[pyclass(crate = "crate", extends = BaseWithData)]
    struct ChildWithoutData;

    #[test]
    fn test_inherited_size() {
        let base_size = PyStaticClassObject::<BaseWithData>::basicsize();
        assert!(base_size > 0); // negative indicates variable sized
        assert_eq!(
            base_size,
            PyStaticClassObject::<ChildWithoutData>::basicsize()
        );
        assert!(base_size < PyStaticClassObject::<ChildWithData>::basicsize());
    }

    fn assert_mutable<T: PyClass<Frozen = False, PyClassMutability = MutableClass>>() {}
    fn assert_immutable<T: PyClass<Frozen = True, PyClassMutability = ImmutableClass>>() {}
    fn assert_mutable_with_mutable_ancestor<
        T: PyClass<Frozen = False, PyClassMutability = ExtendsMutableAncestor<MutableClass>>,
    >() {
    }
    fn assert_immutable_with_mutable_ancestor<
        T: PyClass<Frozen = True, PyClassMutability = ExtendsMutableAncestor<ImmutableClass>>,
    >() {
    }

    #[test]
    fn test_inherited_mutability() {
        // mutable base
        assert_mutable::<MutableBase>();

        // children of mutable base have a mutable ancestor
        assert_mutable_with_mutable_ancestor::<MutableChildOfMutableBase>();
        assert_immutable_with_mutable_ancestor::<ImmutableChildOfMutableBase>();

        // grandchildren of mutable base have a mutable ancestor
        assert_mutable_with_mutable_ancestor::<MutableChildOfMutableChildOfMutableBase>();
        assert_mutable_with_mutable_ancestor::<MutableChildOfImmutableChildOfMutableBase>();
        assert_immutable_with_mutable_ancestor::<ImmutableChildOfMutableChildOfMutableBase>();
        assert_immutable_with_mutable_ancestor::<ImmutableChildOfImmutableChildOfMutableBase>();

        // immutable base and children
        assert_immutable::<ImmutableBase>();
        assert_immutable::<ImmutableChildOfImmutableBase>();
        assert_immutable::<ImmutableChildOfImmutableChildOfImmutableBase>();

        // mutable children of immutable at any level are simply mutable
        assert_mutable::<MutableChildOfImmutableBase>();
        assert_mutable::<MutableChildOfImmutableChildOfImmutableBase>();

        // children of the mutable child display this property
        assert_mutable_with_mutable_ancestor::<MutableChildOfMutableChildOfImmutableBase>();
        assert_immutable_with_mutable_ancestor::<ImmutableChildOfMutableChildOfImmutableBase>();
    }

    #[test]
    fn test_mutable_borrow_prevents_further_borrows() {
        Python::with_gil(|py| {
            let mmm = Py::new(
                py,
                PyClassInitializer::from(MutableBase)
                    .add_subclass(MutableChildOfMutableBase)
                    .add_subclass(MutableChildOfMutableChildOfMutableBase),
            )
            .unwrap();

            let mmm_bound: &Bound<'_, MutableChildOfMutableChildOfMutableBase> = mmm.bind(py);

            let mmm_refmut = mmm_bound.borrow_mut();

            // Cannot take any other mutable or immutable borrows whilst the object is borrowed mutably
            assert!(mmm_bound
                .extract::<PyRef<'_, MutableChildOfMutableChildOfMutableBase>>()
                .is_err());
            assert!(mmm_bound
                .extract::<PyRef<'_, MutableChildOfMutableBase>>()
                .is_err());
            assert!(mmm_bound.extract::<PyRef<'_, MutableBase>>().is_err());
            assert!(mmm_bound
                .extract::<PyRefMut<'_, MutableChildOfMutableChildOfMutableBase>>()
                .is_err());
            assert!(mmm_bound
                .extract::<PyRefMut<'_, MutableChildOfMutableBase>>()
                .is_err());
            assert!(mmm_bound.extract::<PyRefMut<'_, MutableBase>>().is_err());

            // With the borrow dropped, all other borrow attempts will succeed
            drop(mmm_refmut);

            assert!(mmm_bound
                .extract::<PyRef<'_, MutableChildOfMutableChildOfMutableBase>>()
                .is_ok());
            assert!(mmm_bound
                .extract::<PyRef<'_, MutableChildOfMutableBase>>()
                .is_ok());
            assert!(mmm_bound.extract::<PyRef<'_, MutableBase>>().is_ok());
            assert!(mmm_bound
                .extract::<PyRefMut<'_, MutableChildOfMutableChildOfMutableBase>>()
                .is_ok());
            assert!(mmm_bound
                .extract::<PyRefMut<'_, MutableChildOfMutableBase>>()
                .is_ok());
            assert!(mmm_bound.extract::<PyRefMut<'_, MutableBase>>().is_ok());
        })
    }

    #[test]
    fn test_immutable_borrows_prevent_mutable_borrows() {
        Python::with_gil(|py| {
            let mmm = Py::new(
                py,
                PyClassInitializer::from(MutableBase)
                    .add_subclass(MutableChildOfMutableBase)
                    .add_subclass(MutableChildOfMutableChildOfMutableBase),
            )
            .unwrap();

            let mmm_bound: &Bound<'_, MutableChildOfMutableChildOfMutableBase> = mmm.bind(py);

            let mmm_refmut = mmm_bound.borrow();

            // Further immutable borrows are ok
            assert!(mmm_bound
                .extract::<PyRef<'_, MutableChildOfMutableChildOfMutableBase>>()
                .is_ok());
            assert!(mmm_bound
                .extract::<PyRef<'_, MutableChildOfMutableBase>>()
                .is_ok());
            assert!(mmm_bound.extract::<PyRef<'_, MutableBase>>().is_ok());

            // Further mutable borrows are not ok
            assert!(mmm_bound
                .extract::<PyRefMut<'_, MutableChildOfMutableChildOfMutableBase>>()
                .is_err());
            assert!(mmm_bound
                .extract::<PyRefMut<'_, MutableChildOfMutableBase>>()
                .is_err());
            assert!(mmm_bound.extract::<PyRefMut<'_, MutableBase>>().is_err());

            // With the borrow dropped, all mutable borrow attempts will succeed
            drop(mmm_refmut);

            assert!(mmm_bound
                .extract::<PyRefMut<'_, MutableChildOfMutableChildOfMutableBase>>()
                .is_ok());
            assert!(mmm_bound
                .extract::<PyRefMut<'_, MutableChildOfMutableBase>>()
                .is_ok());
            assert!(mmm_bound.extract::<PyRefMut<'_, MutableBase>>().is_ok());
        })
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_thread_safety() {
        #[crate::pyclass(crate = "crate")]
        struct MyClass {
            x: u64,
        }

        Python::with_gil(|py| {
            let inst = Py::new(py, MyClass { x: 0 }).unwrap();

            let total_modifications = py.allow_threads(|| {
                std::thread::scope(|s| {
                    // Spawn a bunch of threads all racing to write to
                    // the same instance of `MyClass`.
                    let threads = (0..10)
                        .map(|_| {
                            s.spawn(|| {
                                Python::with_gil(|py| {
                                    // Each thread records its own view of how many writes it made
                                    let mut local_modifications = 0;
                                    for _ in 0..100 {
                                        if let Ok(mut i) = inst.try_borrow_mut(py) {
                                            i.x += 1;
                                            local_modifications += 1;
                                        }
                                    }
                                    local_modifications
                                })
                            })
                        })
                        .collect::<Vec<_>>();

                    // Sum up the total number of writes made by all threads
                    threads.into_iter().map(|t| t.join().unwrap()).sum::<u64>()
                })
            });

            // If the implementation is free of data races, the total number of writes
            // should match the final value of `x`.
            assert_eq!(total_modifications, inst.borrow(py).x);
        });
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_thread_safety_2() {
        struct SyncUnsafeCell<T>(UnsafeCell<T>);
        unsafe impl<T> Sync for SyncUnsafeCell<T> {}

        impl<T> SyncUnsafeCell<T> {
            fn get(&self) -> *mut T {
                self.0.get()
            }
        }

        let data = SyncUnsafeCell(UnsafeCell::new(0));
        let data2 = SyncUnsafeCell(UnsafeCell::new(0));
        let borrow_checker = BorrowChecker(BorrowFlag(AtomicUsize::new(BorrowFlag::UNUSED)));

        std::thread::scope(|s| {
            s.spawn(|| {
                for _ in 0..1_000_000 {
                    if borrow_checker.try_borrow_mut().is_ok() {
                        // thread 1 writes to both values during the mutable borrow
                        unsafe { *data.get() += 1 };
                        unsafe { *data2.get() += 1 };
                        borrow_checker.release_borrow_mut();
                    }
                }
            });

            s.spawn(|| {
                for _ in 0..1_000_000 {
                    if borrow_checker.try_borrow().is_ok() {
                        // if the borrow checker is working correctly, it should be impossible
                        // for thread 2 to observe a difference in the two values
                        assert_eq!(unsafe { *data.get() }, unsafe { *data2.get() });
                        borrow_checker.release_borrow();
                    }
                }
            });
        });
    }
}

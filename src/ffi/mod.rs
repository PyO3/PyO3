#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
#![cfg_attr(Py_LIMITED_API, allow(unused_imports))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::inline_always))]

// Until `extern type` is stabilized, use the recommended approach to
// model opaque types:
// https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs
macro_rules! opaque_struct {
    ($name:ident) => {
        #[repr(C)]
        pub struct $name([u8; 0]);
    };
}

pub use self::bltinmodule::*;
pub use self::boolobject::*;
pub use self::bytearrayobject::*;
pub use self::bytesobject::*;
pub use self::ceval::*;
pub use self::code::*;
pub use self::codecs::*;
pub use self::compile::*;
pub use self::complexobject::*;
pub use self::context::*;
#[cfg(not(Py_LIMITED_API))]
pub use self::datetime::*;
pub use self::descrobject::*;
pub use self::dictobject::*;
pub use self::enumobject::*;
pub use self::eval::*;
pub use self::fileobject::*;
pub use self::floatobject::*;
pub use self::frameobject::PyFrameObject;
pub use self::funcobject::*;
pub use self::genobject::*;
pub use self::import::*;
#[cfg(all(Py_3_8, not(any(PY_LIMITED_API, PyPy))))]
pub use self::initconfig::*;
pub use self::intrcheck::*;
pub use self::iterobject::*;
pub use self::listobject::*;
pub use self::longobject::*;
pub use self::marshal::*;
pub use self::memoryobject::*;
pub use self::methodobject::*;
pub use self::modsupport::*;
pub use self::moduleobject::*;
pub use self::object::*;
pub use self::objectabstract::*; // FIXME: no matching objectabstract.h in cpython master
pub use self::objimpl::*;
pub use self::osmodule::*;
pub use self::pyarena::*;
pub use self::pycapsule::*;
pub use self::pydebug::*;
pub use self::pyerrors::*;
pub use self::pyhash::*;
pub use self::pylifecycle::*;
pub use self::pymem::*;
pub use self::pyport::*;
pub use self::pystate::*;
pub use self::pystrtod::*;
pub use self::pythonrun::*;
pub use self::rangeobject::*;
pub use self::setobject::*;
pub use self::sliceobject::*;
pub use self::structseq::*;
pub use self::sysmodule::*;
pub use self::traceback::*;
pub use self::tupleobject::*;
pub use self::typeslots::*;
pub use self::unicodeobject::*;
pub use self::warnings::*;
pub use self::weakrefobject::*;

#[cfg(not(Py_LIMITED_API))]
pub use self::cpython::*;

// skipped abstract.h
// skipped asdl.h
// skipped ast.h
mod bltinmodule;
mod boolobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod bytearrayobject;
mod bytesobject;
// skipped cellobject.h
mod ceval; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5

// skipped classobject.h
mod codecs; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod code {}
mod compile; // TODO: incomplete
mod complexobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5

// skipped dynamic_annotations.h
// skipped errcode.h
// skipped exports.h
// skipped fileutils.h
// skipped genericaliasobject.h
// skipped interpreteridobject.h
// skipped longintrepr.h
// skipped namespaceobject.h
// skipped odictobject.h
// skipped opcode.h
// skipped osdefs.h
// skipped parser_interface.h
// skipped patchlevel.h
// skipped picklebufobject.h
// skipped pyctype.h
// skipped py_curses.h
// skipped pydecimal.h
// skipped pydtrace.h
// skipped pyexpat.h
// skipped pyfpe.h
// skipped pyframe.h
// skipped pymacconfig.h
// skipped pymacro.h
// skipped pymath.h
// skipped pystrcmp.h
// skipped pystrhex.h
// skipped Python-ast.h
// this file is Python.h
// skipped pythread.h
// skipped pytime.h

mod pyport;
// mod pymacro; contains nothing of interest for Rust
// mod pyatomic; contains nothing of interest for Rust
// mod pymath; contains nothing of interest for Rust

// [cfg(not(Py_LIMITED_API))]
// mod pytime; contains nothing of interest

#[cfg(all(Py_3_8, not(any(PY_LIMITED_API, PyPy))))]
mod initconfig;
mod object;
mod objimpl;
mod pydebug;
mod pyhash;
mod pymem;
mod typeslots;

mod longobject;
mod unicodeobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
                   // mod longintrepr; TODO excluded by PEP-384
mod dictobject;
mod floatobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod listobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod memoryobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod rangeobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod tupleobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
                 // mod odictobject; TODO new in 3.5
mod enumobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod methodobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod moduleobject;
mod setobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
               // mod funcobject; TODO excluded by PEP-384
               // mod classobject; TODO excluded by PEP-384
mod fileobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod pycapsule; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod sliceobject;
mod traceback; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
               // mod cellobject; TODO excluded by PEP-384
mod descrobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod genobject; // TODO excluded by PEP-384
mod iterobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod structseq;
mod warnings; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod weakrefobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5

mod pyerrors; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod pylifecycle;
mod pystate; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5

#[cfg(Py_LIMITED_API)]
mod pyarena {}
mod modsupport; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
#[cfg(not(Py_LIMITED_API))]
mod pyarena; // TODO: incomplete
mod pythonrun; // TODO some functions need to be moved to pylifecycle
               //mod pylifecycle; // TODO new in 3.5
mod import;
mod intrcheck; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod osmodule;
mod sysmodule; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5

mod objectabstract; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5

#[cfg(all(Py_3_8, not(Py_LIMITED_API)))]
mod context; // It's actually 3.7.1, but no cfg for patches.

#[cfg(not(all(Py_3_8, not(Py_LIMITED_API))))]
mod context {}

mod eval; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5

// mod pyctype; TODO excluded by PEP-384
mod pystrtod; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
              // mod pystrcmp; TODO nothing interesting for Rust?
              // mod dtoa; TODO excluded by PEP-384
              // mod fileutils; TODO no public functions?
              // mod pyfpe; TODO probably not interesting for rust

// Additional headers that are not exported by Python.h
pub mod structmember; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5

#[cfg(not(Py_LIMITED_API))]
pub mod frameobject;
#[cfg(Py_LIMITED_API)]
pub mod frameobject {
    opaque_struct!(PyFrameObject);
}

#[cfg(not(Py_LIMITED_API))]
pub(crate) mod datetime;
pub(crate) mod marshal;

pub(crate) mod funcobject;

#[cfg(not(Py_LIMITED_API))]
mod cpython;

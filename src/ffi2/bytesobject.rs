pub use ffi2::stringobject::PyStringObject as PyBytesObject;
pub use ffi2::stringobject::PyString_Type as PyBytes_Type;
pub use ffi2::stringobject::PyString_Check as PyBytes_Check;
pub use ffi2::stringobject::PyString_CheckExact as PyBytes_CheckExact;
pub use ffi2::stringobject::PyString_AS_STRING as PyBytes_AS_STRING;
pub use ffi2::stringobject::PyString_GET_SIZE as PyBytes_GET_SIZE;
pub use ffi2::object::Py_TPFLAGS_STRING_SUBCLASS as Py_TPFLAGS_BYTES_SUBCLASS;
pub use ffi2::stringobject::PyString_FromStringAndSize as PyBytes_FromStringAndSize;
pub use ffi2::stringobject::PyString_FromString as PyBytes_FromString;
pub use ffi2::stringobject::PyString_FromFormat as PyBytes_FromFormat;
pub use ffi2::stringobject::PyString_Size as PyBytes_Size;
pub use ffi2::stringobject::PyString_AsString as PyBytes_AsString;
pub use ffi2::stringobject::PyString_Concat as PyBytes_Concat;
pub use ffi2::stringobject::PyString_ConcatAndDel as PyBytes_ConcatAndDel;
pub use ffi2::stringobject::PyString_Format as PyBytes_Format;
pub use ffi2::stringobject::PyString_AsStringAndSize as PyBytes_AsStringAndSize;

#![no_implicit_prelude]
#![allow(unused_variables)]

#[crate::pyclass]
#[pyo3(crate = "crate")]
#[derive(::std::clone::Clone)]
pub struct Foo;

#[crate::pyclass]
#[pyo3(crate = "crate")]
pub struct Foo2;

#[crate::pyclass(
    name = "ActuallyBar",
    freelist = 8,
    weakref,
    unsendable,
    subclass,
    extends = crate::types::PyAny,
    module = "Spam"
)]
#[pyo3(crate = "crate")]
pub struct Bar {
    #[pyo3(get, set)]
    a: u8,
    #[pyo3(get, set)]
    b: Foo,
    #[pyo3(set)]
    c: ::std::option::Option<crate::Py<Foo2>>,
}

#[crate::pyclass(eq, eq_int)]
#[pyo3(crate = "crate")]
#[derive(PartialEq)]
pub enum Enum {
    Var0,
}

#[crate::pyclass]
#[pyo3(crate = "crate")]
pub struct Foo3 {
    #[pyo3(get, set)]
    #[cfg(any())]
    field: i32,

    #[pyo3(get, set)]
    #[cfg(not(any()))]
    field: u32,
}

#[crate::pyclass]
#[pyo3(crate = "crate")]
pub struct Foo4 {
    #[pyo3(get, set)]
    #[cfg(any())]
    #[cfg(not(any()))]
    field: i32,

    #[pyo3(get, set)]
    #[cfg(not(any()))]
    field: u32,
}

#[crate::pyclass(eq, str = "{:?}")]
#[pyo3(crate = "crate")]
#[derive(Debug, PartialEq)]
pub enum EnumFmt {
    #[pyo3(name = "NewEnum")]
    Var0,
}

#[crate::pyclass(str)]
#[pyo3(crate = "crate")]
pub struct Point {
    x: i32,
    y: i32,
    z: i32,
}

impl ::std::fmt::Display for Point {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

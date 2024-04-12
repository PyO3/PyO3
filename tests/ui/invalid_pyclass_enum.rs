use pyo3::prelude::*;

#[pyclass(subclass)]
enum NotBaseClass {
    X,
    Y,
}

#[pyclass(extends = PyList)]
enum NotDrivedClass {
    X,
    Y,
}

#[pyclass]
enum NoEmptyEnum {}

#[pyclass]
enum NoUnitVariants {
    StructVariant { field: i32 },
    UnitVariant,
}

fn main() {}

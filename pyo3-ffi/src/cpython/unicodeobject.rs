#[cfg(any(Py_3_11, not(PyPy)))]
use crate::Py_hash_t;
use crate::{PyObject, Py_UCS1, Py_UCS2, Py_UCS4, Py_ssize_t};
use libc::wchar_t;
#[cfg(Py_3_14)]
use std::os::raw::c_ushort;
use std::os::raw::{c_char, c_int, c_uint, c_void};

// skipped Py_UNICODE_ISSPACE()
// skipped Py_UNICODE_ISLOWER()
// skipped Py_UNICODE_ISUPPER()
// skipped Py_UNICODE_ISTITLE()
// skipped Py_UNICODE_ISLINEBREAK
// skipped Py_UNICODE_TOLOWER
// skipped Py_UNICODE_TOUPPER
// skipped Py_UNICODE_TOTITLE
// skipped Py_UNICODE_ISDECIMAL
// skipped Py_UNICODE_ISDIGIT
// skipped Py_UNICODE_ISNUMERIC
// skipped Py_UNICODE_ISPRINTABLE
// skipped Py_UNICODE_TODECIMAL
// skipped Py_UNICODE_TODIGIT
// skipped Py_UNICODE_TONUMERIC
// skipped Py_UNICODE_ISALPHA
// skipped Py_UNICODE_ISALNUM
// skipped Py_UNICODE_COPY
// skipped Py_UNICODE_FILL
// skipped Py_UNICODE_IS_SURROGATE
// skipped Py_UNICODE_IS_HIGH_SURROGATE
// skipped Py_UNICODE_IS_LOW_SURROGATE
// skipped Py_UNICODE_JOIN_SURROGATES
// skipped Py_UNICODE_HIGH_SURROGATE
// skipped Py_UNICODE_LOW_SURROGATE

// generated by bindgen v0.63.0 (with small adaptations)
#[repr(C)]
struct BitfieldUnit<Storage> {
    storage: Storage,
}

impl<Storage> BitfieldUnit<Storage> {
    #[inline]
    pub const fn new(storage: Storage) -> Self {
        Self { storage }
    }
}

#[cfg(not(GraalPy))]
impl<Storage> BitfieldUnit<Storage>
where
    Storage: AsRef<[u8]> + AsMut<[u8]>,
{
    #[inline]
    fn get_bit(&self, index: usize) -> bool {
        debug_assert!(index / 8 < self.storage.as_ref().len());
        let byte_index = index / 8;
        let byte = self.storage.as_ref()[byte_index];
        let bit_index = if cfg!(target_endian = "big") {
            7 - (index % 8)
        } else {
            index % 8
        };
        let mask = 1 << bit_index;
        byte & mask == mask
    }

    #[inline]
    fn set_bit(&mut self, index: usize, val: bool) {
        debug_assert!(index / 8 < self.storage.as_ref().len());
        let byte_index = index / 8;
        let byte = &mut self.storage.as_mut()[byte_index];
        let bit_index = if cfg!(target_endian = "big") {
            7 - (index % 8)
        } else {
            index % 8
        };
        let mask = 1 << bit_index;
        if val {
            *byte |= mask;
        } else {
            *byte &= !mask;
        }
    }

    #[inline]
    fn get(&self, bit_offset: usize, bit_width: u8) -> u64 {
        debug_assert!(bit_width <= 64);
        debug_assert!(bit_offset / 8 < self.storage.as_ref().len());
        debug_assert!((bit_offset + (bit_width as usize)) / 8 <= self.storage.as_ref().len());
        let mut val = 0;
        for i in 0..(bit_width as usize) {
            if self.get_bit(i + bit_offset) {
                let index = if cfg!(target_endian = "big") {
                    bit_width as usize - 1 - i
                } else {
                    i
                };
                val |= 1 << index;
            }
        }
        val
    }

    #[inline]
    fn set(&mut self, bit_offset: usize, bit_width: u8, val: u64) {
        debug_assert!(bit_width <= 64);
        debug_assert!(bit_offset / 8 < self.storage.as_ref().len());
        debug_assert!((bit_offset + (bit_width as usize)) / 8 <= self.storage.as_ref().len());
        for i in 0..(bit_width as usize) {
            let mask = 1 << i;
            let val_bit_is_set = val & mask == mask;
            let index = if cfg!(target_endian = "big") {
                bit_width as usize - 1 - i
            } else {
                i
            };
            self.set_bit(index + bit_offset, val_bit_is_set);
        }
    }
}

#[cfg(not(GraalPy))]
const STATE_INTERNED_INDEX: usize = 0;
#[cfg(all(not(GraalPy), not(Py_3_14)))]
const STATE_INTERNED_WIDTH: u8 = 2;
#[cfg(all(not(GraalPy), Py_3_14))]
const STATE_INTERNED_WIDTH: u8 = 16;

#[cfg(not(GraalPy))]
const STATE_KIND_INDEX: usize = STATE_INTERNED_WIDTH as usize;
#[cfg(not(GraalPy))]
const STATE_KIND_WIDTH: u8 = 3;

#[cfg(not(GraalPy))]
const STATE_COMPACT_INDEX: usize = (STATE_INTERNED_WIDTH + STATE_KIND_WIDTH) as usize;
#[cfg(not(GraalPy))]
const STATE_COMPACT_WIDTH: u8 = 1;

#[cfg(not(GraalPy))]
const STATE_ASCII_INDEX: usize =
    (STATE_INTERNED_WIDTH + STATE_KIND_WIDTH + STATE_COMPACT_WIDTH) as usize;
#[cfg(not(GraalPy))]
const STATE_ASCII_WIDTH: u8 = 1;

#[cfg(all(not(GraalPy), Py_3_12))]
const STATE_STATICALLY_ALLOCATED_INDEX: usize =
    (STATE_INTERNED_WIDTH + STATE_KIND_WIDTH + STATE_COMPACT_WIDTH + STATE_ASCII_WIDTH) as usize;
#[cfg(all(not(GraalPy), Py_3_12))]
const STATE_STATICALLY_ALLOCATED_WIDTH: u8 = 1;

#[cfg(not(any(Py_3_12, GraalPy)))]
const STATE_READY_INDEX: usize =
    (STATE_INTERNED_WIDTH + STATE_KIND_WIDTH + STATE_COMPACT_WIDTH + STATE_ASCII_WIDTH) as usize;
#[cfg(not(any(Py_3_12, GraalPy)))]
const STATE_READY_WIDTH: u8 = 1;

// generated by bindgen v0.63.0 (with small adaptations)
// The same code is generated for Python 3.7, 3.8, 3.9, 3.10, and 3.11, but the "ready" field
// has been removed from Python 3.12.

/// Wrapper around the `PyASCIIObject.state` bitfield with getters and setters that work
/// on most little- and big-endian architectures.
///
/// Memory layout of C bitfields is implementation defined, so these functions are still
/// unsafe. Users must verify that they work as expected on the architectures they target.
#[repr(C)]
#[repr(align(4))]
struct PyASCIIObjectState {
    bitfield_align: [u8; 0],
    bitfield: BitfieldUnit<[u8; 4usize]>,
}

// c_uint and u32 are not necessarily the same type on all targets / architectures
#[cfg(not(GraalPy))]
#[allow(clippy::useless_transmute)]
impl PyASCIIObjectState {
    #[inline]
    #[cfg(not(Py_3_14))]
    unsafe fn interned(&self) -> c_uint {
        std::mem::transmute(
            self.bitfield
                .get(STATE_INTERNED_INDEX, STATE_INTERNED_WIDTH) as u32,
        )
    }

    #[inline]
    #[cfg(not(Py_3_14))]
    unsafe fn set_interned(&mut self, val: c_uint) {
        let val: u32 = std::mem::transmute(val);
        self.bitfield
            .set(STATE_INTERNED_INDEX, STATE_INTERNED_WIDTH, val as u64)
    }

    #[inline]
    #[cfg(Py_3_14)]
    unsafe fn interned(&self) -> u16 {
        std::mem::transmute(
            self.bitfield
                .get(STATE_INTERNED_INDEX, STATE_INTERNED_WIDTH) as u16,
        )
    }

    #[inline]
    #[cfg(Py_3_14)]
    unsafe fn set_interned(&mut self, val: u16) {
        let val: u16 = std::mem::transmute(val);
        self.bitfield
            .set(STATE_INTERNED_INDEX, STATE_INTERNED_WIDTH, val as u64)
    }

    #[inline]
    #[cfg(not(Py_3_14))]
    unsafe fn kind(&self) -> c_uint {
        std::mem::transmute(self.bitfield.get(STATE_KIND_INDEX, STATE_KIND_WIDTH) as u32)
    }

    #[inline]
    #[cfg(Py_3_14)]
    unsafe fn kind(&self) -> c_ushort {
        std::mem::transmute(self.bitfield.get(STATE_KIND_INDEX, STATE_KIND_WIDTH) as c_ushort)
    }

    #[inline]
    #[cfg(not(Py_3_14))]
    unsafe fn set_kind(&mut self, val: c_uint) {
        let val: u32 = std::mem::transmute(val);
        self.bitfield
            .set(STATE_KIND_INDEX, STATE_KIND_WIDTH, val as u64)
    }

    #[inline]
    #[cfg(Py_3_14)]
    unsafe fn set_kind(&mut self, val: c_ushort) {
        let val: c_ushort = std::mem::transmute(val);
        self.bitfield
            .set(STATE_KIND_INDEX, STATE_KIND_WIDTH, val as u64)
    }

    #[inline]
    #[cfg(not(Py_3_14))]
    unsafe fn compact(&self) -> c_uint {
        std::mem::transmute(self.bitfield.get(STATE_COMPACT_INDEX, STATE_COMPACT_WIDTH) as u32)
    }

    #[inline]
    #[cfg(Py_3_14)]
    unsafe fn compact(&self) -> c_ushort {
        std::mem::transmute(self.bitfield.get(STATE_COMPACT_INDEX, STATE_COMPACT_WIDTH) as c_ushort)
    }

    #[inline]
    #[cfg(not(Py_3_14))]
    unsafe fn set_compact(&mut self, val: c_uint) {
        let val: u32 = std::mem::transmute(val);
        self.bitfield
            .set(STATE_COMPACT_INDEX, STATE_COMPACT_WIDTH, val as u64)
    }

    #[inline]
    #[cfg(Py_3_14)]
    unsafe fn set_compact(&mut self, val: c_ushort) {
        let val: c_ushort = std::mem::transmute(val);
        self.bitfield
            .set(STATE_COMPACT_INDEX, STATE_COMPACT_WIDTH, val as u64)
    }

    #[inline]
    #[cfg(not(Py_3_14))]
    unsafe fn ascii(&self) -> c_uint {
        std::mem::transmute(self.bitfield.get(STATE_ASCII_INDEX, STATE_ASCII_WIDTH) as u32)
    }

    #[inline]
    #[cfg(not(Py_3_14))]
    unsafe fn set_ascii(&mut self, val: c_uint) {
        let val: u32 = std::mem::transmute(val);
        self.bitfield
            .set(STATE_ASCII_INDEX, STATE_ASCII_WIDTH, val as u64)
    }

    #[inline]
    #[cfg(Py_3_14)]
    unsafe fn ascii(&self) -> c_ushort {
        std::mem::transmute(self.bitfield.get(STATE_ASCII_INDEX, STATE_ASCII_WIDTH) as c_ushort)
    }

    #[inline]
    #[cfg(Py_3_14)]
    unsafe fn set_ascii(&mut self, val: c_ushort) {
        let val: c_ushort = std::mem::transmute(val);
        self.bitfield
            .set(STATE_ASCII_INDEX, STATE_ASCII_WIDTH, val as u64)
    }

    #[cfg(all(Py_3_12, not(Py_3_14)))]
    #[inline]
    unsafe fn statically_allocated(&self) -> c_uint {
        std::mem::transmute(self.bitfield.get(
            STATE_STATICALLY_ALLOCATED_INDEX,
            STATE_STATICALLY_ALLOCATED_WIDTH,
        ) as u32)
    }

    #[cfg(all(Py_3_12, not(Py_3_14)))]
    #[inline]
    unsafe fn set_statically_allocated(&mut self, val: c_uint) {
        let val: u32 = std::mem::transmute(val);
        self.bitfield.set(
            STATE_STATICALLY_ALLOCATED_INDEX,
            STATE_STATICALLY_ALLOCATED_WIDTH,
            val as u64,
        )
    }

    #[inline]
    #[cfg(Py_3_14)]
    unsafe fn statically_allocated(&self) -> c_ushort {
        std::mem::transmute(self.bitfield.get(
            STATE_STATICALLY_ALLOCATED_INDEX,
            STATE_STATICALLY_ALLOCATED_WIDTH,
        ) as c_ushort)
    }

    #[inline]
    #[cfg(Py_3_14)]
    unsafe fn set_statically_allocated(&mut self, val: c_ushort) {
        let val: c_ushort = std::mem::transmute(val);
        self.bitfield.set(
            STATE_STATICALLY_ALLOCATED_INDEX,
            STATE_STATICALLY_ALLOCATED_WIDTH,
            val as u64,
        )
    }

    #[cfg(not(Py_3_12))]
    #[inline]
    unsafe fn ready(&self) -> c_uint {
        std::mem::transmute(self.bitfield.get(STATE_READY_INDEX, STATE_READY_WIDTH) as u32)
    }

    #[cfg(not(Py_3_12))]
    #[inline]
    unsafe fn set_ready(&mut self, val: c_uint) {
        let val: u32 = std::mem::transmute(val);
        self.bitfield
            .set(STATE_READY_INDEX, STATE_READY_WIDTH, val as u64)
    }
}

impl From<u32> for PyASCIIObjectState {
    #[inline]
    fn from(value: u32) -> Self {
        PyASCIIObjectState {
            bitfield_align: [],
            bitfield: BitfieldUnit::new(value.to_ne_bytes()),
        }
    }
}

impl From<PyASCIIObjectState> for u32 {
    #[inline]
    fn from(value: PyASCIIObjectState) -> Self {
        u32::from_ne_bytes(value.bitfield.storage)
    }
}

#[repr(C)]
pub struct PyASCIIObject {
    pub ob_base: PyObject,
    pub length: Py_ssize_t,
    #[cfg(any(Py_3_11, not(PyPy)))]
    pub hash: Py_hash_t,
    /// A bit field with various properties.
    ///
    /// Rust doesn't expose bitfields. So we have accessor functions for
    /// retrieving values.
    ///
    /// Before 3.12:
    /// unsigned int interned:2; // SSTATE_* constants.
    /// unsigned int kind:3;     // PyUnicode_*_KIND constants.
    /// unsigned int compact:1;
    /// unsigned int ascii:1;
    /// unsigned int ready:1;
    /// unsigned int :24;
    ///
    /// 3.12 and 3.13:
    /// unsigned int interned:2; // SSTATE_* constants.
    /// unsigned int kind:3;     // PyUnicode_*_KIND constants.
    /// unsigned int compact:1;
    /// unsigned int ascii:1;
    /// unsigned int statically_allocated:1;
    /// unsigned int :24;
    ///
    /// 3.14 and later:
    /// uint16_t interned;   // SSTATE_* constants.
    /// unsigned short kind:3; // PyUnicode_*_KIND constants.
    /// unsigned short compact:1;
    /// unsigned short ascii:1;
    /// unsigned int statically_allocated:1;
    /// unsigned int :10;
    pub state: u32,
    #[cfg(not(Py_3_12))]
    pub wstr: *mut wchar_t,
}

/// Interacting with the bitfield is not actually well-defined, so we mark these APIs unsafe.
#[cfg(not(GraalPy))]
impl PyASCIIObject {
    #[cfg_attr(not(Py_3_12), allow(rustdoc::broken_intra_doc_links))] // SSTATE_INTERNED_IMMORTAL_STATIC requires 3.12
    /// Get the `interned` field of the [`PyASCIIObject`] state bitfield.
    ///
    /// Returns one of: [`SSTATE_NOT_INTERNED`], [`SSTATE_INTERNED_MORTAL`],
    /// [`SSTATE_INTERNED_IMMORTAL`], or [`SSTATE_INTERNED_IMMORTAL_STATIC`].
    #[inline]
    #[cfg(not(Py_3_14))]
    pub unsafe fn interned(&self) -> c_uint {
        PyASCIIObjectState::from(self.state).interned()
    }

    #[cfg_attr(not(Py_3_12), allow(rustdoc::broken_intra_doc_links))] // SSTATE_INTERNED_IMMORTAL_STATIC requires 3.12
    /// Set the `interned` field of the [`PyASCIIObject`] state bitfield.
    ///
    /// Calling this function with an argument that is not [`SSTATE_NOT_INTERNED`],
    /// [`SSTATE_INTERNED_MORTAL`], [`SSTATE_INTERNED_IMMORTAL`], or
    /// [`SSTATE_INTERNED_IMMORTAL_STATIC`] is invalid.
    #[inline]
    #[cfg(not(Py_3_14))]
    pub unsafe fn set_interned(&mut self, val: c_uint) {
        let mut state = PyASCIIObjectState::from(self.state);
        state.set_interned(val);
        self.state = u32::from(state);
    }

    #[cfg_attr(not(Py_3_12), allow(rustdoc::broken_intra_doc_links))] // SSTATE_INTERNED_IMMORTAL_STATIC requires 3.12
    /// Get the `interned` field of the [`PyASCIIObject`] state bitfield.
    ///
    /// Returns one of: [`SSTATE_NOT_INTERNED`], [`SSTATE_INTERNED_MORTAL`],
    /// [`SSTATE_INTERNED_IMMORTAL`], or [`SSTATE_INTERNED_IMMORTAL_STATIC`].
    #[inline]
    #[cfg(Py_3_14)]
    pub unsafe fn interned(&self) -> u16 {
        PyASCIIObjectState::from(self.state).interned()
    }

    #[cfg_attr(not(Py_3_12), allow(rustdoc::broken_intra_doc_links))] // SSTATE_INTERNED_IMMORTAL_STATIC requires 3.12
    /// Set the `interned` field of the [`PyASCIIObject`] state bitfield.
    ///
    /// Calling this function with an argument that is not [`SSTATE_NOT_INTERNED`],
    /// [`SSTATE_INTERNED_MORTAL`], [`SSTATE_INTERNED_IMMORTAL`], or
    /// [`SSTATE_INTERNED_IMMORTAL_STATIC`] is invalid.
    #[inline]
    #[cfg(Py_3_14)]
    pub unsafe fn set_interned(&mut self, val: u16) {
        let mut state = PyASCIIObjectState::from(self.state);
        state.set_interned(val);
        self.state = u32::from(state);
    }

    /// Get the `kind` field of the [`PyASCIIObject`] state bitfield.
    ///
    /// Returns one of:
    #[cfg_attr(not(Py_3_12), doc = "[`PyUnicode_WCHAR_KIND`], ")]
    /// [`PyUnicode_1BYTE_KIND`], [`PyUnicode_2BYTE_KIND`], or [`PyUnicode_4BYTE_KIND`].
    #[inline]
    #[cfg(not(Py_3_14))]
    pub unsafe fn kind(&self) -> c_uint {
        PyASCIIObjectState::from(self.state).kind()
    }

    /// Get the `kind` field of the [`PyASCIIObject`] state bitfield.
    ///
    /// Returns one of:
    #[cfg_attr(not(Py_3_12), doc = "[`PyUnicode_WCHAR_KIND`], ")]
    /// [`PyUnicode_1BYTE_KIND`], [`PyUnicode_2BYTE_KIND`], or [`PyUnicode_4BYTE_KIND`].
    #[inline]
    #[cfg(Py_3_14)]
    pub unsafe fn kind(&self) -> c_ushort {
        PyASCIIObjectState::from(self.state).kind()
    }

    /// Set the `kind` field of the [`PyASCIIObject`] state bitfield.
    ///
    /// Calling this function with an argument that is not
    #[cfg_attr(not(Py_3_12), doc = "[`PyUnicode_WCHAR_KIND`], ")]
    /// [`PyUnicode_1BYTE_KIND`], [`PyUnicode_2BYTE_KIND`], or [`PyUnicode_4BYTE_KIND`] is invalid.
    #[inline]
    #[cfg(not(Py_3_14))]
    pub unsafe fn set_kind(&mut self, val: c_uint) {
        let mut state = PyASCIIObjectState::from(self.state);
        state.set_kind(val);
        self.state = u32::from(state);
    }

    /// Set the `kind` field of the [`PyASCIIObject`] state bitfield.
    ///
    /// Calling this function with an argument that is not
    #[cfg_attr(not(Py_3_12), doc = "[`PyUnicode_WCHAR_KIND`], ")]
    /// [`PyUnicode_1BYTE_KIND`], [`PyUnicode_2BYTE_KIND`], or [`PyUnicode_4BYTE_KIND`] is invalid.
    #[inline]
    #[cfg(Py_3_14)]
    pub unsafe fn set_kind(&mut self, val: c_ushort) {
        let mut state = PyASCIIObjectState::from(self.state);
        state.set_kind(val);
        self.state = u32::from(state);
    }

    /// Get the `compact` field of the [`PyASCIIObject`] state bitfield.
    ///
    /// Returns either `0` or `1`.
    #[inline]
    #[cfg(not(Py_3_14))]
    pub unsafe fn compact(&self) -> c_uint {
        PyASCIIObjectState::from(self.state).compact()
    }

    /// Get the `compact` field of the [`PyASCIIObject`] state bitfield.
    ///
    /// Returns either `0` or `1`.
    #[inline]
    #[cfg(Py_3_14)]
    pub unsafe fn compact(&self) -> c_ushort {
        PyASCIIObjectState::from(self.state).compact()
    }

    /// Set the `compact` flag of the [`PyASCIIObject`] state bitfield.
    ///
    /// Calling this function with an argument that is neither `0` nor `1` is invalid.
    #[inline]
    #[cfg(not(Py_3_14))]
    pub unsafe fn set_compact(&mut self, val: c_uint) {
        let mut state = PyASCIIObjectState::from(self.state);
        state.set_compact(val);
        self.state = u32::from(state);
    }

    /// Set the `compact` flag of the [`PyASCIIObject`] state bitfield.
    ///
    /// Calling this function with an argument that is neither `0` nor `1` is invalid.
    #[inline]
    #[cfg(Py_3_14)]
    pub unsafe fn set_compact(&mut self, val: c_ushort) {
        let mut state = PyASCIIObjectState::from(self.state);
        state.set_compact(val);
        self.state = u32::from(state);
    }

    /// Get the `ascii` field of the [`PyASCIIObject`] state bitfield.
    ///
    /// Returns either `0` or `1`.
    #[inline]
    #[cfg(not(Py_3_14))]
    pub unsafe fn ascii(&self) -> c_uint {
        PyASCIIObjectState::from(self.state).ascii()
    }

    /// Set the `ascii` flag of the [`PyASCIIObject`] state bitfield.
    ///
    /// Calling this function with an argument that is neither `0` nor `1` is invalid.
    #[inline]
    #[cfg(not(Py_3_14))]
    pub unsafe fn set_ascii(&mut self, val: c_uint) {
        let mut state = PyASCIIObjectState::from(self.state);
        state.set_ascii(val);
        self.state = u32::from(state);
    }

    /// Get the `ascii` field of the [`PyASCIIObject`] state bitfield.
    ///
    /// Returns either `0` or `1`.
    #[inline]
    #[cfg(Py_3_14)]
    pub unsafe fn ascii(&self) -> c_ushort {
        PyASCIIObjectState::from(self.state).ascii()
    }

    /// Set the `ascii` flag of the [`PyASCIIObject`] state bitfield.
    ///
    /// Calling this function with an argument that is neither `0` nor `1` is invalid.
    #[inline]
    #[cfg(Py_3_14)]
    pub unsafe fn set_ascii(&mut self, val: c_ushort) {
        let mut state = PyASCIIObjectState::from(self.state);
        state.set_ascii(val);
        self.state = u32::from(state);
    }

    /// Get the `ready` field of the [`PyASCIIObject`] state bitfield.
    ///
    /// Returns either `0` or `1`.
    #[cfg(not(Py_3_12))]
    #[inline]
    pub unsafe fn ready(&self) -> c_uint {
        PyASCIIObjectState::from(self.state).ready()
    }

    /// Set the `ready` flag of the [`PyASCIIObject`] state bitfield.
    ///
    /// Calling this function with an argument that is neither `0` nor `1` is invalid.
    #[cfg(not(Py_3_12))]
    #[inline]
    pub unsafe fn set_ready(&mut self, val: c_uint) {
        let mut state = PyASCIIObjectState::from(self.state);
        state.set_ready(val);
        self.state = u32::from(state);
    }

    /// Get the `statically_allocated` field of the [`PyASCIIObject`] state bitfield.
    ///
    /// Returns either `0` or `1`.
    #[inline]
    #[cfg(all(Py_3_12, not(Py_3_14)))]
    pub unsafe fn statically_allocated(&self) -> c_uint {
        PyASCIIObjectState::from(self.state).statically_allocated()
    }

    /// Set the `statically_allocated` flag of the [`PyASCIIObject`] state bitfield.
    ///
    /// Calling this function with an argument that is neither `0` nor `1` is invalid.
    #[inline]
    #[cfg(all(Py_3_12, not(Py_3_14)))]
    pub unsafe fn set_statically_allocated(&mut self, val: c_uint) {
        let mut state = PyASCIIObjectState::from(self.state);
        state.set_statically_allocated(val);
        self.state = u32::from(state);
    }

    /// Get the `statically_allocated` field of the [`PyASCIIObject`] state bitfield.
    ///
    /// Returns either `0` or `1`.
    #[inline]
    #[cfg(Py_3_14)]
    pub unsafe fn statically_allocated(&self) -> c_ushort {
        PyASCIIObjectState::from(self.state).statically_allocated()
    }

    /// Set the `statically_allocated` flag of the [`PyASCIIObject`] state bitfield.
    ///
    /// Calling this function with an argument that is neither `0` nor `1` is invalid.
    #[inline]
    #[cfg(Py_3_14)]
    pub unsafe fn set_statically_allocated(&mut self, val: c_ushort) {
        let mut state = PyASCIIObjectState::from(self.state);
        state.set_statically_allocated(val);
        self.state = u32::from(state);
    }
}

#[repr(C)]
pub struct PyCompactUnicodeObject {
    pub _base: PyASCIIObject,
    pub utf8_length: Py_ssize_t,
    pub utf8: *mut c_char,
    #[cfg(not(Py_3_12))]
    pub wstr_length: Py_ssize_t,
}

#[repr(C)]
pub union PyUnicodeObjectData {
    pub any: *mut c_void,
    pub latin1: *mut Py_UCS1,
    pub ucs2: *mut Py_UCS2,
    pub ucs4: *mut Py_UCS4,
}

#[repr(C)]
pub struct PyUnicodeObject {
    pub _base: PyCompactUnicodeObject,
    pub data: PyUnicodeObjectData,
}

extern "C" {
    #[cfg(not(any(PyPy, GraalPy)))]
    pub fn _PyUnicode_CheckConsistency(op: *mut PyObject, check_content: c_int) -> c_int;
}

// skipped PyUnicode_GET_SIZE
// skipped PyUnicode_GET_DATA_SIZE
// skipped PyUnicode_AS_UNICODE
// skipped PyUnicode_AS_DATA

pub const SSTATE_NOT_INTERNED: c_uint = 0;
pub const SSTATE_INTERNED_MORTAL: c_uint = 1;
pub const SSTATE_INTERNED_IMMORTAL: c_uint = 2;
#[cfg(Py_3_12)]
pub const SSTATE_INTERNED_IMMORTAL_STATIC: c_uint = 3;

#[cfg(all(not(GraalPy), not(Py_3_14)))]
#[inline]
pub unsafe fn PyUnicode_IS_ASCII(op: *mut PyObject) -> c_uint {
    debug_assert!(crate::PyUnicode_Check(op) != 0);
    #[cfg(not(Py_3_12))]
    debug_assert!(PyUnicode_IS_READY(op) != 0);

    (*(op as *mut PyASCIIObject)).ascii()
}

#[cfg(all(not(GraalPy), not(Py_3_14)))]
#[inline]
pub unsafe fn PyUnicode_IS_COMPACT(op: *mut PyObject) -> c_uint {
    (*(op as *mut PyASCIIObject)).compact()
}

#[cfg(all(not(GraalPy), Py_3_14))]
#[inline]
pub unsafe fn PyUnicode_IS_ASCII(op: *mut PyObject) -> c_ushort {
    debug_assert!(crate::PyUnicode_Check(op) != 0);
    #[cfg(not(Py_3_12))]
    debug_assert!(PyUnicode_IS_READY(op) != 0);

    (*(op as *mut PyASCIIObject)).ascii()
}

#[cfg(all(not(GraalPy), Py_3_14))]
#[inline]
pub unsafe fn PyUnicode_IS_COMPACT(op: *mut PyObject) -> c_ushort {
    (*(op as *mut PyASCIIObject)).compact()
}

#[cfg(not(GraalPy))]
#[inline]
pub unsafe fn PyUnicode_IS_COMPACT_ASCII(op: *mut PyObject) -> c_uint {
    ((*(op as *mut PyASCIIObject)).ascii() != 0 && PyUnicode_IS_COMPACT(op) != 0).into()
}

#[cfg(not(Py_3_12))]
#[deprecated(note = "Removed in Python 3.12")]
pub const PyUnicode_WCHAR_KIND: c_uint = 0;

#[cfg(not(Py_3_14))]
pub const PyUnicode_1BYTE_KIND: c_uint = 1;
#[cfg(not(Py_3_14))]
pub const PyUnicode_2BYTE_KIND: c_uint = 2;
#[cfg(not(Py_3_14))]
pub const PyUnicode_4BYTE_KIND: c_uint = 4;

#[cfg(Py_3_14)]
pub const PyUnicode_1BYTE_KIND: c_ushort = 1;
#[cfg(Py_3_14)]
pub const PyUnicode_2BYTE_KIND: c_ushort = 2;
#[cfg(Py_3_14)]
pub const PyUnicode_4BYTE_KIND: c_ushort = 4;

#[cfg(not(any(GraalPy, PyPy)))]
#[inline]
pub unsafe fn PyUnicode_1BYTE_DATA(op: *mut PyObject) -> *mut Py_UCS1 {
    PyUnicode_DATA(op) as *mut Py_UCS1
}

#[cfg(not(any(GraalPy, PyPy)))]
#[inline]
pub unsafe fn PyUnicode_2BYTE_DATA(op: *mut PyObject) -> *mut Py_UCS2 {
    PyUnicode_DATA(op) as *mut Py_UCS2
}

#[cfg(not(any(GraalPy, PyPy)))]
#[inline]
pub unsafe fn PyUnicode_4BYTE_DATA(op: *mut PyObject) -> *mut Py_UCS4 {
    PyUnicode_DATA(op) as *mut Py_UCS4
}

#[cfg(all(not(GraalPy), not(Py_3_14)))]
#[inline]
pub unsafe fn PyUnicode_KIND(op: *mut PyObject) -> c_uint {
    debug_assert!(crate::PyUnicode_Check(op) != 0);
    #[cfg(not(Py_3_12))]
    debug_assert!(PyUnicode_IS_READY(op) != 0);

    (*(op as *mut PyASCIIObject)).kind()
}

#[cfg(all(not(GraalPy), Py_3_14))]
#[inline]
pub unsafe fn PyUnicode_KIND(op: *mut PyObject) -> c_ushort {
    debug_assert!(crate::PyUnicode_Check(op) != 0);
    #[cfg(not(Py_3_12))]
    debug_assert!(PyUnicode_IS_READY(op) != 0);

    (*(op as *mut PyASCIIObject)).kind()
}

#[cfg(not(GraalPy))]
#[inline]
pub unsafe fn _PyUnicode_COMPACT_DATA(op: *mut PyObject) -> *mut c_void {
    if PyUnicode_IS_ASCII(op) != 0 {
        (op as *mut PyASCIIObject).offset(1) as *mut c_void
    } else {
        (op as *mut PyCompactUnicodeObject).offset(1) as *mut c_void
    }
}

#[cfg(not(any(GraalPy, PyPy)))]
#[inline]
pub unsafe fn _PyUnicode_NONCOMPACT_DATA(op: *mut PyObject) -> *mut c_void {
    debug_assert!(!(*(op as *mut PyUnicodeObject)).data.any.is_null());

    (*(op as *mut PyUnicodeObject)).data.any
}

#[cfg(not(any(GraalPy, PyPy)))]
#[inline]
pub unsafe fn PyUnicode_DATA(op: *mut PyObject) -> *mut c_void {
    debug_assert!(crate::PyUnicode_Check(op) != 0);

    if PyUnicode_IS_COMPACT(op) != 0 {
        _PyUnicode_COMPACT_DATA(op)
    } else {
        _PyUnicode_NONCOMPACT_DATA(op)
    }
}

// skipped PyUnicode_WRITE
// skipped PyUnicode_READ
// skipped PyUnicode_READ_CHAR

#[cfg(not(GraalPy))]
#[inline]
pub unsafe fn PyUnicode_GET_LENGTH(op: *mut PyObject) -> Py_ssize_t {
    debug_assert!(crate::PyUnicode_Check(op) != 0);
    #[cfg(not(Py_3_12))]
    debug_assert!(PyUnicode_IS_READY(op) != 0);

    (*(op as *mut PyASCIIObject)).length
}

#[cfg(any(Py_3_12, GraalPy))]
#[inline]
pub unsafe fn PyUnicode_IS_READY(_op: *mut PyObject) -> c_uint {
    // kept in CPython for backwards compatibility
    1
}

#[cfg(not(any(GraalPy, Py_3_12)))]
#[inline]
pub unsafe fn PyUnicode_IS_READY(op: *mut PyObject) -> c_uint {
    (*(op as *mut PyASCIIObject)).ready()
}

#[cfg(any(Py_3_12, GraalPy))]
#[inline]
pub unsafe fn PyUnicode_READY(_op: *mut PyObject) -> c_int {
    0
}

#[cfg(not(any(Py_3_12, GraalPy)))]
#[inline]
pub unsafe fn PyUnicode_READY(op: *mut PyObject) -> c_int {
    debug_assert!(crate::PyUnicode_Check(op) != 0);

    if PyUnicode_IS_READY(op) != 0 {
        0
    } else {
        _PyUnicode_Ready(op)
    }
}

// skipped PyUnicode_MAX_CHAR_VALUE
// skipped _PyUnicode_get_wstr_length
// skipped PyUnicode_WSTR_LENGTH

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_New")]
    pub fn PyUnicode_New(size: Py_ssize_t, maxchar: Py_UCS4) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "_PyPyUnicode_Ready")]
    pub fn _PyUnicode_Ready(unicode: *mut PyObject) -> c_int;

    // skipped _PyUnicode_Copy

    #[cfg(not(PyPy))]
    pub fn PyUnicode_CopyCharacters(
        to: *mut PyObject,
        to_start: Py_ssize_t,
        from: *mut PyObject,
        from_start: Py_ssize_t,
        how_many: Py_ssize_t,
    ) -> Py_ssize_t;

    // skipped _PyUnicode_FastCopyCharacters

    #[cfg(not(PyPy))]
    pub fn PyUnicode_Fill(
        unicode: *mut PyObject,
        start: Py_ssize_t,
        length: Py_ssize_t,
        fill_char: Py_UCS4,
    ) -> Py_ssize_t;

    // skipped _PyUnicode_FastFill

    #[cfg(not(Py_3_12))]
    #[deprecated]
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_FromUnicode")]
    pub fn PyUnicode_FromUnicode(u: *const wchar_t, size: Py_ssize_t) -> *mut PyObject;

    #[cfg_attr(PyPy, link_name = "PyPyUnicode_FromKindAndData")]
    pub fn PyUnicode_FromKindAndData(
        kind: c_int,
        buffer: *const c_void,
        size: Py_ssize_t,
    ) -> *mut PyObject;

    // skipped _PyUnicode_FromASCII
    // skipped _PyUnicode_FindMaxChar

    #[cfg(not(Py_3_12))]
    #[deprecated]
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_AsUnicode")]
    pub fn PyUnicode_AsUnicode(unicode: *mut PyObject) -> *mut wchar_t;

    // skipped _PyUnicode_AsUnicode

    #[cfg(not(Py_3_12))]
    #[deprecated]
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_AsUnicodeAndSize")]
    pub fn PyUnicode_AsUnicodeAndSize(
        unicode: *mut PyObject,
        size: *mut Py_ssize_t,
    ) -> *mut wchar_t;

    // skipped PyUnicode_GetMax
}

// skipped _PyUnicodeWriter
// skipped _PyUnicodeWriter_Init
// skipped _PyUnicodeWriter_Prepare
// skipped _PyUnicodeWriter_PrepareInternal
// skipped _PyUnicodeWriter_PrepareKind
// skipped _PyUnicodeWriter_PrepareKindInternal
// skipped _PyUnicodeWriter_WriteChar
// skipped _PyUnicodeWriter_WriteStr
// skipped _PyUnicodeWriter_WriteSubstring
// skipped _PyUnicodeWriter_WriteASCIIString
// skipped _PyUnicodeWriter_WriteLatin1String
// skipped _PyUnicodeWriter_Finish
// skipped _PyUnicodeWriter_Dealloc
// skipped _PyUnicode_FormatAdvancedWriter

extern "C" {
    // skipped _PyUnicode_AsStringAndSize

    #[cfg_attr(PyPy, link_name = "PyPyUnicode_AsUTF8")]
    pub fn PyUnicode_AsUTF8(unicode: *mut PyObject) -> *const c_char;

    // skipped _PyUnicode_AsString

    pub fn PyUnicode_Encode(
        s: *const wchar_t,
        size: Py_ssize_t,
        encoding: *const c_char,
        errors: *const c_char,
    ) -> *mut PyObject;

    pub fn PyUnicode_EncodeUTF7(
        data: *const wchar_t,
        length: Py_ssize_t,
        base64SetO: c_int,
        base64WhiteSpace: c_int,
        errors: *const c_char,
    ) -> *mut PyObject;

    // skipped _PyUnicode_EncodeUTF7
    // skipped _PyUnicode_AsUTF8String

    #[cfg_attr(PyPy, link_name = "PyPyUnicode_EncodeUTF8")]
    pub fn PyUnicode_EncodeUTF8(
        data: *const wchar_t,
        length: Py_ssize_t,
        errors: *const c_char,
    ) -> *mut PyObject;

    pub fn PyUnicode_EncodeUTF32(
        data: *const wchar_t,
        length: Py_ssize_t,
        errors: *const c_char,
        byteorder: c_int,
    ) -> *mut PyObject;

    // skipped _PyUnicode_EncodeUTF32

    pub fn PyUnicode_EncodeUTF16(
        data: *const wchar_t,
        length: Py_ssize_t,
        errors: *const c_char,
        byteorder: c_int,
    ) -> *mut PyObject;

    // skipped _PyUnicode_EncodeUTF16
    // skipped _PyUnicode_DecodeUnicodeEscape

    pub fn PyUnicode_EncodeUnicodeEscape(data: *const wchar_t, length: Py_ssize_t)
        -> *mut PyObject;

    pub fn PyUnicode_EncodeRawUnicodeEscape(
        data: *const wchar_t,
        length: Py_ssize_t,
    ) -> *mut PyObject;

    // skipped _PyUnicode_AsLatin1String

    #[cfg_attr(PyPy, link_name = "PyPyUnicode_EncodeLatin1")]
    pub fn PyUnicode_EncodeLatin1(
        data: *const wchar_t,
        length: Py_ssize_t,
        errors: *const c_char,
    ) -> *mut PyObject;

    // skipped _PyUnicode_AsASCIIString

    #[cfg_attr(PyPy, link_name = "PyPyUnicode_EncodeASCII")]
    pub fn PyUnicode_EncodeASCII(
        data: *const wchar_t,
        length: Py_ssize_t,
        errors: *const c_char,
    ) -> *mut PyObject;

    pub fn PyUnicode_EncodeCharmap(
        data: *const wchar_t,
        length: Py_ssize_t,
        mapping: *mut PyObject,
        errors: *const c_char,
    ) -> *mut PyObject;

    // skipped _PyUnicode_EncodeCharmap

    pub fn PyUnicode_TranslateCharmap(
        data: *const wchar_t,
        length: Py_ssize_t,
        table: *mut PyObject,
        errors: *const c_char,
    ) -> *mut PyObject;

    // skipped PyUnicode_EncodeMBCS

    #[cfg_attr(PyPy, link_name = "PyPyUnicode_EncodeDecimal")]
    pub fn PyUnicode_EncodeDecimal(
        s: *mut wchar_t,
        length: Py_ssize_t,
        output: *mut c_char,
        errors: *const c_char,
    ) -> c_int;

    #[cfg_attr(PyPy, link_name = "PyPyUnicode_TransformDecimalToASCII")]
    pub fn PyUnicode_TransformDecimalToASCII(s: *mut wchar_t, length: Py_ssize_t) -> *mut PyObject;

    // skipped _PyUnicode_TransformDecimalAndSpaceToASCII
}

// skipped _PyUnicode_JoinArray
// skipped _PyUnicode_EqualToASCIIId
// skipped _PyUnicode_EqualToASCIIString
// skipped _PyUnicode_XStrip
// skipped _PyUnicode_InsertThousandsGrouping

// skipped _Py_ascii_whitespace

// skipped _PyUnicode_IsLowercase
// skipped _PyUnicode_IsUppercase
// skipped _PyUnicode_IsTitlecase
// skipped _PyUnicode_IsXidStart
// skipped _PyUnicode_IsXidContinue
// skipped _PyUnicode_IsWhitespace
// skipped _PyUnicode_IsLinebreak
// skipped _PyUnicode_ToLowercase
// skipped _PyUnicode_ToUppercase
// skipped _PyUnicode_ToTitlecase
// skipped _PyUnicode_ToLowerFull
// skipped _PyUnicode_ToTitleFull
// skipped _PyUnicode_ToUpperFull
// skipped _PyUnicode_ToFoldedFull
// skipped _PyUnicode_IsCaseIgnorable
// skipped _PyUnicode_IsCased
// skipped _PyUnicode_ToDecimalDigit
// skipped _PyUnicode_ToDigit
// skipped _PyUnicode_ToNumeric
// skipped _PyUnicode_IsDecimalDigit
// skipped _PyUnicode_IsDigit
// skipped _PyUnicode_IsNumeric
// skipped _PyUnicode_IsPrintable
// skipped _PyUnicode_IsAlpha
// skipped Py_UNICODE_strlen
// skipped Py_UNICODE_strcpy
// skipped Py_UNICODE_strcat
// skipped Py_UNICODE_strncpy
// skipped Py_UNICODE_strcmp
// skipped Py_UNICODE_strncmp
// skipped Py_UNICODE_strchr
// skipped Py_UNICODE_strrchr
// skipped _PyUnicode_FormatLong
// skipped PyUnicode_AsUnicodeCopy
// skipped _PyUnicode_FromId
// skipped _PyUnicode_EQ
// skipped _PyUnicode_ScanIdentifier

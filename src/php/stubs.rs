//! Stubs for PHP Zend API functions to allow Rust unit tests to link and run.
//! These are only used during testing.

#![allow(non_snake_case)]
#![allow(unused_variables)]

use std::os::raw::{c_char, c_int, c_void};

#[no_mangle]
pub extern "C" fn zend_hash_next_index_insert() -> *mut c_void {
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn zend_hash_str_update() -> *mut c_void {
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn zend_hash_index_update() -> *mut c_void {
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn zval_ptr_dtor() {}

#[no_mangle]
pub extern "C" fn zend_array_count() -> usize {
    0
}

#[no_mangle]
pub extern "C" fn _zend_new_array() -> *mut c_void {
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn zend_array_destroy() {}

#[no_mangle]
pub extern "C" fn zend_hash_get_current_key_type_ex() -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn zend_hash_get_current_key_zval_ex() {}

#[no_mangle]
pub extern "C" fn zend_hash_get_current_data_ex() -> *mut c_void {
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn zend_hash_move_forward_ex() {}

#[no_mangle]
pub extern "C" fn _emalloc(size: usize) -> *mut c_void {
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn __zend_malloc(size: usize) -> *mut c_void {
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn _efree(ptr: *mut c_void) {}

#[no_mangle]
pub extern "C" fn zend_objects_store_del() {}

#[no_mangle]
pub extern "C" fn gc_possible_root() {}

#[no_mangle]
pub extern "C" fn zend_error(type_: c_int, format: *const c_char) {}

#[no_mangle]
pub extern "C" fn zend_wrong_param_count() {}

#[no_mangle]
pub extern "C" fn zend_parse_parameters(num_args: c_int, type_spec: *const c_char) -> c_int {
    0
}

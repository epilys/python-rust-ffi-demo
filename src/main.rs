use std::ffi::OsStr;
use std::ffi::{CStr, CString};
use std::io::Read;
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

mod bindings;
use bindings::*;

mod python;
use python::*;
pub const EmbMethod: PyMethodDef = PyMethodDef {
    ml_name: b"numargs\0".as_ptr() as _,
    ml_meth: Some(emb_numargs),
    ml_flags: METH_VARARGS,
    ml_doc: b"Return the number of arguments received by the process.\0".as_ptr() as _,
};
pub const EmbSMethod: PyMethodDef = PyMethodDef {
    ml_name: b"s\0".as_ptr() as _,
    ml_meth: Some(emb_s),
    ml_flags: METH_VARARGS,
    ml_doc: b"\0".as_ptr() as _,
};
pub const EmbOMethod: PyMethodDef = PyMethodDef {
    ml_name: b"o\0".as_ptr() as _,
    ml_meth: Some(emb_o),
    ml_flags: METH_VARARGS,
    ml_doc: b"\0".as_ptr() as _,
};

pub const EmbMethods: Methods<3> = Methods::new([EmbMethod, EmbSMethod, EmbOMethod]);

pub const EmbModule: PyModuleDef = PyModuleDef {
    m_base: PyModuleDef_HEAD_INIT,
    m_name: b"emb\0".as_ptr() as _,
    m_doc: b"asdfas\0".as_ptr() as _,
    m_size: -1,
    m_methods: &EmbMethods as *const _ as *mut PyMethodDef,
    //m_slots: &EmptySlot as *const _ as *mut _,
    m_slots: std::ptr::null_mut(),
    m_traverse: None,
    m_clear: None,
    m_free: None,
};

unsafe extern "C" fn PyInit_emb() -> *mut PyObject {
    let lib = get_lib().unwrap();
    let ret = unsafe { lib.PyModule_Create2(&mut EmbModule as *mut _, 1013) };

    std::mem::forget(lib);
    ret
}

// Returns 42
unsafe extern "C" fn emb_numargs(_self: *mut PyObject, args: *mut PyObject) -> *mut PyObject {
    let lib = get_lib().unwrap();
    let ret = unsafe { lib.PyLong_FromLong(42) };
    //std::mem::forget(lib);
    ret
}

// Prints a python string
unsafe extern "C" fn emb_s(_self: *mut PyObject, args: *mut PyObject) -> *mut PyObject {
    let lib = get_lib().unwrap();
    let mut s: *mut ::std::os::raw::c_char = std::ptr::null_mut();
    if (lib.PyArg_ParseTuple.as_ref().unwrap())(args, b"s\0".as_ptr() as _, &mut s as *mut *mut _)
        == 0
    {
        return std::ptr::null_mut();
    }
    let cs = unsafe { std::ffi::CStr::from_ptr(s) };
    println!("emb_s s = {:?}", cs);
    let ret = return_none(&lib);
    //std::mem::forget(lib);
    ret
}

// Prints input type
unsafe extern "C" fn emb_o(_self: *mut PyObject, args: *mut PyObject) -> *mut PyObject {
    let lib = get_lib().unwrap();
    let mut o: *mut PyObject = std::ptr::null_mut();
    if (lib.PyArg_ParseTuple.as_ref().unwrap())(args, b"O\0".as_ptr() as _, &mut o as *mut *mut _)
        == 0
    {
        return std::ptr::null_mut();
    }
    let typeobj: *const PyTypeObject = unsafe { (*_self).ob_type };
    let cs = unsafe { std::ffi::CStr::from_ptr((*typeobj).tp_name) };
    println!("emb_o o = {:?}", cs);
    match cs.to_bytes() {
        b"CharKeyInput" => {}
        b"AltKeyInput" => {}
        b"CtrlKeyInput" => {}
        b"EscKeyInput" => {}
        b"PasteKeyInput" => {}
        b"SpecialKey" => {}
        b"FunctionKeyInput" => {}
        _ => {}
    }
    let ret = return_none(&lib);
    //std::mem::forget(lib);
    ret
}

fn main() -> Result<(), Error> {
    /* Return the number of arguments of the application command line */
    //let lparam = closure_pointer_pointer as usize;

    let mut python = PythonBuilder::new()?
        .with_module("emb", PyInit_emb)?
        .with_program_name("python_rust_demo")?
        .build()?;

    python.load_module_from_file("libmeliplugin", "libmeliplugin.py")?;
    let mut input_cstr = file_to_cstring("input.py".as_ref()).unwrap();
    let ret = python.run_code(&input_cstr)?;
    println!("ret: {:?}", ret);
    Ok(())
}

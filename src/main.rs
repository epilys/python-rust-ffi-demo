use std::ffi::OsStr;
use std::io::Read;
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;
use std::process::Command;

mod bindings;
use bindings::*;

const METH_VARARGS: i32 = 0x0001;

#[inline]
pub unsafe fn return_none(lib: &PythonLib) -> *mut PyObject {
    let none = lib._Py_NoneStruct as *const PyObject as *mut PyObject;
    lib.Py_IncRef(none);
    none
}

pub const PyObject_HEAD_INIT: PyObject = PyObject {
    ob_refcnt: 1,
    ob_type: std::ptr::null_mut(),
};
pub const PyModuleDef_HEAD_INIT: PyModuleDef_Base = PyModuleDef_Base {
    ob_base: PyObject_HEAD_INIT,
    m_init: None,
    m_index: 0,
    m_copy: std::ptr::null_mut(),
};

#[repr(C)]
pub struct Methods {
    methods: [PyMethodDef; 3],
}
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
pub const SentinelMethod: PyMethodDef = PyMethodDef {
    ml_name: std::ptr::null(),
    ml_meth: None,
    ml_flags: 0,
    ml_doc: std::ptr::null(),
};

pub const EmbMethods: Methods = Methods {
    methods: [EmbMethod, EmbSMethod, SentinelMethod],
};

pub const EmptySlot: PyModuleDef_Slot = PyModuleDef_Slot {
    slot: 0,
    value: std::ptr::null_mut(),
};

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
    std::mem::forget(lib);
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
    std::mem::forget(lib);
    ret
}

fn get_lib() -> Result<PythonLib, String> {
    let output = Command::new("python3.9-config")
        .arg("--configdir")
        .output()
        .map_err(|err| format!("failed to execute python3.9-config: {}", err))?;
    let mut path: Vec<u8> = output.stdout;
    while path
        .last()
        .map(|&c| c.is_ascii_whitespace())
        .unwrap_or(false)
    {
        path.pop();
    }
    path.extend_from_slice(b"/libpython3.9.so\0");
    unsafe {
        Ok(bindings::PythonLib::new(OsStr::from_bytes(&path))
            .map_err(|err| format!("failed to load libpython: {}", err))?)
    }
}
fn main() {
    /* Return the number of arguments of the application command line */
    //let lparam = closure_pointer_pointer as usize;
    let lib = get_lib().unwrap();

    //unsafe { lib.Py_SetProgramName(b"python-demo\0".as_ptr() as _) };
    //let state = unsafe { lib.PyGILState_Ensure() };
    //let module_obj = unsafe { lib.PyModule_Create2(&mut EmbModule as *mut _, 3) };
    //unsafe { lib.PyState_AddModule(module_obj, &EmbModule as *const _ as *mut _) };
    //unsafe { lib.PyGILState_Release(state) };
    println!("PyImport_AppendInittab = {}", unsafe {
        lib.PyImport_AppendInittab(b"emb\0".as_ptr() as _, Some(PyInit_emb))
    });
    unsafe { lib.Py_SetProgramName(b"afdsfa\0".as_ptr() as _) };
    unsafe { lib.Py_Initialize() };
    let mut libmeli = std::fs::File::open("libmeliplugin.py").unwrap();
    let mut libmeli_cstr = vec![];
    libmeli.read_to_end(&mut libmeli_cstr).unwrap();
    let mut libmeli_cstr = std::ffi::CString::new(libmeli_cstr).unwrap();
    let mut libmelicode = unsafe {
        lib.Py_CompileStringExFlags(
            libmeli_cstr.as_ptr() as _,
            b"libmeliplugin.py\0".as_ptr() as _,
            257,                  // Py_file_input
            std::ptr::null_mut(), //compiler_flags,
            0,
        )
    };
    //std::dbg!(&libmelicode);
    let pluginmodule = unsafe {
        lib.PyImport_ExecCodeModuleEx(
            b"libmeliplugin\0".as_ptr() as _,
            libmelicode,
            b"libmeliplugin.py\0".as_ptr() as _,
        )
    };
    if pluginmodule.is_null() {
        unsafe { lib.PyErr_Print() };
        return;
    }
    std::dbg!(&pluginmodule);
    //unsafe { lib.PyImport_ImportModule(b"emb\0".as_ptr() as _) };

    let ret = unsafe {
        lib.PyRun_SimpleString(
            //b"from time import time,ctime\nprint('Today is', ctime(time()))\n\0".as_ptr() as _,
            b"import emb\nimport libmeliplugin\nprint(dir(libmeliplugin))\nprint(\"Number of arguments\", emb.s('Im a python string'))\n\0".as_ptr()
                as _,
        )
    };
    println!("ret: {:?}", ret);
    //let ret = unsafe { lib.Py_FinalizeEx() };
    println!("2ret: {:?}", ret);

    let version = unsafe { std::ffi::CStr::from_ptr(lib.Py_GetVersion()) };
    println!("Hello, world! {:?}", version);
}

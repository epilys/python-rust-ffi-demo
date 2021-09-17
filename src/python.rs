/*
 * python-rust-ffi-demo
 *
 * Copyright 2021 python-rust-ffi-demo - Manos Pitsidianakis
 *
 * This file is part of python-rust-ffi-demo.
 *
 * python-rust-ffi-demo is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * python-rust-ffi-demo is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with python-rust-ffi-demo. If not, see <http://www.gnu.org/licenses/>.
 */

use std::ffi::OsStr;
use std::ffi::{CStr, CString};
use std::io::Read;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::process::Command;

use crate::bindings::*;

pub type Error = Box<dyn std::error::Error>;

pub const METH_VARARGS: i32 = 0x0001;

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

pub const SentinelMethod: PyMethodDef = PyMethodDef {
    ml_name: std::ptr::null(),
    ml_meth: None,
    ml_flags: 0,
    ml_doc: std::ptr::null(),
};

#[repr(C)]
pub struct Methods<const N: usize> {
    methods: [PyMethodDef; N],
    sentinel: PyMethodDef,
}

impl<const N: usize> Methods<N> {
    pub const fn new(methods: [PyMethodDef; N]) -> Self {
        Self {
            methods,
            sentinel: SentinelMethod,
        }
    }
}

pub const EmptySlot: PyModuleDef_Slot = PyModuleDef_Slot {
    slot: 0,
    value: std::ptr::null_mut(),
};

pub fn get_lib() -> Result<PythonLib, String> {
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
        Ok(PythonLib::new(OsStr::from_bytes(&path))
            .map_err(|err| format!("failed to load libpython: {}", err))?)
    }
}

pub fn file_to_cstring(path: &Path) -> Result<CString, Error> {
    let mut f = std::fs::File::open(path)?;
    let mut vec = vec![];
    f.read_to_end(&mut vec)?;
    Ok(CString::new(vec)?)
}

pub struct PythonBuilder {
    lib: PythonLib,
    program_name: CString,
    modules: Vec<CString>,
}

impl PythonBuilder {
    pub fn new() -> Result<Self, Error> {
        Ok(PythonBuilder {
            lib: get_lib()?,
            program_name: CString::new("")?,
            modules: vec![],
        })
    }

    pub fn with_program_name(mut self, name: &str) -> Result<Self, Error> {
        let cname = CString::new(name)?;
        unsafe { self.lib.Py_SetProgramName(cname.as_ptr() as _) };
        self.program_name = cname;
        Ok(self)
    }

    pub fn with_module(
        mut self,
        module_name: &str,
        module_init: unsafe extern "C" fn() -> *mut PyObject,
    ) -> Result<Self, Error> {
        let cname = CString::new(module_name)?;
        std::dbg!(&cname);
        let ret = unsafe {
            self.lib
                .PyImport_AppendInittab(cname.as_ptr() as _, Some(module_init))
        };
        self.modules.push(cname);
        std::dbg!(ret);
        Ok(self)
    }

    pub fn build(self) -> Result<Python, Error> {
        unsafe { self.lib.Py_Initialize() };
        Ok(Python {
            lib: self.lib,
            program_name: self.program_name,
            modules: self.modules,
            fmodules: vec![],
        })
    }
}

pub struct Python {
    lib: PythonLib,
    program_name: CString,
    modules: Vec<CString>,
    fmodules: Vec<(CString, CString)>,
}

pub fn get_py_err(lib: &PythonLib) -> Option<String> {
    let mut ptype: *mut PyObject = std::ptr::null_mut();
    let mut pvalue: *mut PyObject = std::ptr::null_mut();
    let mut ptraceback: *mut PyObject = std::ptr::null_mut();
    unsafe {
        lib.PyErr_Fetch(
            &mut ptype as *mut *mut PyObject,
            &mut pvalue as *mut *mut PyObject,
            &mut ptraceback as *mut *mut PyObject,
        )
    };
    if pvalue.is_null() {
        return None;
    }
    Some(unsafe {
        CStr::from_ptr(lib.PyUnicode_AsUTF8(pvalue))
            .to_string_lossy()
            .to_string()
    })
}

impl Python {
    pub fn load_module_from_file<P: AsRef<Path>>(
        &mut self,
        module_name: &str,
        module_path: P,
    ) -> Result<(), Error> {
        let path = module_path.as_ref();
        let mname = CString::new(module_name)?;
        let file_name = CString::new(path.file_name().ok_or_else(|| format!("load_module_from_file(): could not extract file name from {}: is it not a file path?", path.display()))?.as_bytes())?;
        let mut code_cstr = file_to_cstring(path)?;
        let mut code_obj = unsafe {
            self.lib.Py_CompileStringExFlags(
                code_cstr.as_ptr() as _,
                file_name.as_ptr() as _,
                257,                  // Py_file_input
                std::ptr::null_mut(), //compiler_flags,
                0,
            )
        };
        //std::dbg!(&libmelicode);
        let pluginmodule = unsafe {
            self.lib.PyImport_ExecCodeModuleEx(
                mname.as_ptr() as _,
                code_obj,
                file_name.as_ptr() as _,
            )
        };
        self.fmodules.push((mname, file_name));
        if pluginmodule.is_null() {
            //unsafe { self.lib.PyErr_Print() };
            Err(get_py_err(&self.lib).ok_or_else(|| "load_module_from_file(): Could not import module, PyImport_ExecCodeModuleEx returned null")?)?
        }
        Ok(())
    }

    pub fn version(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.lib.Py_GetVersion()) }
    }

    pub fn run_code(&mut self, code: &CStr) -> Result<(), Error> {
        let ret = unsafe { self.lib.PyRun_SimpleString(code.as_ptr() as _) };
        if ret != 0 {
            if let Some(err) = get_py_err(&self.lib) {
                Err(err)?;
            }
        }
        Ok(())
    }
}

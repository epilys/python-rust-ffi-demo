src/bindgen.rs: /usr/include/python3.9/Python.h
	bindgen /usr/include/python3.9/Python.h -o src/bindings.rs --dynamic-loading "PythonLib" --allowlist-function 'Py.*' --allowlist-function "_Py.*" --allowlist-type 'Py.*' --allowlist-type "_Py.*" -- -I/usr/include/python3.9/

all: src/bindgen.rs
	cargo build

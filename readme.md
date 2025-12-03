# DLPack Rust FFI binding

This is a minimal FFI binding to [DLPack](https://github.com/dmlc/dlpack). DLPack is an open in-memory tensor structure for sharing tensors among frameworks.

Current FFI version is DLPack [v1.2](https://github.com/dmlc/dlpack/releases/tag/v1.2). Version of this crate will follow the DLPack version. We do not add any safe-wrappers or abstractions on top of the FFI layer.

This crate is not official bindgen project. This project will only serve as a minimal binding layer for users who want to build their own safe abstractions on top of DLPack FFI. We will not add anything other than bindgen automatically generated code.

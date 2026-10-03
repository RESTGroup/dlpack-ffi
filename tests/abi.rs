//! ABI layout checks for the DLPack bindings.
//!
//! Expected sizes follow the DLPack v1.3 header (`header/dlpack.h`) on
//! 64-bit targets; pointer-width-dependent checks are skipped elsewhere.

use core::ffi::c_void;
use core::mem::{align_of, size_of};

use dlpack_ffi::*;

#[test]
fn struct_layout_64bit() {
    if size_of::<*mut c_void>() != 8 {
        return;
    }
    assert_eq!(size_of::<DLPackVersion>(), 8);
    assert_eq!(size_of::<DLDevice>(), 8);
    assert_eq!(size_of::<DLDataType>(), 4);
    assert_eq!(size_of::<DLTensor>(), 48);
    assert_eq!(size_of::<DLManagedTensor>(), 64);
    assert_eq!(size_of::<DLManagedTensorVersioned>(), 80);
}

#[test]
fn newtype_enums_are_transparent_integers() {
    assert_eq!(size_of::<DLDeviceType>(), 4);
    assert_eq!(align_of::<DLDeviceType>(), 4);
    assert_eq!(size_of::<DLDataTypeCode>(), 4);
    assert_eq!(align_of::<DLDataTypeCode>(), 4);
}

#[test]
fn version_constants_and_known_values() {
    assert_eq!(DLPACK_MAJOR_VERSION, 1);
    assert_eq!(DLPACK_MINOR_VERSION, 3);
    assert_eq!(DLDeviceType::kDLCPU, DLDeviceType(1));
    assert_eq!(DLDeviceType::kDLCUDA, DLDeviceType(2));
    assert_eq!(DLDeviceType::kDLTrn, DLDeviceType(18));
    assert_eq!(DLDataTypeCode::kDLInt, DLDataTypeCode(0));
    assert_eq!(DLDataTypeCode::kDLFloat, DLDataTypeCode(2));
    assert_eq!(DLDataTypeCode::kDLBool, DLDataTypeCode(6));
    assert_eq!(DLDataTypeCode::kDLFloat4_e2m1fn, DLDataTypeCode(17));
}

#[test]
fn unknown_values_are_representable() {
    // Values from newer DLPack minor releases (or vendor extensions) must
    // flow through the raw binding without invalid-value UB.
    let future_device = DLDeviceType(9999);
    let device = DLDevice { device_type: future_device, device_id: 0 };
    assert_eq!(device.device_type.0, 9999);

    // `DLDataType.code` is a plain `uint8_t` in the header by design.
    let dtype = DLDataType { code: 200, bits: 8, lanes: 1 };
    assert_eq!(dtype.code, 200);
}

#[test]
fn enum_derives_preserved() {
    // Guards against a bindgen upgrade silently dropping derives from the
    // newtype enums - a breaking API change no other gate would catch.
    fn assert_derives<T: core::fmt::Debug + Copy + Clone + core::hash::Hash + Eq>() {}
    assert_derives::<DLDeviceType>();
    assert_derives::<DLDataTypeCode>();
}

//! Round-trip checks for the DLPack ownership protocol.
//!
//! A managed tensor carrying a Rust-owned buffer is consumed through its
//! `deleter`, which must be invoked exactly once and free the buffer.

use core::ffi::c_void;
use std::sync::atomic::{AtomicUsize, Ordering};

use dlpack_ffi::*;

static LEGACY_DELETER_CALLS: AtomicUsize = AtomicUsize::new(0);
static VERSIONED_DELETER_CALLS: AtomicUsize = AtomicUsize::new(0);

/// Drops the `Vec<f32>` stored in `manager_ctx` of a legacy managed tensor.
unsafe extern "C" fn deleter_legacy(self_: *mut DLManagedTensor) {
    LEGACY_DELETER_CALLS.fetch_add(1, Ordering::SeqCst);
    let tensor = unsafe { &mut *self_ };
    drop(unsafe { Box::from_raw(tensor.manager_ctx as *mut Vec<f32>) });
}

/// Drops the `Vec<i32>` stored in `manager_ctx` of a versioned managed tensor.
unsafe extern "C" fn deleter_versioned(self_: *mut DLManagedTensorVersioned) {
    VERSIONED_DELETER_CALLS.fetch_add(1, Ordering::SeqCst);
    let tensor = unsafe { &mut *self_ };
    drop(unsafe { Box::from_raw(tensor.manager_ctx as *mut Vec<i32>) });
}

#[test]
fn legacy_managed_tensor_roundtrip() {
    LEGACY_DELETER_CALLS.store(0, Ordering::SeqCst);

    let mut data = vec![1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0];
    let shape = [2i64, 3];
    let strides = [3i64, 1];

    let mut managed = DLManagedTensor {
        dl_tensor: DLTensor {
            data: data.as_mut_ptr() as *mut c_void,
            device: DLDevice { device_type: DLDeviceType::kDLCPU, device_id: 0 },
            ndim: 2,
            dtype: DLDataType { code: (DLDataTypeCode::kDLFloat).0 as u8, bits: 32, lanes: 1 },
            shape: shape.as_ptr() as *mut i64,
            strides: strides.as_ptr() as *mut i64,
            byte_offset: 0,
        },
        manager_ctx: Box::into_raw(Box::new(data)) as *mut c_void,
        deleter: Some(deleter_legacy),
    };

    // Consumer side: reading through the DLTensor.
    let ptr = managed.dl_tensor.data as *const f32;
    let values = unsafe { (0..6).map(|i| *ptr.add(i)).collect::<Vec<f32>>() };
    assert_eq!(values, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    assert_eq!(unsafe { *managed.dl_tensor.shape.add(1) }, 3);
    assert_eq!(unsafe { *managed.dl_tensor.strides.add(0) }, 3);

    // Release exactly once.
    let deleter = managed.deleter.expect("deleter must be set");
    unsafe { deleter(&mut managed) };
    assert_eq!(LEGACY_DELETER_CALLS.load(Ordering::SeqCst), 1);
}

#[test]
fn versioned_managed_tensor_roundtrip() {
    VERSIONED_DELETER_CALLS.store(0, Ordering::SeqCst);

    let mut data = vec![10i32, 20, 30, 40];
    let data_ptr = data.as_mut_ptr();
    let shape = [4i64];
    let strides = [1i64];

    let mut managed = DLManagedTensorVersioned {
        version: DLPackVersion { major: DLPACK_MAJOR_VERSION, minor: DLPACK_MINOR_VERSION },
        manager_ctx: Box::into_raw(Box::new(data)) as *mut c_void,
        deleter: Some(deleter_versioned),
        flags: DLPACK_FLAG_BITMASK_READ_ONLY as u64,
        dl_tensor: DLTensor {
            data: data_ptr as *mut c_void,
            device: DLDevice { device_type: DLDeviceType::kDLCPU, device_id: 0 },
            ndim: 1,
            dtype: DLDataType { code: (DLDataTypeCode::kDLInt).0 as u8, bits: 32, lanes: 1 },
            shape: shape.as_ptr() as *mut i64,
            strides: strides.as_ptr() as *mut i64,
            byte_offset: 0,
        },
    };

    assert_eq!(managed.version.major, DLPACK_MAJOR_VERSION);
    assert_eq!(managed.version.minor, DLPACK_MINOR_VERSION);
    assert_eq!(
        managed.flags & (DLPACK_FLAG_BITMASK_READ_ONLY as u64),
        DLPACK_FLAG_BITMASK_READ_ONLY as u64
    );

    let ptr = managed.dl_tensor.data as *const i32;
    let values = unsafe { (0..4).map(|i| *ptr.add(i)).collect::<Vec<i32>>() };
    assert_eq!(values, vec![10, 20, 30, 40]);

    // Release exactly once. (On a major-version mismatch the protocol
    // requires a consumer to call the deleter and read nothing else.)
    let deleter = managed.deleter.expect("deleter must be set");
    unsafe { deleter(&mut managed) };
    assert_eq!(VERSIONED_DELETER_CALLS.load(Ordering::SeqCst), 1);
}

use std::ffi::{c_char, CString};
use std::ptr;

use temporal_rs::sys::Temporal;
use timezone_provider::tzif::CompiledTzdbProvider;

/// Returns the current instant as an ISO 8601 string (e.g., "2024-01-15T10:30:45.123Z").
/// The caller is responsible for freeing the returned string using `temporal_free_string`.
///
/// Returns NULL on error.
#[no_mangle]
pub extern "C" fn temporal_instant_now() -> *mut c_char {
    match get_instant_now_string() {
        Ok(s) => match CString::new(s) {
            Ok(c_str) => c_str.into_raw(),
            Err(_) => ptr::null_mut(),
        },
        Err(_) => ptr::null_mut(),
    }
}

/// Frees a string allocated by temporal functions.
///
/// # Safety
/// The pointer must have been allocated by a temporal function (e.g., `temporal_instant_now`).
#[no_mangle]
pub unsafe extern "C" fn temporal_free_string(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}

fn get_instant_now_string() -> Result<String, Box<dyn std::error::Error>> {
    let now = Temporal::utc_now();
    let instant = now.instant()?;
    let provider = CompiledTzdbProvider::default();
    let iso_string = instant.to_ixdtf_string_with_provider(None, Default::default(), &provider)?;
    Ok(iso_string)
}

// ============================================================================
// Android JNI bindings
// ============================================================================

#[cfg(target_os = "android")]
mod android {
    use jni::objects::JClass;
    use jni::sys::jstring;
    use jni::JNIEnv;

    use super::get_instant_now_string;

    /// JNI function for `com.temporal.TemporalNative.instantNow()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_instantNow(
        env: JNIEnv,
        _class: JClass,
    ) -> jstring {
        match get_instant_now_string() {
            Ok(s) => env
                .new_string(s)
                .map(|js| js.into_raw())
                .unwrap_or(ptr::null_mut()),
            Err(_) => ptr::null_mut(),
        }
    }

    use std::ptr;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instant_now() {
        let result = get_instant_now_string().unwrap();
        // Should be in ISO 8601 format like "2024-01-15T10:30:45.123456789Z"
        assert!(result.ends_with('Z'), "Expected UTC timestamp: {}", result);
        assert!(result.contains('T'), "Expected ISO format: {}", result);
        println!("Current instant: {}", result);
    }
}

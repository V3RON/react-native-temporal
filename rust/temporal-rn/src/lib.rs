use std::ffi::{c_char, CString};
use std::ptr;
use std::str::FromStr;

use temporal_rs::sys::Temporal;
use temporal_rs::{
    options::{DisplayCalendar, ToStringRoundingOptions},
    Calendar, Duration, Instant, PlainDate, PlainDateTime, PlainTime, TimeZone, ZonedDateTime,
};
use timezone_provider::tzif::CompiledTzdbProvider;

// ============================================================================
// Error Types (matching TC39 Temporal)
// ============================================================================

/// Error types matching TC39 Temporal specification
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TemporalErrorType {
    /// No error
    None = 0,
    /// RangeError - value out of range or invalid format
    RangeError = 1,
    /// TypeError - wrong type or invalid argument
    TypeError = 2,
}

/// Result structure for FFI operations that can fail
#[repr(C)]
pub struct TemporalResult {
    /// The result value (NULL if error)
    pub value: *mut c_char,
    /// Error type (0 = success)
    pub error_type: i32,
    /// Error message (NULL if success). Caller must free with temporal_free_string.
    pub error_message: *mut c_char,
}

impl TemporalResult {
    fn success(value: String) -> Self {
        match CString::new(value) {
            Ok(c_str) => Self {
                value: c_str.into_raw(),
                error_type: TemporalErrorType::None as i32,
                error_message: ptr::null_mut(),
            },
            Err(_) => Self::type_error("Failed to convert result to C string"),
        }
    }

    fn range_error(message: &str) -> Self {
        let error_msg = CString::new(message)
            .map(|s| s.into_raw())
            .unwrap_or(ptr::null_mut());
        Self {
            value: ptr::null_mut(),
            error_type: TemporalErrorType::RangeError as i32,
            error_message: error_msg,
        }
    }

    fn type_error(message: &str) -> Self {
        let error_msg = CString::new(message)
            .map(|s| s.into_raw())
            .unwrap_or(ptr::null_mut());
        Self {
            value: ptr::null_mut(),
            error_type: TemporalErrorType::TypeError as i32,
            error_message: error_msg,
        }
    }
}

/// Frees a TemporalResult's allocated strings.
/// 
/// # Safety
/// The result must have been returned by a temporal function.
#[no_mangle]
pub unsafe extern "C" fn temporal_free_result(result: *mut TemporalResult) {
    if result.is_null() {
        return;
    }
    let r = &mut *result;
    if !r.value.is_null() {
        drop(CString::from_raw(r.value));
        r.value = ptr::null_mut();
    }
    if !r.error_message.is_null() {
        drop(CString::from_raw(r.error_message));
        r.error_message = ptr::null_mut();
    }
}

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
// Instant API (Expanded)
// ============================================================================

/// Parses an ISO 8601 string into an Instant and returns the normalized string.
#[no_mangle]
pub extern "C" fn temporal_instant_from_string(s: *const c_char) -> TemporalResult {
    let s_str = match parse_c_str(s, "instant string") {
        Ok(s) => s,
        Err(e) => return e,
    };
    match Instant::from_str(s_str) {
        Ok(instant) => {
            let provider = CompiledTzdbProvider::default();
            match instant.to_ixdtf_string_with_provider(None, Default::default(), &provider) {
                Ok(s) => TemporalResult::success(s),
                Err(e) => TemporalResult::range_error(&format!("Failed to format instant: {}", e)),
            }
        },
        Err(e) => TemporalResult::range_error(&format!("Invalid instant '{}': {}", s_str, e)),
    }
}

/// Creates an Instant from epoch milliseconds.
#[no_mangle]
pub extern "C" fn temporal_instant_from_epoch_milliseconds(ms: i64) -> TemporalResult {
    // Instant::from_epoch_milliseconds is the likely API, or we construct via ns
    // Using i128 arithmetic to be safe: ms * 1,000,000
    let ns = (ms as i128).saturating_mul(1_000_000);
    match Instant::try_new(ns) {
        Ok(instant) => {
            let provider = CompiledTzdbProvider::default();
            match instant.to_ixdtf_string_with_provider(None, Default::default(), &provider) {
                Ok(s) => TemporalResult::success(s),
                Err(e) => TemporalResult::range_error(&format!("Failed to format instant: {}", e)),
            }
        },
        Err(e) => TemporalResult::range_error(&format!("Invalid epoch milliseconds: {}", e)),
    }
}

/// Creates an Instant from epoch nanoseconds (string input for i128 precision).
#[no_mangle]
pub extern "C" fn temporal_instant_from_epoch_nanoseconds(ns_str: *const c_char) -> TemporalResult {
    let s_str = match parse_c_str(ns_str, "nanoseconds string") {
        Ok(s) => s,
        Err(e) => return e,
    };
    
    let ns = match i128::from_str(s_str) {
        Ok(n) => n,
        Err(_) => return TemporalResult::range_error("Invalid nanoseconds string"),
    };

    match Instant::try_new(ns) {
        Ok(instant) => {
            let provider = CompiledTzdbProvider::default();
            match instant.to_ixdtf_string_with_provider(None, Default::default(), &provider) {
                Ok(s) => TemporalResult::success(s),
                Err(e) => TemporalResult::range_error(&format!("Failed to format instant: {}", e)),
            }
        },
        Err(e) => TemporalResult::range_error(&format!("Invalid epoch nanoseconds: {}", e)),
    }
}

/// Returns the epoch milliseconds of an Instant.
#[no_mangle]
pub extern "C" fn temporal_instant_epoch_milliseconds(s: *const c_char) -> TemporalResult {
    let instant = match parse_instant(s, "instant") {
        Ok(i) => i,
        Err(e) => return e,
    };
    // Format as string to return via TemporalResult (which expects char*)
    // Alternatively we could change return type, but keeping uniform interface is good.
    // However, JS side expects a number.
    // For now, let's return string and parse in JS/Native layer?
    // Actually, getting a primitive value out might be better done with a specific function returning double/int64.
    // But TemporalResult standardizes error handling.
    // I'll return string for consistency and parse in Kotlin/ObjC/JS.
    let ms = instant.epoch_milliseconds();
    TemporalResult::success(ms.to_string())
}

/// Returns the epoch nanoseconds of an Instant (as string).
#[no_mangle]
pub extern "C" fn temporal_instant_epoch_nanoseconds(s: *const c_char) -> TemporalResult {
    let instant = match parse_instant(s, "instant") {
        Ok(i) => i,
        Err(e) => return e,
    };
    let ns = instant.epoch_nanoseconds();
    TemporalResult::success(ns.0.to_string())
}

/// Adds a duration to an instant.
#[no_mangle]
pub extern "C" fn temporal_instant_add(instant_str: *const c_char, duration_str: *const c_char) -> TemporalResult {
    let instant = match parse_instant(instant_str, "instant") {
        Ok(i) => i,
        Err(e) => return e,
    };
    let duration = match parse_duration(duration_str, "duration") {
        Ok(d) => d,
        Err(e) => return e,
    };
    
    match instant.add(&duration) {
        Ok(result) => {
            let provider = CompiledTzdbProvider::default();
            match result.to_ixdtf_string_with_provider(None, Default::default(), &provider) {
                Ok(s) => TemporalResult::success(s),
                Err(e) => TemporalResult::range_error(&format!("Failed to format instant: {}", e)),
            }
        },
        Err(e) => TemporalResult::range_error(&format!("Failed to add duration: {}", e)),
    }
}

/// Subtracts a duration from an instant.
#[no_mangle]
pub extern "C" fn temporal_instant_subtract(instant_str: *const c_char, duration_str: *const c_char) -> TemporalResult {
    let instant = match parse_instant(instant_str, "instant") {
        Ok(i) => i,
        Err(e) => return e,
    };
    let duration = match parse_duration(duration_str, "duration") {
        Ok(d) => d,
        Err(e) => return e,
    };
    
    match instant.subtract(&duration) {
        Ok(result) => {
            let provider = CompiledTzdbProvider::default();
            match result.to_ixdtf_string_with_provider(None, Default::default(), &provider) {
                Ok(s) => TemporalResult::success(s),
                Err(e) => TemporalResult::range_error(&format!("Failed to format instant: {}", e)),
            }
        },
        Err(e) => TemporalResult::range_error(&format!("Failed to subtract duration: {}", e)),
    }
}

/// Compares two instants.
#[no_mangle]
pub extern "C" fn temporal_instant_compare(a: *const c_char, b: *const c_char) -> CompareResult {
    let instant_a = match parse_instant(a, "first instant") {
        Ok(i) => i,
        Err(e) => return CompareResult::range_error(
            &unsafe { std::ffi::CStr::from_ptr(e.error_message) }.to_string_lossy()
        ),
    };
    let instant_b = match parse_instant(b, "second instant") {
        Ok(i) => i,
        Err(e) => return CompareResult::range_error(
            &unsafe { std::ffi::CStr::from_ptr(e.error_message) }.to_string_lossy()
        ),
    };
    
    CompareResult::success(instant_a.cmp(&instant_b) as i32)
}

// ============================================================================
// Now API
// ============================================================================

#[no_mangle]
pub extern "C" fn temporal_now_plain_date_time_iso(tz_id: *const c_char) -> TemporalResult {
    let tz_str = match parse_c_str(tz_id, "timezone id") {
        Ok(s) => s,
        Err(e) => return e,
    };
    
    match get_now_plain_date_time_string(tz_str) {
        Ok(s) => TemporalResult::success(s),
        Err(e) => TemporalResult::range_error(&format!("Failed to get plain date time: {}", e)),
    }
}

#[no_mangle]
pub extern "C" fn temporal_now_plain_date_iso(tz_id: *const c_char) -> TemporalResult {
    let tz_str = match parse_c_str(tz_id, "timezone id") {
        Ok(s) => s,
        Err(e) => return e,
    };
    
    match get_now_plain_date_string(tz_str) {
        Ok(s) => TemporalResult::success(s),
        Err(e) => TemporalResult::range_error(&format!("Failed to get plain date: {}", e)),
    }
}

#[no_mangle]
pub extern "C" fn temporal_now_plain_time_iso(tz_id: *const c_char) -> TemporalResult {
    let tz_str = match parse_c_str(tz_id, "timezone id") {
        Ok(s) => s,
        Err(e) => return e,
    };
    
    match get_now_plain_time_string(tz_str) {
        Ok(s) => TemporalResult::success(s),
        Err(e) => TemporalResult::range_error(&format!("Failed to get plain time: {}", e)),
    }
}

fn get_now_plain_date_time_string(tz_id: &str) -> Result<String, Box<dyn std::error::Error>> {
    let now = Temporal::utc_now();
    let instant = now.instant()?;
    let time_zone = TimeZone::try_from_str(tz_id)?;
    let zdt = instant.to_zoned_date_time_iso(time_zone)?;
    Ok(zdt
        .to_plain_date_time()
        .to_ixdtf_string(ToStringRoundingOptions::default(), DisplayCalendar::Auto)?)
}

fn get_now_plain_date_string(tz_id: &str) -> Result<String, Box<dyn std::error::Error>> {
    let now = Temporal::utc_now();
    let instant = now.instant()?;
    let time_zone = TimeZone::try_from_str(tz_id)?;
    let zdt = instant.to_zoned_date_time_iso(time_zone)?;
    Ok(zdt.to_plain_date().to_ixdtf_string(DisplayCalendar::Auto))
}

fn get_now_plain_time_string(tz_id: &str) -> Result<String, Box<dyn std::error::Error>> {
    let now = Temporal::utc_now();
    let instant = now.instant()?;
    let time_zone = TimeZone::try_from_str(tz_id)?;
    let zdt = instant.to_zoned_date_time_iso(time_zone)?;
    Ok(zdt
        .to_plain_time()
        .to_ixdtf_string(ToStringRoundingOptions::default())?)
}

// ============================================================================
// PlainTime API
// ============================================================================

/// Represents a PlainTime's component values for FFI.
#[repr(C)]
pub struct PlainTimeComponents {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub millisecond: u16,
    pub microsecond: u16,
    pub nanosecond: u16,
    /// 1 if the components are valid, 0 if parsing failed
    pub is_valid: i8,
}

impl Default for PlainTimeComponents {
    fn default() -> Self {
        Self {
            hour: 0,
            minute: 0,
            second: 0,
            millisecond: 0,
            microsecond: 0,
            nanosecond: 0,
            is_valid: 0,
        }
    }
}

/// Parses an ISO 8601 string into a PlainTime and returns the normalized string.
#[no_mangle]
pub extern "C" fn temporal_plain_time_from_string(s: *const c_char) -> TemporalResult {
    let s_str = match parse_c_str(s, "plain time string") {
        Ok(s) => s,
        Err(e) => return e,
    };
    match PlainTime::from_str(s_str) {
        Ok(time) => match time.to_ixdtf_string(ToStringRoundingOptions::default()) {
            Ok(s) => TemporalResult::success(s),
            Err(e) => TemporalResult::range_error(&format!("Failed to format plain time: {}", e)),
        },
        Err(e) => TemporalResult::range_error(&format!("Invalid plain time '{}': {}", s_str, e)),
    }
}

/// Creates a PlainTime from individual components.
/// Validates ranges: hour (0-23), minute (0-59), second (0-59), 
/// millisecond/microsecond/nanosecond (0-999).
#[no_mangle]
pub extern "C" fn temporal_plain_time_from_components(
    hour: u8,
    minute: u8,
    second: u8,
    millisecond: u16,
    microsecond: u16,
    nanosecond: u16,
) -> TemporalResult {
    // Validate ranges
    if hour > 23 {
        return TemporalResult::range_error(&format!("Invalid hour: {} (must be 0-23)", hour));
    }
    if minute > 59 {
        return TemporalResult::range_error(&format!("Invalid minute: {} (must be 0-59)", minute));
    }
    if second > 59 {
        return TemporalResult::range_error(&format!("Invalid second: {} (must be 0-59)", second));
    }
    if millisecond > 999 {
        return TemporalResult::range_error(&format!("Invalid millisecond: {} (must be 0-999)", millisecond));
    }
    if microsecond > 999 {
        return TemporalResult::range_error(&format!("Invalid microsecond: {} (must be 0-999)", microsecond));
    }
    if nanosecond > 999 {
        return TemporalResult::range_error(&format!("Invalid nanosecond: {} (must be 0-999)", nanosecond));
    }

    match PlainTime::new(hour, minute, second, millisecond, microsecond, nanosecond) {
        Ok(time) => match time.to_ixdtf_string(ToStringRoundingOptions::default()) {
            Ok(s) => TemporalResult::success(s),
            Err(e) => TemporalResult::range_error(&format!("Failed to format plain time: {}", e)),
        },
        Err(e) => TemporalResult::range_error(&format!("Invalid plain time components: {}", e)),
    }
}

/// Gets all component values from a PlainTime string.
#[no_mangle]
pub extern "C" fn temporal_plain_time_get_components(
    s: *const c_char,
    out: *mut PlainTimeComponents,
) {
    if out.is_null() {
        return;
    }

    unsafe { *out = PlainTimeComponents::default(); }

    if s.is_null() {
        return;
    }

    let time = match parse_plain_time(s, "plain time") {
        Ok(t) => t,
        Err(_) => return,
    };

    unsafe {
        (*out).hour = time.hour();
        (*out).minute = time.minute();
        (*out).second = time.second();
        (*out).millisecond = time.millisecond();
        (*out).microsecond = time.microsecond();
        (*out).nanosecond = time.nanosecond();
        (*out).is_valid = 1;
    }
}

/// Adds a duration to a PlainTime.
#[no_mangle]
pub extern "C" fn temporal_plain_time_add(time_str: *const c_char, duration_str: *const c_char) -> TemporalResult {
    let time = match parse_plain_time(time_str, "plain time") {
        Ok(t) => t,
        Err(e) => return e,
    };
    let duration = match parse_duration(duration_str, "duration") {
        Ok(d) => d,
        Err(e) => return e,
    };

    match time.add(&duration) {
        Ok(result) => match result.to_ixdtf_string(ToStringRoundingOptions::default()) {
            Ok(s) => TemporalResult::success(s),
            Err(e) => TemporalResult::range_error(&format!("Failed to format plain time: {}", e)),
        },
        Err(e) => TemporalResult::range_error(&format!("Failed to add duration: {}", e)),
    }
}

/// Subtracts a duration from a PlainTime.
#[no_mangle]
pub extern "C" fn temporal_plain_time_subtract(time_str: *const c_char, duration_str: *const c_char) -> TemporalResult {
    let time = match parse_plain_time(time_str, "plain time") {
        Ok(t) => t,
        Err(e) => return e,
    };
    let duration = match parse_duration(duration_str, "duration") {
        Ok(d) => d,
        Err(e) => return e,
    };

    match time.subtract(&duration) {
        Ok(result) => match result.to_ixdtf_string(ToStringRoundingOptions::default()) {
            Ok(s) => TemporalResult::success(s),
            Err(e) => TemporalResult::range_error(&format!("Failed to format plain time: {}", e)),
        },
        Err(e) => TemporalResult::range_error(&format!("Failed to subtract duration: {}", e)),
    }
}

/// Compares two PlainTime objects.
#[no_mangle]
pub extern "C" fn temporal_plain_time_compare(a: *const c_char, b: *const c_char) -> CompareResult {
    let time_a = match parse_plain_time(a, "first plain time") {
        Ok(t) => t,
        Err(e) => return CompareResult::range_error(
            &unsafe { std::ffi::CStr::from_ptr(e.error_message) }.to_string_lossy()
        ),
    };
    let time_b = match parse_plain_time(b, "second plain time") {
        Ok(t) => t,
        Err(e) => return CompareResult::range_error(
            &unsafe { std::ffi::CStr::from_ptr(e.error_message) }.to_string_lossy()
        ),
    };

    CompareResult::success(time_a.cmp(&time_b) as i32)
}

// ============================================================================
// Calendar API
// ============================================================================

/// Gets a Calendar from a string identifier.
#[no_mangle]
pub extern "C" fn temporal_calendar_from(id: *const c_char) -> TemporalResult {
    let id_str = match parse_c_str(id, "calendar identifier") {
        Ok(s) => s,
        Err(e) => return e,
    };
    
    match Calendar::from_str(id_str) {
        Ok(calendar) => TemporalResult::success(calendar.identifier().to_string()),
        Err(e) => TemporalResult::range_error(&format!("Invalid calendar identifier '{}': {}", id_str, e)),
    }
}

/// Gets the identifier of a calendar.
#[no_mangle]
pub extern "C" fn temporal_calendar_id(id: *const c_char) -> TemporalResult {
    // This function essentially normalizes the calendar ID
    // If the input is already a valid ID, it returns it.
    let id_str = match parse_c_str(id, "calendar identifier") {
        Ok(s) => s,
        Err(e) => return e,
    };
    
    match Calendar::from_str(id_str) {
        Ok(calendar) => TemporalResult::success(calendar.identifier().to_string()),
        Err(e) => TemporalResult::range_error(&format!("Invalid calendar identifier '{}': {}", id_str, e)),
    }
}

// ============================================================================
// Duration API

// ============================================================================
/// Note: microseconds and nanoseconds are clamped to i64 range for FFI safety.
#[repr(C)]
pub struct DurationComponents {
    pub years: i64,
    pub months: i64,
    pub weeks: i64,
    pub days: i64,
    pub hours: i64,
    pub minutes: i64,
    pub seconds: i64,
    pub milliseconds: i64,
    pub microseconds: i64,
    pub nanoseconds: i64,
    /// Sign of the duration: -1, 0, or 1
    pub sign: i8,
    /// 1 if the components are valid, 0 if parsing failed
    pub is_valid: i8,
}

impl Default for DurationComponents {
    fn default() -> Self {
        Self {
            years: 0,
            months: 0,
            weeks: 0,
            days: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
            milliseconds: 0,
            microseconds: 0,
            nanoseconds: 0,
            sign: 0,
            is_valid: 0,
        }
    }
}

/// Parses an ISO 8601 duration string and returns a TemporalResult.
#[no_mangle]
pub extern "C" fn temporal_duration_from_string(s: *const c_char) -> TemporalResult {
    if s.is_null() {
        return TemporalResult::type_error("Duration string cannot be null");
    }

    let c_str = match unsafe { std::ffi::CStr::from_ptr(s) }.to_str() {
        Ok(s) => s,
        Err(_) => return TemporalResult::type_error("Invalid UTF-8 in duration string"),
    };

    match Duration::from_str(c_str) {
        Ok(duration) => TemporalResult::success(duration.to_string()),
        Err(e) => TemporalResult::range_error(&format!("Invalid duration '{}': {}", c_str, e)),
    }
}

/// Gets all component values from a duration string in a single call.
/// Sets out->is_valid to 1 on success, 0 on error.
#[no_mangle]
pub extern "C" fn temporal_duration_get_components(
    s: *const c_char,
    out: *mut DurationComponents,
) {
    if out.is_null() {
        return;
    }

    // Initialize to invalid state
    unsafe {
        *out = DurationComponents::default();
    }

    if s.is_null() {
        return;
    }

    let c_str = unsafe { std::ffi::CStr::from_ptr(s) };
    let duration_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return,
    };

    let duration = match Duration::from_str(duration_str) {
        Ok(d) => d,
        Err(_) => return,
    };

    unsafe {
        (*out).years = duration.years();
        (*out).months = duration.months();
        (*out).weeks = duration.weeks();
        (*out).days = duration.days();
        (*out).hours = duration.hours();
        (*out).minutes = duration.minutes();
        (*out).seconds = duration.seconds();
        (*out).milliseconds = duration.milliseconds();
        // Clamp i128 values to i64 range for FFI safety
        (*out).microseconds = duration.microseconds().clamp(i64::MIN as i128, i64::MAX as i128) as i64;
        (*out).nanoseconds = duration.nanoseconds().clamp(i64::MIN as i128, i64::MAX as i128) as i64;
        (*out).sign = duration.sign() as i8;
        (*out).is_valid = 1;
    }
}

/// Adds two durations and returns a TemporalResult.
#[no_mangle]
pub extern "C" fn temporal_duration_add(a: *const c_char, b: *const c_char) -> TemporalResult {
    duration_binary_op(a, b, "add", |d1, d2| d1.add(&d2))
}

/// Subtracts duration b from a and returns a TemporalResult.
#[no_mangle]
pub extern "C" fn temporal_duration_subtract(a: *const c_char, b: *const c_char) -> TemporalResult {
    duration_binary_op(a, b, "subtract", |d1, d2| d1.subtract(&d2))
}

/// Negates a duration and returns a TemporalResult.
#[no_mangle]
pub extern "C" fn temporal_duration_negated(s: *const c_char) -> TemporalResult {
    duration_unary_op(s, "negate", |d| Ok(d.negated()))
}

/// Gets the absolute value of a duration and returns a TemporalResult.
#[no_mangle]
pub extern "C" fn temporal_duration_abs(s: *const c_char) -> TemporalResult {
    duration_unary_op(s, "abs", |d| Ok(d.abs()))
}

/// Creates a duration from individual component values.
/// Returns a TemporalResult with the ISO string representation.
#[no_mangle]
pub extern "C" fn temporal_duration_from_components(
    years: i64,
    months: i64,
    weeks: i64,
    days: i64,
    hours: i64,
    minutes: i64,
    seconds: i64,
    milliseconds: i64,
    microseconds: i64,
    nanoseconds: i64,
) -> TemporalResult {
    // Check for mixed signs (TC39 requirement)
    let values = [years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds];
    let non_zero: Vec<i64> = values.iter().copied().filter(|&v| v != 0).collect();

    if !non_zero.is_empty() {
        let first_sign = non_zero[0].signum();
        if !non_zero.iter().all(|&v| v.signum() == first_sign) {
            return TemporalResult::range_error("All non-zero duration values must have the same sign");
        }
    }

    match Duration::new(
        years,
        months,
        weeks,
        days,
        hours,
        minutes,
        seconds,
        milliseconds,
        microseconds as i128,
        nanoseconds as i128,
    ) {
        Ok(duration) => TemporalResult::success(duration.to_string()),
        Err(e) => TemporalResult::range_error(&format!("Invalid duration components: {}", e)),
    }
}

/// Compares two durations. Returns -1, 0, or 1.
/// Note: Durations with calendar units (years, months, weeks) cannot be compared
/// without a relativeTo point, which is not yet supported.
/// For now, this only works reliably with time-only durations.
#[repr(C)]
pub struct CompareResult {
    /// -1, 0, or 1 for less than, equal, or greater than
    pub value: i32,
    /// Error type (0 = success)
    pub error_type: i32,
    /// Error message (NULL if success)
    pub error_message: *mut c_char,
}

impl CompareResult {
    fn success(value: i32) -> Self {
        Self {
            value,
            error_type: TemporalErrorType::None as i32,
            error_message: ptr::null_mut(),
        }
    }

    fn range_error(message: &str) -> Self {
        let error_msg = CString::new(message)
            .map(|s| s.into_raw())
            .unwrap_or(ptr::null_mut());
        Self {
            value: 0,
            error_type: TemporalErrorType::RangeError as i32,
            error_message: error_msg,
        }
    }

    fn type_error(message: &str) -> Self {
        let error_msg = CString::new(message)
            .map(|s| s.into_raw())
            .unwrap_or(ptr::null_mut());
        Self {
            value: 0,
            error_type: TemporalErrorType::TypeError as i32,
            error_message: error_msg,
        }
    }
}

/// Frees a CompareResult's allocated strings.
#[no_mangle]
pub unsafe extern "C" fn temporal_free_compare_result(result: *mut CompareResult) {
    if result.is_null() {
        return;
    }
    let r = &mut *result;
    if !r.error_message.is_null() {
        drop(CString::from_raw(r.error_message));
        r.error_message = ptr::null_mut();
    }
}

#[no_mangle]
pub extern "C" fn temporal_duration_compare(a: *const c_char, b: *const c_char) -> CompareResult {
    let duration_a = match parse_duration(a, "first duration") {
        Ok(d) => d,
        Err(e) => return CompareResult::range_error(
            &unsafe { std::ffi::CStr::from_ptr(e.error_message) }.to_string_lossy()
        ),
    };
    let duration_b = match parse_duration(b, "second duration") {
        Ok(d) => d,
        Err(e) => return CompareResult::range_error(
            &unsafe { std::ffi::CStr::from_ptr(e.error_message) }.to_string_lossy()
        ),
    };

    // Check if durations have calendar units (years, months, weeks)
    let has_calendar_a = duration_a.years() != 0 || duration_a.months() != 0 || duration_a.weeks() != 0;
    let has_calendar_b = duration_b.years() != 0 || duration_b.months() != 0 || duration_b.weeks() != 0;

    if has_calendar_a || has_calendar_b {
        return CompareResult::range_error(
            "Comparing durations with years, months, or weeks requires a relativeTo option (not yet supported)"
        );
    }

    // For time-only durations, we can compare by converting to total nanoseconds
    let total_a = duration_a.days() as i128 * 86_400_000_000_000
        + duration_a.hours() as i128 * 3_600_000_000_000
        + duration_a.minutes() as i128 * 60_000_000_000
        + duration_a.seconds() as i128 * 1_000_000_000
        + duration_a.milliseconds() as i128 * 1_000_000
        + duration_a.microseconds() * 1_000
        + duration_a.nanoseconds();

    let total_b = duration_b.days() as i128 * 86_400_000_000_000
        + duration_b.hours() as i128 * 3_600_000_000_000
        + duration_b.minutes() as i128 * 60_000_000_000
        + duration_b.seconds() as i128 * 1_000_000_000
        + duration_b.milliseconds() as i128 * 1_000_000
        + duration_b.microseconds() * 1_000
        + duration_b.nanoseconds();

    CompareResult::success(total_a.cmp(&total_b) as i32)
}

/// Sentinel value for "unchanged" component in durationWith.
/// Matches JavaScript's Number.MIN_SAFE_INTEGER (-(2^53 - 1)).
const UNCHANGED_SENTINEL: i64 = -9007199254740991;

/// Creates a new duration by replacing specified components.
/// Pass UNCHANGED_SENTINEL (-9007199254740991) for components that should not be changed.
#[no_mangle]
pub extern "C" fn temporal_duration_with(
    original: *const c_char,
    years: i64,
    months: i64,
    weeks: i64,
    days: i64,
    hours: i64,
    minutes: i64,
    seconds: i64,
    milliseconds: i64,
    microseconds: i64,
    nanoseconds: i64,
) -> TemporalResult {
    let duration = match parse_duration(original, "duration") {
        Ok(d) => d,
        Err(e) => return e,
    };

    // Use original values for any component set to UNCHANGED_SENTINEL (sentinel for "unchanged")
    let new_years = if years == UNCHANGED_SENTINEL { duration.years() } else { years };
    let new_months = if months == UNCHANGED_SENTINEL { duration.months() } else { months };
    let new_weeks = if weeks == UNCHANGED_SENTINEL { duration.weeks() } else { weeks };
    let new_days = if days == UNCHANGED_SENTINEL { duration.days() } else { days };
    let new_hours = if hours == UNCHANGED_SENTINEL { duration.hours() } else { hours };
    let new_minutes = if minutes == UNCHANGED_SENTINEL { duration.minutes() } else { minutes };
    let new_seconds = if seconds == UNCHANGED_SENTINEL { duration.seconds() } else { seconds };
    let new_milliseconds = if milliseconds == UNCHANGED_SENTINEL { duration.milliseconds() } else { milliseconds };
    let new_microseconds = if microseconds == UNCHANGED_SENTINEL {
        duration.microseconds().clamp(i64::MIN as i128, i64::MAX as i128) as i64
    } else {
        microseconds
    };
    let new_nanoseconds = if nanoseconds == UNCHANGED_SENTINEL {
        duration.nanoseconds().clamp(i64::MIN as i128, i64::MAX as i128) as i64
    } else {
        nanoseconds
    };

    // Check for mixed signs
    let values = [new_years, new_months, new_weeks, new_days, new_hours, new_minutes,
                  new_seconds, new_milliseconds, new_microseconds, new_nanoseconds];
    let non_zero: Vec<i64> = values.iter().copied().filter(|&v| v != 0).collect();

    if !non_zero.is_empty() {
        let first_sign = non_zero[0].signum();
        if !non_zero.iter().all(|&v| v.signum() == first_sign) {
            return TemporalResult::range_error("All non-zero duration values must have the same sign");
        }
    }

    match Duration::new(
        new_years,
        new_months,
        new_weeks,
        new_days,
        new_hours,
        new_minutes,
        new_seconds,
        new_milliseconds,
        new_microseconds as i128,
        new_nanoseconds as i128,
    ) {
        Ok(duration) => TemporalResult::success(duration.to_string()),
        Err(e) => TemporalResult::range_error(&format!("Invalid duration: {}", e)),
    }
}

// Helper functions

fn parse_c_str(s: *const c_char, param_name: &str) -> Result<&str, TemporalResult> {
    if s.is_null() {
        return Err(TemporalResult::type_error(&format!("{} cannot be null", param_name)));
    }
    unsafe { std::ffi::CStr::from_ptr(s) }
        .to_str()
        .map_err(|_| TemporalResult::type_error(&format!("Invalid UTF-8 in {}", param_name)))
}

fn parse_duration(s: *const c_char, param_name: &str) -> Result<Duration, TemporalResult> {
    let str_val = parse_c_str(s, param_name)?;
    Duration::from_str(str_val)
        .map_err(|e| TemporalResult::range_error(&format!("Invalid duration '{}': {}", str_val, e)))
}

fn parse_instant(s: *const c_char, param_name: &str) -> Result<Instant, TemporalResult> {
    let str_val = parse_c_str(s, param_name)?;
    Instant::from_str(str_val)
        .map_err(|e| TemporalResult::range_error(&format!("Invalid instant '{}': {}", str_val, e)))
}

fn parse_plain_time(s: *const c_char, param_name: &str) -> Result<PlainTime, TemporalResult> {
    let str_val = parse_c_str(s, param_name)?;
    PlainTime::from_str(str_val)
        .map_err(|e| TemporalResult::range_error(&format!("Invalid plain time '{}': {}", str_val, e)))
}

fn duration_binary_op<F>(
    a: *const c_char,
    b: *const c_char,
    op_name: &str,
    op: F,
) -> TemporalResult
where
    F: FnOnce(Duration, Duration) -> Result<Duration, temporal_rs::TemporalError>,
{
    let duration_a = match parse_duration(a, "first duration") {
        Ok(d) => d,
        Err(e) => return e,
    };
    let duration_b = match parse_duration(b, "second duration") {
        Ok(d) => d,
        Err(e) => return e,
    };

    match op(duration_a, duration_b) {
        Ok(result) => TemporalResult::success(result.to_string()),
        Err(e) => TemporalResult::range_error(&format!("Failed to {} durations: {}", op_name, e)),
    }
}

fn duration_unary_op<F>(
    s: *const c_char,
    op_name: &str,
    op: F,
) -> TemporalResult
where
    F: FnOnce(Duration) -> Result<Duration, temporal_rs::TemporalError>,
{
    let duration = match parse_duration(s, "duration") {
        Ok(d) => d,
        Err(e) => return e,
    };

    match op(duration) {
        Ok(result) => TemporalResult::success(result.to_string()),
        Err(e) => TemporalResult::range_error(&format!("Failed to {} duration: {}", op_name, e)),
    }
}

// ============================================================================
// Android JNI bindings
// ============================================================================

#[cfg(target_os = "android")]
mod android {
    use jni::objects::{JClass, JString};
    use jni::sys::{jint, jlong, jlongArray, jstring};
    use jni::JNIEnv;

    use super::{
        get_instant_now_string, get_now_plain_date_string, get_now_plain_date_time_string,
        get_now_plain_time_string,
    };
    use temporal_rs::{
        options::ToStringRoundingOptions, Calendar, Duration, Instant, PlainTime,
    };
    use std::str::FromStr;
    use std::ptr;

    use timezone_provider::tzif::CompiledTzdbProvider;
    
    const RANGE_ERROR_CLASS: &str = "com/temporal/TemporalRangeError";
    const TYPE_ERROR_CLASS: &str = "com/temporal/TemporalTypeError";

    /// Throws a RangeError exception
    fn throw_range_error(env: &mut JNIEnv, message: &str) {
        let _ = env.throw_new(RANGE_ERROR_CLASS, message);
    }

    /// Throws a TypeError exception
    fn throw_type_error(env: &mut JNIEnv, message: &str) {
        let _ = env.throw_new(TYPE_ERROR_CLASS, message);
    }

    /// Parses a JNI string, throwing TypeError if null or invalid
    fn parse_jstring(env: &mut JNIEnv, s: &JString, name: &str) -> Option<String> {
        if s.is_null() {
            throw_type_error(env, &format!("{} cannot be null", name));
            return None;
        }
        match env.get_string(s) {
            Ok(js) => Some(js.to_string_lossy().into_owned()),
            Err(_) => {
                throw_type_error(env, &format!("Invalid UTF-8 in {}", name));
                None
            }
        }
    }

    /// Parses a duration string, throwing RangeError if invalid
    fn parse_duration(env: &mut JNIEnv, s: &JString, name: &str) -> Option<Duration> {
        let s_str = parse_jstring(env, s, name)?;
        match Duration::from_str(&s_str) {
            Ok(d) => Some(d),
            Err(e) => {
                throw_range_error(env, &format!("Invalid duration '{}': {}", s_str, e));
                None
            }
        }
    }

    /// Parses an instant string, throwing RangeError if invalid
    fn parse_instant(env: &mut JNIEnv, s: &JString, name: &str) -> Option<Instant> {
        let s_str = parse_jstring(env, s, name)?;
        match Instant::from_str(&s_str) {
            Ok(i) => Some(i),
            Err(e) => {
                throw_range_error(env, &format!("Invalid instant '{}': {}", s_str, e));
                None
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.instantNow()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_instantNow(
        mut env: JNIEnv,
        _class: JClass,
    ) -> jstring {
        match get_instant_now_string() {
            Ok(s) => env
                .new_string(s)
                .map(|js| js.into_raw())
                .unwrap_or_else(|_| {
                    throw_range_error(&mut env, "Failed to create string");
                    ptr::null_mut()
                }),
            Err(e) => {
                throw_range_error(&mut env, &format!("Failed to get current instant: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.instantFromString()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_instantFromString(
        mut env: JNIEnv,
        _class: JClass,
        s: JString,
    ) -> jstring {
        let instant = match parse_instant(&mut env, &s, "instant string") {
            Some(i) => i,
            None => return ptr::null_mut(),
        };
        let provider = CompiledTzdbProvider::default();
        match instant.to_ixdtf_string_with_provider(None, Default::default(), &provider) {
            Ok(s) => env
                .new_string(s)
                .map(|js| js.into_raw())
                .unwrap_or(ptr::null_mut()),
            Err(e) => {
                throw_range_error(&mut env, &format!("Failed to format instant: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.instantFromEpochMilliseconds()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_instantFromEpochMilliseconds(
        mut env: JNIEnv,
        _class: JClass,
        ms: jlong,
    ) -> jstring {
        let ns = (ms as i128).saturating_mul(1_000_000);
        match Instant::try_new(ns) {
            Ok(instant) => {
                let provider = CompiledTzdbProvider::default();
                match instant.to_ixdtf_string_with_provider(None, Default::default(), &provider) {
                    Ok(s) => env
                        .new_string(s)
                        .map(|js| js.into_raw())
                        .unwrap_or(ptr::null_mut()),
                    Err(e) => {
                        throw_range_error(&mut env, &format!("Failed to format instant: {}", e));
                        ptr::null_mut()
                    }
                }
            },
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid epoch milliseconds: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.instantFromEpochNanoseconds()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_instantFromEpochNanoseconds(
        mut env: JNIEnv,
        _class: JClass,
        ns_str: JString,
    ) -> jstring {
        let s_str = parse_jstring(&mut env, &ns_str, "nanoseconds string");
        let s_val = match s_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        
        let ns = match i128::from_str(&s_val) {
            Ok(n) => n,
            Err(_) => {
                throw_range_error(&mut env, "Invalid nanoseconds string");
                return ptr::null_mut();
            }
        };

        match Instant::try_new(ns) {
            Ok(instant) => {
                let provider = CompiledTzdbProvider::default();
                match instant.to_ixdtf_string_with_provider(None, Default::default(), &provider) {
                    Ok(s) => env
                        .new_string(s)
                        .map(|js| js.into_raw())
                        .unwrap_or(ptr::null_mut()),
                    Err(e) => {
                        throw_range_error(&mut env, &format!("Failed to format instant: {}", e));
                        ptr::null_mut()
                    }
                }
            },
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid epoch nanoseconds: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.instantEpochMilliseconds()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_instantEpochMilliseconds(
        mut env: JNIEnv,
        _class: JClass,
        s: JString,
    ) -> jstring {
        let instant = match parse_instant(&mut env, &s, "instant") {
            Some(i) => i,
            None => return ptr::null_mut(),
        };
        let ms = instant.epoch_milliseconds();
        env.new_string(ms.to_string())
            .map(|js| js.into_raw())
            .unwrap_or(ptr::null_mut())
    }

    /// JNI function for `com.temporal.TemporalNative.instantEpochNanoseconds()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_instantEpochNanoseconds(
        mut env: JNIEnv,
        _class: JClass,
        s: JString,
    ) -> jstring {
        let instant = match parse_instant(&mut env, &s, "instant") {
            Some(i) => i,
            None => return ptr::null_mut(),
        };
        let ns = instant.epoch_nanoseconds();
        env.new_string(ns.0.to_string())
            .map(|js| js.into_raw())
            .unwrap_or(ptr::null_mut())
    }

    /// JNI function for `com.temporal.TemporalNative.instantAdd()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_instantAdd(
        mut env: JNIEnv,
        _class: JClass,
        instant_str: JString,
        duration_str: JString,
    ) -> jstring {
        let instant = match parse_instant(&mut env, &instant_str, "instant") {
            Some(i) => i,
            None => return ptr::null_mut(),
        };
        let duration = match parse_duration(&mut env, &duration_str, "duration") {
            Some(d) => d,
            None => return ptr::null_mut(),
        };
        
        match instant.add(&duration) {
            Ok(result) => {
                let provider = CompiledTzdbProvider::default();
                match result.to_ixdtf_string_with_provider(None, Default::default(), &provider) {
                    Ok(s) => env
                        .new_string(s)
                        .map(|js| js.into_raw())
                        .unwrap_or(ptr::null_mut()),
                    Err(e) => {
                        throw_range_error(&mut env, &format!("Failed to format instant: {}", e));
                        ptr::null_mut()
                    }
                }
            },
            Err(e) => {
                throw_range_error(&mut env, &format!("Failed to add duration: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.instantSubtract()`

    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_instantSubtract(
        mut env: JNIEnv,
        _class: JClass,
        instant_str: JString,
        duration_str: JString,
    ) -> jstring {
        let instant = match parse_instant(&mut env, &instant_str, "instant") {
            Some(i) => i,
            None => return ptr::null_mut(),
        };
        let duration = match parse_duration(&mut env, &duration_str, "duration") {
            Some(d) => d,
            None => return ptr::null_mut(),
        };
        
        match instant.subtract(&duration) {
            Ok(result) => {
                let provider = CompiledTzdbProvider::default();
                match result.to_ixdtf_string_with_provider(None, Default::default(), &provider) {
                    Ok(s) => env
                        .new_string(s)
                        .map(|js| js.into_raw())
                        .unwrap_or(ptr::null_mut()),
                    Err(e) => {
                        throw_range_error(&mut env, &format!("Failed to format instant: {}", e));
                        ptr::null_mut()
                    }
                }
            },
            Err(e) => {
                throw_range_error(&mut env, &format!("Failed to subtract duration: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.instantCompare()`

    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_instantCompare(
        mut env: JNIEnv,
        _class: JClass,
        a: JString,
        b: JString,
    ) -> jint {
        let instant_a = match parse_instant(&mut env, &a, "first instant") {
            Some(i) => i,
            None => return 0,
        };
        let instant_b = match parse_instant(&mut env, &b, "second instant") {
            Some(i) => i,
            None => return 0,
        };
        
        instant_a.cmp(&instant_b) as jint
    }

    /// JNI function for `com.temporal.TemporalNative.nowPlainDateTimeISO()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_nowPlainDateTimeISO(
        mut env: JNIEnv,
        _class: JClass,
        tz_id: JString,
    ) -> jstring {
        let tz_str = parse_jstring(&mut env, &tz_id, "timezone id");
        let tz_val = match tz_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        
        match get_now_plain_date_time_string(&tz_val) {
            Ok(s) => env
                .new_string(s)
                .map(|js| js.into_raw())
                .unwrap_or(ptr::null_mut()),
            Err(e) => {
                throw_range_error(&mut env, &format!("Failed to get plain date time: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.nowPlainDateISO()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_nowPlainDateISO(
        mut env: JNIEnv,
        _class: JClass,
        tz_id: JString,
    ) -> jstring {
        let tz_str = parse_jstring(&mut env, &tz_id, "timezone id");
        let tz_val = match tz_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        
        match get_now_plain_date_string(&tz_val) {
            Ok(s) => env
                .new_string(s)
                .map(|js| js.into_raw())
                .unwrap_or(ptr::null_mut()),
            Err(e) => {
                throw_range_error(&mut env, &format!("Failed to get plain date: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.nowPlainTimeISO()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_nowPlainTimeISO(
        mut env: JNIEnv,
        _class: JClass,
        tz_id: JString,
    ) -> jstring {
        let tz_str = parse_jstring(&mut env, &tz_id, "timezone id");
        let tz_val = match tz_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        
        match get_now_plain_time_string(&tz_val) {
            Ok(s) => env
                .new_string(s)
                .map(|js| js.into_raw())
                .unwrap_or(ptr::null_mut()),
            Err(e) => {
                throw_range_error(&mut env, &format!("Failed to get plain time: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// Parses a PlainTime string, throwing RangeError if invalid
    fn parse_plain_time(env: &mut JNIEnv, s: &JString, name: &str) -> Option<PlainTime> {
        let s_str = parse_jstring(env, s, name)?;
        match PlainTime::from_str(&s_str) {
            Ok(t) => Some(t),
            Err(e) => {
                throw_range_error(env, &format!("Invalid plain time '{}': {}", s_str, e));
                None
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.plainTimeFromString()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainTimeFromString(
        mut env: JNIEnv,
        _class: JClass,
        s: JString,
    ) -> jstring {
        let time = match parse_plain_time(&mut env, &s, "plain time string") {
            Some(t) => t,
            None => return ptr::null_mut(),
        };
        match time.to_ixdtf_string(ToStringRoundingOptions::default()) {
            Ok(s) => env.new_string(s)
                .map(|js| js.into_raw())
                .unwrap_or(ptr::null_mut()),
            Err(e) => {
                throw_range_error(&mut env, &format!("Failed to format plain time: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.plainTimeFromComponents()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainTimeFromComponents(
        mut env: JNIEnv,
        _class: JClass,
        hour: jint,
        minute: jint,
        second: jint,
        millisecond: jint,
        microsecond: jint,
        nanosecond: jint,
    ) -> jstring {
        // Validate ranges before casting to narrower types
        if hour < 0 || hour > 23 {
            throw_range_error(&mut env, &format!("Invalid hour: {} (must be 0-23)", hour));
            return ptr::null_mut();
        }
        if minute < 0 || minute > 59 {
            throw_range_error(&mut env, &format!("Invalid minute: {} (must be 0-59)", minute));
            return ptr::null_mut();
        }
        if second < 0 || second > 59 {
            throw_range_error(&mut env, &format!("Invalid second: {} (must be 0-59)", second));
            return ptr::null_mut();
        }
        if millisecond < 0 || millisecond > 999 {
            throw_range_error(&mut env, &format!("Invalid millisecond: {} (must be 0-999)", millisecond));
            return ptr::null_mut();
        }
        if microsecond < 0 || microsecond > 999 {
            throw_range_error(&mut env, &format!("Invalid microsecond: {} (must be 0-999)", microsecond));
            return ptr::null_mut();
        }
        if nanosecond < 0 || nanosecond > 999 {
            throw_range_error(&mut env, &format!("Invalid nanosecond: {} (must be 0-999)", nanosecond));
            return ptr::null_mut();
        }

        match PlainTime::new(
            hour as u8,
            minute as u8,
            second as u8,
            millisecond as u16,
            microsecond as u16,
            nanosecond as u16
        ) {
            Ok(time) => match time.to_ixdtf_string(ToStringRoundingOptions::default()) {
                Ok(s) => env
                    .new_string(s)
                    .map(|js| js.into_raw())
                    .unwrap_or_else(|_| {
                        throw_range_error(&mut env, "Failed to create result string");
                        ptr::null_mut()
                    }),
                Err(e) => {
                    throw_range_error(&mut env, &format!("Failed to format plain time: {}", e));
                    ptr::null_mut()
                }
            },
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid plain time components: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.plainTimeGetAllComponents()`
    /// Returns: [hour, minute, second, millisecond, microsecond, nanosecond]
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainTimeGetAllComponents(
        mut env: JNIEnv,
        _class: JClass,
        s: JString,
    ) -> jlongArray {
        let time = match parse_plain_time(&mut env, &s, "plain time string") {
            Some(t) => t,
            None => return ptr::null_mut(),
        };

        let components: [i64; 6] = [
            time.hour() as i64,
            time.minute() as i64,
            time.second() as i64,
            time.millisecond() as i64,
            time.microsecond() as i64,
            time.nanosecond() as i64,
        ];

        match env.new_long_array(6) {
            Ok(arr) => {
                if env.set_long_array_region(&arr, 0, &components).is_err() {
                    throw_range_error(&mut env, "Failed to set array elements");
                    return ptr::null_mut();
                }
                arr.into_raw()
            }
            Err(_) => {
                throw_range_error(&mut env, "Failed to create result array");
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.plainTimeAdd()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainTimeAdd(
        mut env: JNIEnv,
        _class: JClass,
        time_str: JString,
        duration_str: JString,
    ) -> jstring {
        let time = match parse_plain_time(&mut env, &time_str, "plain time") {
            Some(t) => t,
            None => return ptr::null_mut(),
        };
        let duration = match parse_duration(&mut env, &duration_str, "duration") {
            Some(d) => d,
            None => return ptr::null_mut(),
        };

        match time.add(&duration) {
            Ok(result) => match result.to_ixdtf_string(ToStringRoundingOptions::default()) {
                Ok(s) => env
                    .new_string(s)
                    .map(|js| js.into_raw())
                    .unwrap_or(ptr::null_mut()),
                Err(e) => {
                    throw_range_error(&mut env, &format!("Failed to format plain time: {}", e));
                    ptr::null_mut()
                }
            },
            Err(e) => {
                throw_range_error(&mut env, &format!("Failed to add duration: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.plainTimeSubtract()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainTimeSubtract(
        mut env: JNIEnv,
        _class: JClass,
        time_str: JString,
        duration_str: JString,
    ) -> jstring {
        let time = match parse_plain_time(&mut env, &time_str, "plain time") {
            Some(t) => t,
            None => return ptr::null_mut(),
        };
        let duration = match parse_duration(&mut env, &duration_str, "duration") {
            Some(d) => d,
            None => return ptr::null_mut(),
        };

        match time.subtract(&duration) {
            Ok(result) => match result.to_ixdtf_string(ToStringRoundingOptions::default()) {
                Ok(s) => env
                    .new_string(s)
                    .map(|js| js.into_raw())
                    .unwrap_or(ptr::null_mut()),
                Err(e) => {
                    throw_range_error(&mut env, &format!("Failed to format plain time: {}", e));
                    ptr::null_mut()
                }
            },
            Err(e) => {
                throw_range_error(&mut env, &format!("Failed to subtract duration: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.plainTimeCompare()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainTimeCompare(
        mut env: JNIEnv,
        _class: JClass,
        a: JString,
        b: JString,
    ) -> jint {
        let time_a = match parse_plain_time(&mut env, &a, "first plain time") {
            Some(t) => t,
            None => return 0,
        };
        let time_b = match parse_plain_time(&mut env, &b, "second plain time") {
            Some(t) => t,
            None => return 0,
        };

        time_a.cmp(&time_b) as jint
    }

    /// JNI function for `com.temporal.TemporalNative.calendarFrom()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_calendarFrom(
        mut env: JNIEnv,
        _class: JClass,
        id: JString,
    ) -> jstring {
        let id_str = parse_jstring(&mut env, &id, "calendar identifier");
        let id_val = match id_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        
        match Calendar::from_str(&id_val) {
            Ok(calendar) => env
                .new_string(calendar.identifier().to_string())
                .map(|js| js.into_raw())
                .unwrap_or(ptr::null_mut()),
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid calendar identifier '{}': {}", id_val, e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.calendarId()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_calendarId(
        mut env: JNIEnv,
        _class: JClass,
        id: JString,
    ) -> jstring {
        // Just reusing calendarFrom logic since ID access is basically normalization
        Java_com_temporal_TemporalNative_calendarFrom(env, _class, id)
    }

    /// JNI function for `com.temporal.TemporalNative.durationFromString()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_durationFromString(
        mut env: JNIEnv,
        _class: JClass,
        input: JString,
    ) -> jstring {
        let duration = match parse_duration(&mut env, &input, "duration string") {
            Some(d) => d,
            None => return ptr::null_mut(),
        };

        env.new_string(duration.to_string())
            .map(|js| js.into_raw())
            .unwrap_or_else(|_| {
                throw_range_error(&mut env, "Failed to create result string");
                ptr::null_mut()
            })
    }

    /// JNI function for `com.temporal.TemporalNative.durationFromComponents()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_durationFromComponents(
        mut env: JNIEnv,
        _class: JClass,
        years: jlong,
        months: jlong,
        weeks: jlong,
        days: jlong,
        hours: jlong,
        minutes: jlong,
        seconds: jlong,
        milliseconds: jlong,
        microseconds: jlong,
        nanoseconds: jlong,
    ) -> jstring {
        // Check for mixed signs
        let values = [years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds];
        let non_zero: Vec<i64> = values.iter().copied().filter(|&v| v != 0).collect();

        if !non_zero.is_empty() {
            let first_sign = non_zero[0].signum();
            if !non_zero.iter().all(|&v| v.signum() == first_sign) {
                throw_range_error(&mut env, "All non-zero duration values must have the same sign");
                return ptr::null_mut();
            }
        }

        match Duration::new(
            years,
            months,
            weeks,
            days,
            hours,
            minutes,
            seconds,
            milliseconds,
            microseconds as i128,
            nanoseconds as i128,
        ) {
            Ok(duration) => env
                .new_string(duration.to_string())
                .map(|js| js.into_raw())
                .unwrap_or_else(|_| {
                    throw_range_error(&mut env, "Failed to create result string");
                    ptr::null_mut()
                }),
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid duration components: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.durationGetAllComponents()`
    /// Returns a long array: [years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds, sign, blank]
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_durationGetAllComponents(
        mut env: JNIEnv,
        _class: JClass,
        duration_str: JString,
    ) -> jlongArray {
        let duration = match parse_duration(&mut env, &duration_str, "duration string") {
            Some(d) => d,
            None => return ptr::null_mut(),
        };

        let components: [i64; 12] = [
            duration.years(),
            duration.months(),
            duration.weeks(),
            duration.days(),
            duration.hours(),
            duration.minutes(),
            duration.seconds(),
            duration.milliseconds(),
            duration.microseconds().clamp(i64::MIN as i128, i64::MAX as i128) as i64,
            duration.nanoseconds().clamp(i64::MIN as i128, i64::MAX as i128) as i64,
            duration.sign() as i64,
            if duration.is_zero() { 1 } else { 0 },
        ];

        match env.new_long_array(12) {
            Ok(arr) => {
                if env.set_long_array_region(&arr, 0, &components).is_err() {
                    throw_range_error(&mut env, "Failed to set array elements");
                    return ptr::null_mut();
                }
                arr.into_raw()
            }
            Err(_) => {
                throw_range_error(&mut env, "Failed to create result array");
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.durationAdd()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_durationAdd(
        mut env: JNIEnv,
        _class: JClass,
        a: JString,
        b: JString,
    ) -> jstring {
        let duration_a = match parse_duration(&mut env, &a, "first duration") {
            Some(d) => d,
            None => return ptr::null_mut(),
        };
        let duration_b = match parse_duration(&mut env, &b, "second duration") {
            Some(d) => d,
            None => return ptr::null_mut(),
        };

        match duration_a.add(&duration_b) {
            Ok(result) => env
                .new_string(result.to_string())
                .map(|js| js.into_raw())
                .unwrap_or_else(|_| {
                    throw_range_error(&mut env, "Failed to create result string");
                    ptr::null_mut()
                }),
            Err(e) => {
                throw_range_error(&mut env, &format!("Failed to add durations: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.durationSubtract()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_durationSubtract(
        mut env: JNIEnv,
        _class: JClass,
        a: JString,
        b: JString,
    ) -> jstring {
        let duration_a = match parse_duration(&mut env, &a, "first duration") {
            Some(d) => d,
            None => return ptr::null_mut(),
        };
        let duration_b = match parse_duration(&mut env, &b, "second duration") {
            Some(d) => d,
            None => return ptr::null_mut(),
        };

        match duration_a.subtract(&duration_b) {
            Ok(result) => env
                .new_string(result.to_string())
                .map(|js| js.into_raw())
                .unwrap_or_else(|_| {
                    throw_range_error(&mut env, "Failed to create result string");
                    ptr::null_mut()
                }),
            Err(e) => {
                throw_range_error(&mut env, &format!("Failed to subtract durations: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.durationNegated()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_durationNegated(
        mut env: JNIEnv,
        _class: JClass,
        s: JString,
    ) -> jstring {
        let duration = match parse_duration(&mut env, &s, "duration") {
            Some(d) => d,
            None => return ptr::null_mut(),
        };

        env.new_string(duration.negated().to_string())
            .map(|js| js.into_raw())
            .unwrap_or_else(|_| {
                throw_range_error(&mut env, "Failed to create result string");
                ptr::null_mut()
            })
    }

    /// JNI function for `com.temporal.TemporalNative.durationAbs()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_durationAbs(
        mut env: JNIEnv,
        _class: JClass,
        s: JString,
    ) -> jstring {
        let duration = match parse_duration(&mut env, &s, "duration") {
            Some(d) => d,
            None => return ptr::null_mut(),
        };

        env.new_string(duration.abs().to_string())
            .map(|js| js.into_raw())
            .unwrap_or_else(|_| {
                throw_range_error(&mut env, "Failed to create result string");
                ptr::null_mut()
            })
    }

    /// JNI function for `com.temporal.TemporalNative.durationCompare()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_durationCompare(
        mut env: JNIEnv,
        _class: JClass,
        a: JString,
        b: JString,
    ) -> jint {
        let duration_a = match parse_duration(&mut env, &a, "first duration") {
            Some(d) => d,
            None => return 0,
        };
        let duration_b = match parse_duration(&mut env, &b, "second duration") {
            Some(d) => d,
            None => return 0,
        };

        // Check if durations have calendar units
        let has_calendar_a = duration_a.years() != 0 || duration_a.months() != 0 || duration_a.weeks() != 0;
        let has_calendar_b = duration_b.years() != 0 || duration_b.months() != 0 || duration_b.weeks() != 0;

        if has_calendar_a || has_calendar_b {
            throw_range_error(&mut env, "Comparing durations with years, months, or weeks requires a relativeTo option (not yet supported)");
            return 0;
        }

        // For time-only durations, compare by total nanoseconds
        let total_a = duration_a.days() as i128 * 86_400_000_000_000
            + duration_a.hours() as i128 * 3_600_000_000_000
            + duration_a.minutes() as i128 * 60_000_000_000
            + duration_a.seconds() as i128 * 1_000_000_000
            + duration_a.milliseconds() as i128 * 1_000_000
            + duration_a.microseconds() * 1_000
            + duration_a.nanoseconds();

        let total_b = duration_b.days() as i128 * 86_400_000_000_000
            + duration_b.hours() as i128 * 3_600_000_000_000
            + duration_b.minutes() as i128 * 60_000_000_000
            + duration_b.seconds() as i128 * 1_000_000_000
            + duration_b.milliseconds() as i128 * 1_000_000
            + duration_b.microseconds() * 1_000
            + duration_b.nanoseconds();

        total_a.cmp(&total_b) as jint
    }

    /// Sentinel value for "unchanged" component in durationWith.
    /// Matches JavaScript's Number.MIN_SAFE_INTEGER (-(2^53 - 1)).
    const UNCHANGED_SENTINEL: i64 = -9007199254740991;

    /// JNI function for `com.temporal.TemporalNative.durationWith()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_durationWith(
        mut env: JNIEnv,
        _class: JClass,
        original: JString,
        years: jlong,
        months: jlong,
        weeks: jlong,
        days: jlong,
        hours: jlong,
        minutes: jlong,
        seconds: jlong,
        milliseconds: jlong,
        microseconds: jlong,
        nanoseconds: jlong,
    ) -> jstring {
        let duration = match parse_duration(&mut env, &original, "duration") {
            Some(d) => d,
            None => return ptr::null_mut(),
        };

        // Use original values for any component set to UNCHANGED_SENTINEL (sentinel)
        let new_years = if years == UNCHANGED_SENTINEL { duration.years() } else { years };
        let new_months = if months == UNCHANGED_SENTINEL { duration.months() } else { months };
        let new_weeks = if weeks == UNCHANGED_SENTINEL { duration.weeks() } else { weeks };
        let new_days = if days == UNCHANGED_SENTINEL { duration.days() } else { days };
        let new_hours = if hours == UNCHANGED_SENTINEL { duration.hours() } else { hours };
        let new_minutes = if minutes == UNCHANGED_SENTINEL { duration.minutes() } else { minutes };
        let new_seconds = if seconds == UNCHANGED_SENTINEL { duration.seconds() } else { seconds };
        let new_milliseconds = if milliseconds == UNCHANGED_SENTINEL { duration.milliseconds() } else { milliseconds };
        let new_microseconds = if microseconds == UNCHANGED_SENTINEL {
            duration.microseconds().clamp(i64::MIN as i128, i64::MAX as i128) as i64
        } else {
            microseconds
        };
        let new_nanoseconds = if nanoseconds == UNCHANGED_SENTINEL {
            duration.nanoseconds().clamp(i64::MIN as i128, i64::MAX as i128) as i64
        } else {
            nanoseconds
        };

        // Check for mixed signs
        let values = [new_years, new_months, new_weeks, new_days, new_hours, new_minutes,
                      new_seconds, new_milliseconds, new_microseconds, new_nanoseconds];
        let non_zero: Vec<i64> = values.iter().copied().filter(|&v| v != 0).collect();

        if !non_zero.is_empty() {
            let first_sign = non_zero[0].signum();
            if !non_zero.iter().all(|&v| v.signum() == first_sign) {
                throw_range_error(&mut env, "All non-zero duration values must have the same sign");
                return ptr::null_mut();
            }
        }

        match Duration::new(
            new_years,
            new_months,
            new_weeks,
            new_days,
            new_hours,
            new_minutes,
            new_seconds,
            new_milliseconds,
            new_microseconds as i128,
            new_nanoseconds as i128,
        ) {
            Ok(result) => env
                .new_string(result.to_string())
                .map(|js| js.into_raw())
                .unwrap_or_else(|_| {
                    throw_range_error(&mut env, "Failed to create result string");
                    ptr::null_mut()
                }),
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid duration: {}", e));
                ptr::null_mut()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    // Helper to extract value from TemporalResult or panic with error message
    fn extract_result(mut result: TemporalResult) -> String {
        if result.error_type != TemporalErrorType::None as i32 {
            let error_msg = if !result.error_message.is_null() {
                unsafe { std::ffi::CStr::from_ptr(result.error_message) }
                    .to_string_lossy()
                    .to_string()
            } else {
                "Unknown error".to_string()
            };
            unsafe { temporal_free_result(&mut result) };
            panic!("TemporalResult error: {}", error_msg);
        }

        let value = if !result.value.is_null() {
            unsafe { std::ffi::CStr::from_ptr(result.value) }
                .to_string_lossy()
                .to_string()
        } else {
            String::new()
        };
        
        unsafe { temporal_free_result(&mut result) };
        value
    }

    #[test]
    fn test_instant_now() {
        let result = get_instant_now_string().unwrap();
        // Should be in ISO 8601 format like "2024-01-15T10:30:45.123456789Z"
        assert!(result.ends_with('Z'), "Expected UTC timestamp: {}", result);
        assert!(result.contains('T'), "Expected ISO format: {}", result);
        println!("Current instant: {}", result);
    }

    #[test]
    fn test_duration_from_string_valid() {
        let input = CString::new("P1Y2M3DT4H5M6S").unwrap();
        let result = temporal_duration_from_string(input.as_ptr());
        let result_string = extract_result(result);
        
        // Should parse and normalize the duration
        assert!(result_string.starts_with('P'), "Should start with P: {}", result_string);
    }

    #[test]
    fn test_duration_from_string_invalid() {
        let input = CString::new("invalid").unwrap();
        let result = temporal_duration_from_string(input.as_ptr());
        assert_eq!(result.error_type, TemporalErrorType::RangeError as i32, "Invalid duration should return RangeError");
        assert!(!result.error_message.is_null(), "Should have error message");
        unsafe { temporal_free_result(&mut { result }) };
    }

    #[test]
    fn test_duration_from_string_null() {
        let result = temporal_duration_from_string(ptr::null());
        assert_eq!(result.error_type, TemporalErrorType::TypeError as i32, "Null input should return TypeError");
        unsafe { temporal_free_result(&mut { result }) };
    }

    #[test]
    fn test_duration_get_components() {
        let input = CString::new("P1Y2M3W4DT5H6M7S").unwrap();
        let mut components = DurationComponents::default();
        
        temporal_duration_get_components(input.as_ptr(), &mut components);
        
        assert_eq!(components.is_valid, 1, "Should be valid");
        assert_eq!(components.years, 1);
        assert_eq!(components.months, 2);
        assert_eq!(components.weeks, 3);
        assert_eq!(components.days, 4);
        assert_eq!(components.hours, 5);
        assert_eq!(components.minutes, 6);
        assert_eq!(components.seconds, 7);
        assert_eq!(components.sign, 1, "Positive duration should have sign 1");
    }

    #[test]
    fn test_duration_get_components_negative() {
        let input = CString::new("-P1Y2M").unwrap();
        let mut components = DurationComponents::default();
        
        temporal_duration_get_components(input.as_ptr(), &mut components);
        
        assert_eq!(components.is_valid, 1);
        assert_eq!(components.years, -1);
        assert_eq!(components.months, -2);
        assert_eq!(components.sign, -1, "Negative duration should have sign -1");
    }

    #[test]
    fn test_duration_get_components_zero() {
        let input = CString::new("PT0S").unwrap();
        let mut components = DurationComponents::default();
        
        temporal_duration_get_components(input.as_ptr(), &mut components);
        
        assert_eq!(components.is_valid, 1);
        assert_eq!(components.sign, 0, "Zero duration should have sign 0");
    }

    #[test]
    fn test_duration_get_components_invalid() {
        let input = CString::new("invalid").unwrap();
        let mut components = DurationComponents::default();
        
        temporal_duration_get_components(input.as_ptr(), &mut components);
        
        assert_eq!(components.is_valid, 0, "Invalid duration should set is_valid to 0");
    }

    #[test]
    fn test_duration_add() {
        // Use time-only durations which don't require relative context
        let a = CString::new("PT1H30M").unwrap();
        let b = CString::new("PT2H15M").unwrap();
        
        let result = temporal_duration_add(a.as_ptr(), b.as_ptr());
        let result_string = extract_result(result);
        
        // PT1H30M + PT2H15M = PT3H45M
        assert!(result_string.contains("3H"), "1H30M + 2H15M should contain 3H: {}", result_string);
        assert!(result_string.contains("45M"), "1H30M + 2H15M should contain 45M: {}", result_string);
    }

    #[test]
    fn test_duration_subtract() {
        // Use time-only durations which don't require relative context
        let a = CString::new("PT3H45M").unwrap();
        let b = CString::new("PT1H15M").unwrap();
        
        let result = temporal_duration_subtract(a.as_ptr(), b.as_ptr());
        let result_string = extract_result(result);
        
        // PT3H45M - PT1H15M = PT2H30M
        assert!(result_string.contains("2H"), "3H45M - 1H15M should contain 2H: {}", result_string);
        assert!(result_string.contains("30M"), "3H45M - 1H15M should contain 30M: {}", result_string);
    }

    #[test]
    fn test_duration_negated() {
        let input = CString::new("P1Y2M").unwrap();
        
        let result = temporal_duration_negated(input.as_ptr());
        let result_string = extract_result(result);
        
        // Negation should produce negative duration
        assert!(result_string.starts_with("-P"), "Negated should start with -P: {}", result_string);
    }

    #[test]
    fn test_duration_abs() {
        let input = CString::new("-P1Y2M").unwrap();
        
        let result = temporal_duration_abs(input.as_ptr());
        let result_string = extract_result(result);
        
        // Absolute value should be positive
        assert!(result_string.starts_with('P') && !result_string.starts_with("-P"), 
                "Abs should be positive: {}", result_string);
    }

    #[test]
    fn test_error_types() {
        // Test TypeError for null input
        let result = temporal_duration_from_string(ptr::null());
        assert_eq!(result.error_type, TemporalErrorType::TypeError as i32);
        unsafe { temporal_free_result(&mut { result }) };
        
        // Test RangeError for invalid format
        let invalid = CString::new("not-a-duration").unwrap();
        let result = temporal_duration_from_string(invalid.as_ptr());
        assert_eq!(result.error_type, TemporalErrorType::RangeError as i32);
        
        // Check error message contains useful info
        let error_msg = unsafe { std::ffi::CStr::from_ptr(result.error_message) }
            .to_string_lossy()
            .to_string();
        assert!(error_msg.contains("not-a-duration"), "Error message should include input: {}", error_msg);
        unsafe { temporal_free_result(&mut { result }) };
    }
}

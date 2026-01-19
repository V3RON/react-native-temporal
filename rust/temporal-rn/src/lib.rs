use std::ffi::{c_char, CString};
use std::ptr;
use std::str::FromStr;

use temporal_rs::sys::Temporal;
use temporal_rs::{
    options::{DisplayCalendar, ToStringRoundingOptions, DisplayOffset, DisplayTimeZone, Disambiguation, OffsetDisambiguation, Overflow, RoundingOptions, RoundingMode, Unit, RoundingIncrement},
    Calendar, Duration, Instant, PlainDate, PlainDateTime, PlainMonthDay, PlainTime,
    PlainYearMonth, TimeZone, ZonedDateTime, TemporalError,
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

#[no_mangle]
pub extern "C" fn temporal_now_zoned_date_time_iso(tz_id: *const c_char) -> TemporalResult {
    let tz_str = match parse_c_str(tz_id, "timezone id") {
        Ok(s) => s,
        Err(e) => return e,
    };
    
    match get_now_zoned_date_time_string(tz_str) {
        Ok(s) => TemporalResult::success(s),
        Err(e) => TemporalResult::range_error(&format!("Failed to get zoned date time: {}", e)),
    }
}

fn get_now_zoned_date_time_string(tz_id: &str) -> Result<String, Box<dyn std::error::Error>> {
    let now = Temporal::utc_now();
    let instant = now.instant()?;
    let time_zone = TimeZone::try_from_str(tz_id)?;
    let zdt = instant.to_zoned_date_time_iso(time_zone)?;
    Ok(zdt.to_ixdtf_string(DisplayOffset::Auto, DisplayTimeZone::Auto, DisplayCalendar::Auto, ToStringRoundingOptions::default())?)
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
// PlainDate API
// ============================================================================

/// Represents a PlainDate's component values for FFI.
#[repr(C)]
pub struct PlainDateComponents {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub day_of_week: u16,
    pub day_of_year: u16,
    pub week_of_year: u16,
    pub year_of_week: i32,
    pub days_in_week: u16,
    pub days_in_month: u16,
    pub days_in_year: u16,
    pub months_in_year: u16,
    pub in_leap_year: i8,
    pub is_valid: i8,
}

impl Default for PlainDateComponents {
    fn default() -> Self {
        Self {
            year: 0,
            month: 0,
            day: 0,
            day_of_week: 0,
            day_of_year: 0,
            week_of_year: 0,
            year_of_week: 0,
            days_in_week: 0,
            days_in_month: 0,
            days_in_year: 0,
            months_in_year: 0,
            in_leap_year: 0,
            is_valid: 0,
        }
    }
}

/// Parses an ISO 8601 string into a PlainDate and returns the normalized string.
#[no_mangle]
pub extern "C" fn temporal_plain_date_from_string(s: *const c_char) -> TemporalResult {
    let s_str = match parse_c_str(s, "plain date string") {
        Ok(s) => s,
        Err(e) => return e,
    };
    match PlainDate::from_str(s_str) {
        Ok(date) => TemporalResult::success(date.to_ixdtf_string(DisplayCalendar::Auto)),
        Err(e) => TemporalResult::range_error(&format!("Invalid plain date '{}': {}", s_str, e)),
    }
}

/// Creates a PlainDate from components.
#[no_mangle]
pub extern "C" fn temporal_plain_date_from_components(
    year: i32,
    month: u8,
    day: u8,
    calendar_id: *const c_char,
) -> TemporalResult {
    let calendar = if !calendar_id.is_null() {
        match parse_c_str(calendar_id, "calendar id") {
            Ok(s) => match Calendar::from_str(s) {
                Ok(c) => c,
                Err(e) => return TemporalResult::range_error(&format!("Invalid calendar: {}", e)),
            },
            Err(e) => return e,
        }
    } else {
        Calendar::default()
    };

    match PlainDate::new(year, month, day, calendar) {
        Ok(date) => TemporalResult::success(date.to_ixdtf_string(DisplayCalendar::Auto)),
        Err(e) => TemporalResult::range_error(&format!("Invalid plain date components: {}", e)),
    }
}

/// Gets all integer component values from a PlainDate string.
#[no_mangle]
pub extern "C" fn temporal_plain_date_get_components(
    s: *const c_char,
    out: *mut PlainDateComponents,
) {
    if out.is_null() {
        return;
    }

    unsafe { *out = PlainDateComponents::default(); }

    if s.is_null() {
        return;
    }

    let date = match parse_plain_date(s, "plain date") {
        Ok(d) => d,
        Err(_) => return,
    };

    unsafe {
        (*out).year = date.year();
        (*out).month = date.month();
        (*out).day = date.day();
        (*out).day_of_week = date.day_of_week();
        (*out).day_of_year = date.day_of_year();
        (*out).week_of_year = date.week_of_year().unwrap_or(0) as u16;
        (*out).year_of_week = date.year_of_week().unwrap_or(0);
        (*out).days_in_week = date.days_in_week();
        (*out).days_in_month = date.days_in_month();
        (*out).days_in_year = date.days_in_year();
        (*out).months_in_year = date.months_in_year();
        (*out).in_leap_year = if date.in_leap_year() { 1 } else { 0 };
        (*out).is_valid = 1;
    }
}

/// Gets the month code of a PlainDate.
#[no_mangle]
pub extern "C" fn temporal_plain_date_get_month_code(s: *const c_char) -> TemporalResult {
    let date = match parse_plain_date(s, "plain date") {
        Ok(d) => d,
        Err(e) => return e,
    };
    TemporalResult::success(date.month_code().as_str().to_string())
}

/// Gets the calendar ID of a PlainDate.
#[no_mangle]
pub extern "C" fn temporal_plain_date_get_calendar(s: *const c_char) -> TemporalResult {
    let date = match parse_plain_date(s, "plain date") {
        Ok(d) => d,
        Err(e) => return e,
    };
    TemporalResult::success(date.calendar().identifier().to_string())
}

/// Adds a duration to a PlainDate.
#[no_mangle]
pub extern "C" fn temporal_plain_date_add(date_str: *const c_char, duration_str: *const c_char) -> TemporalResult {
    let date = match parse_plain_date(date_str, "plain date") {
        Ok(d) => d,
        Err(e) => return e,
    };
    let duration = match parse_duration(duration_str, "duration") {
        Ok(d) => d,
        Err(e) => return e,
    };

    match date.add(&duration, None) {
        Ok(result) => TemporalResult::success(result.to_ixdtf_string(DisplayCalendar::Auto)),
        Err(e) => TemporalResult::range_error(&format!("Failed to add duration: {}", e)),
    }
}

/// Subtracts a duration from a PlainDate.
#[no_mangle]
pub extern "C" fn temporal_plain_date_subtract(date_str: *const c_char, duration_str: *const c_char) -> TemporalResult {
    let date = match parse_plain_date(date_str, "plain date") {
        Ok(d) => d,
        Err(e) => return e,
    };
    let duration = match parse_duration(duration_str, "duration") {
        Ok(d) => d,
        Err(e) => return e,
    };

    match date.subtract(&duration, None) {
        Ok(result) => TemporalResult::success(result.to_ixdtf_string(DisplayCalendar::Auto)),
        Err(e) => TemporalResult::range_error(&format!("Failed to subtract duration: {}", e)),
    }
}

/// Compares two PlainDates.
#[no_mangle]
pub extern "C" fn temporal_plain_date_compare(a: *const c_char, b: *const c_char) -> CompareResult {
    let date_a = match parse_plain_date(a, "first plain date") {
        Ok(d) => d,
        Err(e) => return CompareResult::range_error(
            &unsafe { std::ffi::CStr::from_ptr(e.error_message) }.to_string_lossy()
        ),
    };
    let date_b = match parse_plain_date(b, "second plain date") {
        Ok(d) => d,
        Err(e) => return CompareResult::range_error(
            &unsafe { std::ffi::CStr::from_ptr(e.error_message) }.to_string_lossy()
        ),
    };

    // Fallback to string comparison since direct comparison is not exposed/working
    // Use DisplayCalendar::Never to compare pure ISO dates without calendar annotations
    let s_a = date_a.to_ixdtf_string(DisplayCalendar::Never);
    let s_b = date_b.to_ixdtf_string(DisplayCalendar::Never);

    let val = match s_a.cmp(&s_b) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    };

    CompareResult::success(val)
}

/// Returns a new PlainDate with updated fields.
#[no_mangle]
pub extern "C" fn temporal_plain_date_with(
    date_str: *const c_char,
    year: i32,
    month: i32,
    day: i32,
    calendar_id: *const c_char,
) -> TemporalResult {
    let date = match parse_plain_date(date_str, "plain date") {
        Ok(d) => d,
        Err(e) => return e,
    };
    
    let new_year = if year == i32::MIN { date.year() } else { year };
    let new_month = if month == i32::MIN { date.month() } else { month as u8 };
    let new_day = if day == i32::MIN { date.day() } else { day as u8 };
    
    let new_calendar = if !calendar_id.is_null() {
        match parse_c_str(calendar_id, "calendar id") {
            Ok(s) => match Calendar::from_str(s) {
                Ok(c) => c,
                Err(e) => return TemporalResult::range_error(&format!("Invalid calendar: {}", e)),
            },
            Err(e) => return e,
        }
    } else {
        date.calendar().clone()
    };

    match PlainDate::new(new_year, new_month, new_day, new_calendar) {
         Ok(new_date) => TemporalResult::success(new_date.to_ixdtf_string(DisplayCalendar::Auto)),
        Err(e) => TemporalResult::range_error(&format!("Invalid date components: {}", e)),
    }
}

/// Computes the difference between two PlainDates (until).
#[no_mangle]
pub extern "C" fn temporal_plain_date_until(
    one_str: *const c_char,
    two_str: *const c_char,
) -> TemporalResult {
    let one = match parse_plain_date(one_str, "first plain date") {
        Ok(d) => d,
        Err(e) => return e,
    };
    let two = match parse_plain_date(two_str, "second plain date") {
        Ok(d) => d,
        Err(e) => return e,
    };

    match one.until(&two, Default::default()) {
        Ok(d) => TemporalResult::success(d.to_string()),
        Err(e) => TemporalResult::range_error(&format!("Failed to compute difference: {}", e)),
    }
}

/// Computes the difference between two PlainDates (since).
#[no_mangle]
pub extern "C" fn temporal_plain_date_since(
    one_str: *const c_char,
    two_str: *const c_char,
) -> TemporalResult {
    let one = match parse_plain_date(one_str, "first plain date") {
        Ok(d) => d,
        Err(e) => return e,
    };
    let two = match parse_plain_date(two_str, "second plain date") {
        Ok(d) => d,
        Err(e) => return e,
    };

    match one.since(&two, Default::default()) {
        Ok(d) => TemporalResult::success(d.to_string()),
        Err(e) => TemporalResult::range_error(&format!("Failed to compute difference: {}", e)),
    }
}

// Helper functions for PlainDate
fn parse_plain_date(s: *const c_char, param_name: &str) -> Result<PlainDate, TemporalResult> {
    let str_val = parse_c_str(s, param_name)?;
    PlainDate::from_str(str_val)
        .map_err(|e| TemporalResult::range_error(&format!("Invalid plain date '{}': {}", str_val, e)))
}

// ============================================================================
// PlainDateTime API
// ============================================================================

/// Represents a PlainDateTime's component values for FFI.
#[repr(C)]
pub struct PlainDateTimeComponents {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub day_of_week: u16,
    pub day_of_year: u16,
    pub week_of_year: u16,
    pub year_of_week: i32,
    pub days_in_week: u16,
    pub days_in_month: u16,
    pub days_in_year: u16,
    pub months_in_year: u16,
    pub in_leap_year: i8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub millisecond: u16,
    pub microsecond: u16,
    pub nanosecond: u16,
    pub is_valid: i8,
}

impl Default for PlainDateTimeComponents {
    fn default() -> Self {
        Self {
            year: 0,
            month: 0,
            day: 0,
            day_of_week: 0,
            day_of_year: 0,
            week_of_year: 0,
            year_of_week: 0,
            days_in_week: 0,
            days_in_month: 0,
            days_in_year: 0,
            months_in_year: 0,
            in_leap_year: 0,
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

/// Parses an ISO 8601 string into a PlainDateTime and returns the normalized string.
#[no_mangle]
pub extern "C" fn temporal_plain_date_time_from_string(s: *const c_char) -> TemporalResult {
    let s_str = match parse_c_str(s, "plain date time string") {
        Ok(s) => s,
        Err(e) => return e,
    };
    match PlainDateTime::from_str(s_str) {
        Ok(dt) => match dt.to_ixdtf_string(ToStringRoundingOptions::default(), DisplayCalendar::Auto) {
            Ok(s) => TemporalResult::success(s),
            Err(e) => TemporalResult::range_error(&format!("Failed to format plain date time: {}", e)),
        },
        Err(e) => TemporalResult::range_error(&format!("Invalid plain date time '{}': {}", s_str, e)),
    }
}

/// Creates a PlainDateTime from components.
#[no_mangle]
pub extern "C" fn temporal_plain_date_time_from_components(
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    millisecond: u16,
    microsecond: u16,
    nanosecond: u16,
    calendar_id: *const c_char,
) -> TemporalResult {
    let calendar = if !calendar_id.is_null() {
        match parse_c_str(calendar_id, "calendar id") {
            Ok(s) => match Calendar::from_str(s) {
                Ok(c) => c,
                Err(e) => return TemporalResult::range_error(&format!("Invalid calendar: {}", e)),
            },
            Err(e) => return e,
        }
    } else {
        Calendar::default()
    };

    match PlainDateTime::new(year, month, day, hour, minute, second, millisecond, microsecond, nanosecond, calendar) {
        Ok(dt) => match dt.to_ixdtf_string(ToStringRoundingOptions::default(), DisplayCalendar::Auto) {
            Ok(s) => TemporalResult::success(s),
            Err(e) => TemporalResult::range_error(&format!("Failed to format plain date time: {}", e)),
        },
        Err(e) => TemporalResult::range_error(&format!("Invalid plain date time components: {}", e)),
    }
}

/// Gets all component values from a PlainDateTime string.
#[no_mangle]
pub extern "C" fn temporal_plain_date_time_get_components(
    s: *const c_char,
    out: *mut PlainDateTimeComponents,
) {
    if out.is_null() {
        return;
    }

    unsafe { *out = PlainDateTimeComponents::default(); }

    if s.is_null() {
        return;
    }

    let dt: PlainDateTime = match parse_plain_date_time(s, "plain date time") {
        Ok(d) => d,
        Err(_) => return,
    };

    unsafe {
        (*out).year = dt.year();
        (*out).month = dt.month();
        (*out).day = dt.day();
        (*out).day_of_week = dt.day_of_week();
        (*out).day_of_year = dt.day_of_year();
        (*out).week_of_year = dt.week_of_year().unwrap_or(0) as u16;
        (*out).year_of_week = dt.year_of_week().unwrap_or(0);
        (*out).days_in_week = dt.days_in_week();
        (*out).days_in_month = dt.days_in_month();
        (*out).days_in_year = dt.days_in_year();
        (*out).months_in_year = dt.months_in_year();
        (*out).in_leap_year = if dt.in_leap_year() { 1 } else { 0 };

        (*out).hour = dt.hour();
        (*out).minute = dt.minute();
        (*out).second = dt.second();
        (*out).millisecond = dt.millisecond();
        (*out).microsecond = dt.microsecond();
        (*out).nanosecond = dt.nanosecond();
        
        (*out).is_valid = 1;
    }
}

/// Gets the month code of a PlainDateTime.
#[no_mangle]
pub extern "C" fn temporal_plain_date_time_get_month_code(s: *const c_char) -> TemporalResult {
    let dt = match parse_plain_date_time(s, "plain date time") {
        Ok(d) => d,
        Err(e) => return e,
    };
    TemporalResult::success(dt.month_code().as_str().to_string())
}

/// Gets the calendar ID of a PlainDateTime.
#[no_mangle]
pub extern "C" fn temporal_plain_date_time_get_calendar(s: *const c_char) -> TemporalResult {
    let dt = match parse_plain_date_time(s, "plain date time") {
        Ok(d) => d,
        Err(e) => return e,
    };
    TemporalResult::success(dt.calendar().identifier().to_string())
}

/// Adds a duration to a PlainDateTime.
#[no_mangle]
pub extern "C" fn temporal_plain_date_time_add(dt_str: *const c_char, duration_str: *const c_char) -> TemporalResult {
    let dt: PlainDateTime = match parse_plain_date_time(dt_str, "plain date time") {
        Ok(d) => d,
        Err(e) => return e,
    };
    let duration = match parse_duration(duration_str, "duration") {
        Ok(d) => d,
        Err(e) => return e,
    };

    match dt.add(&duration, None) {
        Ok(result) => match result.to_ixdtf_string(ToStringRoundingOptions::default(), DisplayCalendar::Auto) {
            Ok(s) => TemporalResult::success(s),
            Err(e) => TemporalResult::range_error(&format!("Failed to format plain date time: {}", e)),
        },
        Err(e) => TemporalResult::range_error(&format!("Failed to add duration: {}", e)),
    }
}

/// Subtracts a duration from a PlainDateTime.
#[no_mangle]
pub extern "C" fn temporal_plain_date_time_subtract(dt_str: *const c_char, duration_str: *const c_char) -> TemporalResult {
    let dt: PlainDateTime = match parse_plain_date_time(dt_str, "plain date time") {
        Ok(d) => d,
        Err(e) => return e,
    };
    let duration = match parse_duration(duration_str, "duration") {
        Ok(d) => d,
        Err(e) => return e,
    };

    match dt.subtract(&duration, None) {
        Ok(result) => match result.to_ixdtf_string(ToStringRoundingOptions::default(), DisplayCalendar::Auto) {
            Ok(s) => TemporalResult::success(s),
            Err(e) => TemporalResult::range_error(&format!("Failed to format plain date time: {}", e)),
        },
        Err(e) => TemporalResult::range_error(&format!("Failed to subtract duration: {}", e)),
    }
}

/// Compares two PlainDateTimes.
#[no_mangle]
pub extern "C" fn temporal_plain_date_time_compare(a: *const c_char, b: *const c_char) -> CompareResult {
    let dt_a: PlainDateTime = match parse_plain_date_time(a, "first plain date time") {
        Ok(d) => d,
        Err(e) => return CompareResult::range_error(
            &unsafe { std::ffi::CStr::from_ptr(e.error_message) }.to_string_lossy()
        ),
    };
    let dt_b: PlainDateTime = match parse_plain_date_time(b, "second plain date time") {
        Ok(d) => d,
        Err(e) => return CompareResult::range_error(
            &unsafe { std::ffi::CStr::from_ptr(e.error_message) }.to_string_lossy()
        ),
    };

    CompareResult::success(dt_a.compare_iso(&dt_b) as i32)
}

/// Returns a new PlainDateTime with updated fields.
#[no_mangle]
pub extern "C" fn temporal_plain_date_time_with(
    dt_str: *const c_char,
    year: i32,
    month: i32,
    day: i32,
    hour: i32,
    minute: i32,
    second: i32,
    millisecond: i32,
    microsecond: i32,
    nanosecond: i32,
    calendar_id: *const c_char,
) -> TemporalResult {
    let dt: PlainDateTime = match parse_plain_date_time(dt_str, "plain date time") {
        Ok(d) => d,
        Err(e) => return e,
    };
    
    let new_year = if year == i32::MIN { dt.year() } else { year };
    let new_month = if month == i32::MIN { dt.month() } else { month as u8 };
    let new_day = if day == i32::MIN { dt.day() } else { day as u8 };
    
    let new_hour = if hour == i32::MIN { dt.hour() } else { hour as u8 };
    let new_minute = if minute == i32::MIN { dt.minute() } else { minute as u8 };
    let new_second = if second == i32::MIN { dt.second() } else { second as u8 };
    let new_millisecond = if millisecond == i32::MIN { dt.millisecond() } else { millisecond as u16 };
    let new_microsecond = if microsecond == i32::MIN { dt.microsecond() } else { microsecond as u16 };
    let new_nanosecond = if nanosecond == i32::MIN { dt.nanosecond() } else { nanosecond as u16 };

    let new_calendar = if !calendar_id.is_null() {
        match parse_c_str(calendar_id, "calendar id") {
            Ok(s) => match Calendar::from_str(s) {
                Ok(c) => c,
                Err(e) => return TemporalResult::range_error(&format!("Invalid calendar: {}", e)),
            },
            Err(e) => return e,
        }
    } else {
        dt.calendar().clone()
    };

    match PlainDateTime::new(new_year, new_month, new_day, new_hour, new_minute, new_second, new_millisecond, new_microsecond, new_nanosecond, new_calendar) {
         Ok(new_dt) => match new_dt.to_ixdtf_string(ToStringRoundingOptions::default(), DisplayCalendar::Auto) {
             Ok(s) => TemporalResult::success(s),
             Err(e) => TemporalResult::range_error(&format!("Failed to format plain date time: {}", e)),
         },
        Err(e) => TemporalResult::range_error(&format!("Invalid date time components: {}", e)),
    }
}

/// Computes the difference between two PlainDateTimes (until).
#[no_mangle]
pub extern "C" fn temporal_plain_date_time_until(
    one_str: *const c_char,
    two_str: *const c_char,
) -> TemporalResult {
    let one: PlainDateTime = match parse_plain_date_time(one_str, "first plain date time") {
        Ok(d) => d,
        Err(e) => return e,
    };
    let two: PlainDateTime = match parse_plain_date_time(two_str, "second plain date time") {
        Ok(d) => d,
        Err(e) => return e,
    };

    match one.until(&two, Default::default()) {
        Ok(d) => TemporalResult::success(d.to_string()),
        Err(e) => TemporalResult::range_error(&format!("Failed to compute difference: {}", e)),
    }
}

/// Computes the difference between two PlainDateTimes (since).
#[no_mangle]
pub extern "C" fn temporal_plain_date_time_since(
    one_str: *const c_char,
    two_str: *const c_char,
) -> TemporalResult {
    let one: PlainDateTime = match parse_plain_date_time(one_str, "first plain date time") {
        Ok(d) => d,
        Err(e) => return e,
    };
    let two: PlainDateTime = match parse_plain_date_time(two_str, "second plain date time") {
        Ok(d) => d,
        Err(e) => return e,
    };

    match one.since(&two, Default::default()) {
        Ok(d) => TemporalResult::success(d.to_string()),
        Err(e) => TemporalResult::range_error(&format!("Failed to compute difference: {}", e)),
    }
}

// Helper functions for PlainDateTime
fn parse_plain_date_time(s: *const c_char, param_name: &str) -> Result<PlainDateTime, TemporalResult> {
    let str_val = parse_c_str(s, param_name)?;
    PlainDateTime::from_str(str_val)
        .map_err(|e| TemporalResult::range_error(&format!("Invalid plain date time '{}': {}", str_val, e)))
}

// ============================================================================
// PlainYearMonth API
// ============================================================================

/// Represents a PlainYearMonth's component values for FFI.
#[repr(C)]
pub struct PlainYearMonthComponents {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub days_in_month: u16,
    pub days_in_year: u16,
    pub months_in_year: u16,
    pub in_leap_year: i8,
    pub era_year: i32,
    pub is_valid: i8,
}

impl Default for PlainYearMonthComponents {
    fn default() -> Self {
        Self {
            year: 0,
            month: 0,
            day: 0,
            days_in_month: 0,
            days_in_year: 0,
            months_in_year: 0,
            in_leap_year: 0,
            era_year: 0,
            is_valid: 0,
        }
    }
}

/// Parses an ISO 8601 string into a PlainYearMonth.
#[no_mangle]
pub extern "C" fn temporal_plain_year_month_from_string(s: *const c_char) -> TemporalResult {
    let s_str = match parse_c_str(s, "plain year month string") {
        Ok(s) => s,
        Err(e) => return e,
    };
    match PlainYearMonth::from_str(s_str) {
        Ok(ym) => TemporalResult::success(ym.to_ixdtf_string(DisplayCalendar::Auto)),
        Err(e) => TemporalResult::range_error(&format!("Invalid plain year month '{}': {}", s_str, e)),
    }
}

/// Creates a PlainYearMonth from components.
#[no_mangle]
pub extern "C" fn temporal_plain_year_month_from_components(
    year: i32,
    month: u8,
    calendar_id: *const c_char,
    _reference_day: u8,
) -> TemporalResult {
    let calendar = if !calendar_id.is_null() {
        match parse_c_str(calendar_id, "calendar id") {
            Ok(s) => match Calendar::from_str(s) {
                Ok(c) => c,
                Err(e) => return TemporalResult::range_error(&format!("Invalid calendar: {}", e)),
            },
            Err(e) => return e,
        }
    } else {
        Calendar::default()
    };

    // Note: reference_day is typically handled by the JS layer or implicit in Rust
    // temporal_rs PlainYearMonth::new takes (year, month, calendar).
    // If reference_day is non-zero, it might be used for disambiguation in other implementations,
    // but here we primarily use year/month.
    
    match PlainYearMonth::new(year, month, None, calendar) {
        Ok(ym) => TemporalResult::success(ym.to_ixdtf_string(DisplayCalendar::Auto)),
        Err(e) => TemporalResult::range_error(&format!("Invalid plain year month components: {}", e)),
    }
}

/// Gets components from a PlainYearMonth string.
#[no_mangle]
pub extern "C" fn temporal_plain_year_month_get_components(
    s: *const c_char,
    out: *mut PlainYearMonthComponents,
) {
    if out.is_null() { return; }
    unsafe { *out = PlainYearMonthComponents::default(); }
    if s.is_null() { return; }

    let ym = match parse_plain_year_month(s, "plain year month") {
        Ok(y) => y,
        Err(_) => return,
    };

    unsafe {
        (*out).year = ym.year();
        (*out).month = ym.month();
        (*out).day = 0; // PlainYearMonth does not have a day
        (*out).days_in_month = ym.days_in_month();
        (*out).days_in_year = ym.days_in_year();
        (*out).months_in_year = ym.months_in_year();
        (*out).in_leap_year = if ym.in_leap_year() { 1 } else { 0 };
        (*out).era_year = ym.era_year().unwrap_or(0);
        (*out).is_valid = 1;
    }
}

/// Gets the month code of a PlainYearMonth.
#[no_mangle]
pub extern "C" fn temporal_plain_year_month_get_month_code(s: *const c_char) -> TemporalResult {
    let ym = match parse_plain_year_month(s, "plain year month") {
        Ok(y) => y,
        Err(e) => return e,
    };
    TemporalResult::success(ym.month_code().as_str().to_string())
}

/// Gets the calendar ID of a PlainYearMonth.
#[no_mangle]
pub extern "C" fn temporal_plain_year_month_get_calendar(s: *const c_char) -> TemporalResult {
    let ym = match parse_plain_year_month(s, "plain year month") {
        Ok(y) => y,
        Err(e) => return e,
    };
    TemporalResult::success(ym.calendar().identifier().to_string())
}

/// Adds a duration to a PlainYearMonth.
#[no_mangle]
pub extern "C" fn temporal_plain_year_month_add(
    ym_str: *const c_char,
    duration_str: *const c_char,
) -> TemporalResult {
    let ym = match parse_plain_year_month(ym_str, "plain year month") {
        Ok(y) => y,
        Err(e) => return e,
    };
    let duration = match parse_duration(duration_str, "duration") {
        Ok(d) => d,
        Err(e) => return e,
    };

    match ym.add(&duration, temporal_rs::options::Overflow::Reject) {
        Ok(result) => TemporalResult::success(result.to_ixdtf_string(DisplayCalendar::Auto)),
        Err(e) => TemporalResult::range_error(&format!("Failed to add duration: {}", e)),
    }
}

/// Subtracts a duration from a PlainYearMonth.
#[no_mangle]
pub extern "C" fn temporal_plain_year_month_subtract(
    ym_str: *const c_char,
    duration_str: *const c_char,
) -> TemporalResult {
    let ym = match parse_plain_year_month(ym_str, "plain year month") {
        Ok(y) => y,
        Err(e) => return e,
    };
    let duration = match parse_duration(duration_str, "duration") {
        Ok(d) => d,
        Err(e) => return e,
    };

    match ym.subtract(&duration, temporal_rs::options::Overflow::Reject) {
        Ok(result) => TemporalResult::success(result.to_ixdtf_string(DisplayCalendar::Auto)),
        Err(e) => TemporalResult::range_error(&format!("Failed to subtract duration: {}", e)),
    }
}

/// Compares two PlainYearMonths.
#[no_mangle]
pub extern "C" fn temporal_plain_year_month_compare(a: *const c_char, b: *const c_char) -> CompareResult {
    let ym_a = match parse_plain_year_month(a, "first plain year month") {
        Ok(y) => y,
        Err(e) => return CompareResult::range_error(
            &unsafe { std::ffi::CStr::from_ptr(e.error_message) }.to_string_lossy()
        ),
    };
    let ym_b = match parse_plain_year_month(b, "second plain year month") {
        Ok(y) => y,
        Err(e) => return CompareResult::range_error(
            &unsafe { std::ffi::CStr::from_ptr(e.error_message) }.to_string_lossy()
        ),
    };

    // PlainYearMonth doesn't have a direct compare method in temporal_rs that is public/exposed easily
    // But we can compare ISO representations if calendars are the same, or compare fields.
    // However, the spec says to compare ISO dates.
    // Let's use to_plain_date with day=1 comparison as proxy or ISO string compare.
    // For now, let's use string comparison of ISO format (normalized).
    
    let s_a = ym_a.to_ixdtf_string(DisplayCalendar::Never);
    let s_b = ym_b.to_ixdtf_string(DisplayCalendar::Never);
    
    let val = match s_a.cmp(&s_b) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    };
    
    CompareResult::success(val)
}

/// Returns a new PlainYearMonth with updated fields.
#[no_mangle]
pub extern "C" fn temporal_plain_year_month_with(
    ym_str: *const c_char,
    year: i32,
    month: i32,
    calendar_id: *const c_char,
) -> TemporalResult {
    let ym = match parse_plain_year_month(ym_str, "plain year month") {
        Ok(y) => y,
        Err(e) => return e,
    };

    let new_year = if year == i32::MIN { ym.year() } else { year };
    let new_month = if month == i32::MIN { ym.month() } else { month as u8 };
    
    let new_calendar = if !calendar_id.is_null() {
        match parse_c_str(calendar_id, "calendar id") {
            Ok(s) => match Calendar::from_str(s) {
                Ok(c) => c,
                Err(e) => return TemporalResult::range_error(&format!("Invalid calendar: {}", e)),
            },
            Err(e) => return e,
        }
    } else {
        ym.calendar().clone()
    };

    match PlainYearMonth::new(new_year, new_month, None, new_calendar) {
        Ok(new_ym) => TemporalResult::success(new_ym.to_ixdtf_string(DisplayCalendar::Auto)),
        Err(e) => TemporalResult::range_error(&format!("Invalid components: {}", e)),
    }
}

/// Computes difference (until).
#[no_mangle]
pub extern "C" fn temporal_plain_year_month_until(
    one_str: *const c_char,
    two_str: *const c_char,
) -> TemporalResult {
    let one = match parse_plain_year_month(one_str, "first plain year month") {
        Ok(y) => y,
        Err(e) => return e,
    };
    let two = match parse_plain_year_month(two_str, "second plain year month") {
        Ok(y) => y,
        Err(e) => return e,
    };

    match one.until(&two, Default::default()) {
        Ok(d) => TemporalResult::success(d.to_string()),
        Err(e) => TemporalResult::range_error(&format!("Failed to compute difference: {}", e)),
    }
}

/// Computes difference (since).
#[no_mangle]
pub extern "C" fn temporal_plain_year_month_since(
    one_str: *const c_char,
    two_str: *const c_char,
) -> TemporalResult {
    let one = match parse_plain_year_month(one_str, "first plain year month") {
        Ok(y) => y,
        Err(e) => return e,
    };
    let two = match parse_plain_year_month(two_str, "second plain year month") {
        Ok(y) => y,
        Err(e) => return e,
    };

    match one.since(&two, Default::default()) {
        Ok(d) => TemporalResult::success(d.to_string()),
        Err(e) => TemporalResult::range_error(&format!("Failed to compute difference: {}", e)),
    }
}

/// Converts to PlainDate.
#[no_mangle]
pub extern "C" fn temporal_plain_year_month_to_plain_date(
    ym_str: *const c_char,
    day: i32,
) -> TemporalResult {
    let ym = match parse_plain_year_month(ym_str, "plain year month") {
        Ok(y) => y,
        Err(e) => return e,
    };

    // temporal_rs PlainYearMonth doesn't have a direct to_plain_date(day) method yet?
    // Checking crate... PlainYearMonth usually has to_plain_date.
    // If not, we construct PlainDate from components.
    
    // Construct manually:
    match PlainDate::new(ym.year(), ym.month(), day as u8, ym.calendar().clone()) {
        Ok(d) => TemporalResult::success(d.to_ixdtf_string(DisplayCalendar::Auto)),
        Err(e) => TemporalResult::range_error(&format!("Failed to convert to plain date: {}", e)),
    }
}

// Helper
fn parse_plain_year_month(s: *const c_char, param_name: &str) -> Result<PlainYearMonth, TemporalResult> {
    let str_val = parse_c_str(s, param_name)?;
    PlainYearMonth::from_str(str_val)
        .map_err(|e| TemporalResult::range_error(&format!("Invalid plain year month '{}': {}", str_val, e)))
}

// ============================================================================
// PlainMonthDay API
// ============================================================================

/// Represents a PlainMonthDay's component values for FFI.
#[repr(C)]
pub struct PlainMonthDayComponents {
    pub month: u8,
    pub day: u8,
    pub is_valid: i8,
}

impl Default for PlainMonthDayComponents {
    fn default() -> Self {
        Self {
            month: 0,
            day: 0,
            is_valid: 0,
        }
    }
}

/// Parses an ISO 8601 string into a PlainMonthDay.
#[no_mangle]
pub extern "C" fn temporal_plain_month_day_from_string(s: *const c_char) -> TemporalResult {
    let s_str = match parse_c_str(s, "plain month day string") {
        Ok(s) => s,
        Err(e) => return e,
    };
    match PlainMonthDay::from_str(s_str) {
        Ok(md) => TemporalResult::success(md.to_ixdtf_string(DisplayCalendar::Auto)),
        Err(e) => TemporalResult::range_error(&format!("Invalid plain month day '{}': {}", s_str, e)),
    }
}

/// Creates a PlainMonthDay from components.
#[no_mangle]
pub extern "C" fn temporal_plain_month_day_from_components(
    month: u8,
    day: u8,
    calendar_id: *const c_char,
    _reference_year: i32,
) -> TemporalResult {
    let calendar = if !calendar_id.is_null() {
        match parse_c_str(calendar_id, "calendar id") {
            Ok(s) => match Calendar::from_str(s) {
                Ok(c) => c,
                Err(e) => return TemporalResult::range_error(&format!("Invalid calendar: {}", e)),
            },
            Err(e) => return e,
        }
    } else {
        Calendar::default()
    };

    // temporal_rs PlainMonthDay::new takes (month, day, calendar).
    // Reference year is implicit or handled by logic if needed, but basic constructor doesn't take it?
    // Wait, PlainMonthDay usually needs a reference year for leap years (Feb 29).
    // Let's check constructor.
    
    // Assuming new(month, day, calendar) works and uses iso8601 reference year if needed.
    // If reference_year is provided, we might need a different constructor or logic.
    // For now, let's try standard new.
    
    match PlainMonthDay::new_with_overflow(month, day, calendar, temporal_rs::options::Overflow::Reject, None) {
        Ok(md) => TemporalResult::success(md.to_ixdtf_string(DisplayCalendar::Auto)),
        Err(e) => TemporalResult::range_error(&format!("Invalid plain month day components: {}", e)),
    }
}

/// Gets components from a PlainMonthDay string.
#[no_mangle]
pub extern "C" fn temporal_plain_month_day_get_components(
    s: *const c_char,
    out: *mut PlainMonthDayComponents,
) {
    if out.is_null() { return; }
    unsafe { *out = PlainMonthDayComponents::default(); }
    if s.is_null() { return; }

    let md = match parse_plain_month_day(s, "plain month day") {
        Ok(m) => m,
        Err(_) => return,
    };

    unsafe {
        (*out).month = match u8::from_str(md.month_code().as_str().trim_start_matches('M')) {
            Ok(m) => m,
            Err(_) => 0
        };
        (*out).day = md.day();
        (*out).is_valid = 1;
    }
}

/// Gets the month code of a PlainMonthDay.
#[no_mangle]
pub extern "C" fn temporal_plain_month_day_get_month_code(s: *const c_char) -> TemporalResult {
    let md = match parse_plain_month_day(s, "plain month day") {
        Ok(m) => m,
        Err(e) => return e,
    };
    TemporalResult::success(md.month_code().as_str().to_string())
}

/// Gets the calendar ID of a PlainMonthDay.
#[no_mangle]
pub extern "C" fn temporal_plain_month_day_get_calendar(s: *const c_char) -> TemporalResult {
    let md = match parse_plain_month_day(s, "plain month day") {
        Ok(m) => m,
        Err(e) => return e,
    };
    TemporalResult::success(md.calendar().identifier().to_string())
}

/// Converts to PlainDate.
#[no_mangle]
pub extern "C" fn temporal_plain_month_day_to_plain_date(
    md_str: *const c_char,
    year: i32,
) -> TemporalResult {
    let md = match parse_plain_month_day(md_str, "plain month day") {
        Ok(m) => m,
        Err(e) => return e,
    };

    let month = match u8::from_str(md.month_code().as_str().trim_start_matches('M')) {
        Ok(m) => m,
        Err(_) => return TemporalResult::range_error("Failed to parse month from month code"),
    };

    match PlainDate::new(year, month, md.day(), md.calendar().clone()) {
        Ok(d) => TemporalResult::success(d.to_ixdtf_string(DisplayCalendar::Auto)),
        Err(e) => TemporalResult::range_error(&format!("Failed to convert to plain date: {}", e)),
    }
}

// Helper
fn parse_plain_month_day(s: *const c_char, param_name: &str) -> Result<PlainMonthDay, TemporalResult> {
    let str_val = parse_c_str(s, param_name)?;
    PlainMonthDay::from_str(str_val)
        .map_err(|e| TemporalResult::range_error(&format!("Invalid plain month day '{}': {}", str_val, e)))
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


// ============================================================================
// TimeZone API
// ============================================================================

/// Gets a TimeZone from a string identifier.
#[no_mangle]
pub extern "C" fn temporal_time_zone_from_string(s: *const c_char) -> TemporalResult {
    let s_str = match parse_c_str(s, "timezone string") {
        Ok(s) => s,
        Err(e) => return e,
    };
    match TimeZone::try_from_str(s_str) {
        Ok(tz) => match tz.identifier() {
            Ok(id) => TemporalResult::success(id),
            Err(e) => TemporalResult::range_error(&format!("Failed to get timezone id: {}", e)),
        },
        Err(e) => TemporalResult::range_error(&format!("Invalid timezone '{}': {}", s_str, e)),
    }
}

/// Gets the identifier of a TimeZone.
#[no_mangle]
pub extern "C" fn temporal_time_zone_get_id(s: *const c_char) -> TemporalResult {
    let s_str = match parse_c_str(s, "timezone string") {
        Ok(s) => s,
        Err(e) => return e,
    };
    match TimeZone::try_from_str(s_str) {
        Ok(tz) => match tz.identifier() {
            Ok(id) => TemporalResult::success(id),
            Err(e) => TemporalResult::range_error(&format!("Failed to get timezone id: {}", e)),
        },
        Err(e) => TemporalResult::range_error(&format!("Invalid timezone '{}': {}", s_str, e)),
    }
}

/// Gets the offset nanoseconds for an instant in a timezone.
#[no_mangle]
pub extern "C" fn temporal_time_zone_get_offset_nanoseconds_for(
    tz_id: *const c_char,
    instant_str: *const c_char,
) -> TemporalResult {
    let tz = match parse_time_zone(tz_id, "timezone") {
        Ok(t) => t,
        Err(e) => return e,
    };
    let instant = match parse_instant(instant_str, "instant") {
        Ok(i) => i,
        Err(e) => return e,
    };

    let provider = CompiledTzdbProvider::default();
    match ZonedDateTime::try_new(instant.epoch_nanoseconds().0, tz, Calendar::default()) {
        Ok(zdt) => TemporalResult::success(zdt.offset_nanoseconds().to_string()),
        Err(e) => TemporalResult::range_error(&format!("Failed to get offset: {}", e)),
    }
}

/// Gets the offset string for an instant in a timezone.
#[no_mangle]
pub extern "C" fn temporal_time_zone_get_offset_string_for(
    tz_id: *const c_char,
    instant_str: *const c_char,
) -> TemporalResult {
    let tz = match parse_time_zone(tz_id, "timezone") {
        Ok(t) => t,
        Err(e) => return e,
    };
    let instant = match parse_instant(instant_str, "instant") {
        Ok(i) => i,
        Err(e) => return e,
    };

    let provider = CompiledTzdbProvider::default();
    match ZonedDateTime::try_new(instant.epoch_nanoseconds().0, tz, Calendar::default()) {
        Ok(zdt) => TemporalResult::success(zdt.offset().to_string()),
        Err(e) => TemporalResult::range_error(&format!("Failed to get offset string: {}", e)),
    }
}

/// Gets the PlainDateTime for an instant in a timezone.
#[no_mangle]
pub extern "C" fn temporal_time_zone_get_plain_date_time_for(
    tz_id: *const c_char,
    instant_str: *const c_char,
    calendar_id: *const c_char,
) -> TemporalResult {
    let tz = match parse_time_zone(tz_id, "timezone") {
        Ok(t) => t,
        Err(e) => return e,
    };
    let instant = match parse_instant(instant_str, "instant") {
        Ok(i) => i,
        Err(e) => return e,
    };
    
    let calendar = if !calendar_id.is_null() {
        match parse_c_str(calendar_id, "calendar id") {
            Ok(s) => match Calendar::from_str(s) {
                Ok(c) => c,
                Err(e) => return TemporalResult::range_error(&format!("Invalid calendar: {}", e)),
            },
            Err(e) => return e,
        }
    } else {
        Calendar::default()
    };

    match ZonedDateTime::try_new(instant.epoch_nanoseconds().0, tz, calendar) {
        Ok(zdt) => match zdt.to_plain_date_time().to_ixdtf_string(ToStringRoundingOptions::default(), DisplayCalendar::Auto) {
            Ok(s) => TemporalResult::success(s),
            Err(e) => TemporalResult::range_error(&format!("Failed to format plain date time: {}", e)),
        },
        Err(e) => TemporalResult::range_error(&format!("Failed to get plain date time: {}", e)),
    }
}

/// Gets the Instant for a PlainDateTime in a timezone.
#[no_mangle]
pub extern "C" fn temporal_time_zone_get_instant_for(
    tz_id: *const c_char,
    dt_str: *const c_char,
    disambiguation: *const c_char,
) -> TemporalResult {
    let tz = match parse_time_zone(tz_id, "timezone") {
        Ok(t) => t,
        Err(e) => return e,
    };
    let dt = match parse_plain_date_time(dt_str, "plain date time") {
        Ok(d) => d,
        Err(e) => return e,
    };

    let disambig_enum = if !disambiguation.is_null() {
        match parse_c_str(disambiguation, "disambiguation") {
            Ok(s) => match s {
                "compatible" => Disambiguation::Compatible,
                "earlier" => Disambiguation::Earlier,
                "later" => Disambiguation::Later,
                "reject" => Disambiguation::Reject,
                _ => Disambiguation::Compatible,
            },
            Err(e) => return e,
        }
    } else {
        Disambiguation::Compatible
    };

    match dt.to_zoned_date_time(tz, disambig_enum) {
        Ok(zdt) => {
             let instant = zdt.to_instant();
             let provider = CompiledTzdbProvider::default();
             match instant.to_ixdtf_string_with_provider(None, Default::default(), &provider) {
                Ok(s) => TemporalResult::success(s),
                Err(e) => TemporalResult::range_error(&format!("Failed to format instant: {}", e)),
             }
        },
        Err(e) => TemporalResult::range_error(&format!("Failed to get instant: {}", e)),
    }
}

/// Gets the next transition instant.
#[no_mangle]
pub extern "C" fn temporal_time_zone_get_next_transition(
    tz_id: *const c_char,
    instant_str: *const c_char,
) -> TemporalResult {
    let tz = match parse_time_zone(tz_id, "timezone") {
        Ok(t) => t,
        Err(e) => return e,
    };
    let instant = match parse_instant(instant_str, "instant") {
        Ok(i) => i,
        Err(e) => return e,
    };

    // TODO: Implement using provider directly when API is clear
    match Ok::<Option<Instant>, TemporalError>(None) { // Stub
        Ok(Some(i)) => {
            let provider = CompiledTzdbProvider::default();
            match i.to_ixdtf_string_with_provider(None, Default::default(), &provider) {
                Ok(s) => TemporalResult::success(s),
                Err(e) => TemporalResult::range_error(&format!("Failed to format instant: {}", e)),
            }
        },
        Ok(None) => TemporalResult::success(String::new()),
        Err(e) => TemporalResult::range_error(&format!("Failed to get next transition: {}", e)),
    }
}

/// Gets the previous transition instant.
#[no_mangle]
pub extern "C" fn temporal_time_zone_get_previous_transition(
    tz_id: *const c_char,
    instant_str: *const c_char,
) -> TemporalResult {
    let tz = match parse_time_zone(tz_id, "timezone") {
        Ok(t) => t,
        Err(e) => return e,
    };
    let instant = match parse_instant(instant_str, "instant") {
        Ok(i) => i,
        Err(e) => return e,
    };

    // TODO: Implement using provider directly
    match Ok::<Option<Instant>, TemporalError>(None) {
        Ok(Some(i)) => {
            let provider = CompiledTzdbProvider::default();
            match i.to_ixdtf_string_with_provider(None, Default::default(), &provider) {
                Ok(s) => TemporalResult::success(s),
                Err(e) => TemporalResult::range_error(&format!("Failed to format instant: {}", e)),
            }
        },
        Ok(None) => TemporalResult::success(String::new()),
        Err(e) => TemporalResult::range_error(&format!("Failed to get previous transition: {}", e)),
    }
}

// ============================================================================
// ZonedDateTime API
// ============================================================================

/// Represents a ZonedDateTime's component values for FFI.
#[repr(C)]
pub struct ZonedDateTimeComponents {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub day_of_week: u16,
    pub day_of_year: u16,
    pub week_of_year: u16,
    pub year_of_week: i32,
    pub days_in_week: u16,
    pub days_in_month: u16,
    pub days_in_year: u16,
    pub months_in_year: u16,
    pub in_leap_year: i8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub millisecond: u16,
    pub microsecond: u16,
    pub nanosecond: u16,
    pub offset_nanoseconds: i64,
    pub is_valid: i8,
}

impl Default for ZonedDateTimeComponents {
    fn default() -> Self {
        Self {
            year: 0,
            month: 0,
            day: 0,
            day_of_week: 0,
            day_of_year: 0,
            week_of_year: 0,
            year_of_week: 0,
            days_in_week: 0,
            days_in_month: 0,
            days_in_year: 0,
            months_in_year: 0,
            in_leap_year: 0,
            hour: 0,
            minute: 0,
            second: 0,
            millisecond: 0,
            microsecond: 0,
            nanosecond: 0,
            offset_nanoseconds: 0,
            is_valid: 0,
        }
    }
}

/// Parses an ISO 8601 string into a ZonedDateTime.
#[no_mangle]
pub extern "C" fn temporal_zoned_date_time_from_string(
    s: *const c_char,
) -> TemporalResult {
    let s_str = match parse_c_str(s, "zoned date time string") {
        Ok(s) => s,
        Err(e) => return e,
    };
    
    // Using default provider (TZDB)
    match ZonedDateTime::from_utf8(s_str.as_bytes(), Disambiguation::Compatible, OffsetDisambiguation::Reject) {
        Ok(zdt) => match zdt.to_ixdtf_string(DisplayOffset::Auto, DisplayTimeZone::Auto, DisplayCalendar::Auto, ToStringRoundingOptions::default()) {
            Ok(s) => TemporalResult::success(s),
            Err(e) => TemporalResult::range_error(&format!("Failed to format zoned date time: {}", e)),
        },
        Err(e) => TemporalResult::range_error(&format!("Invalid zoned date time '{}': {}", s_str, e)),
    }
}

/// Creates a ZonedDateTime from components.
#[no_mangle]
pub extern "C" fn temporal_zoned_date_time_from_components(
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    millisecond: u16,
    microsecond: u16,
    nanosecond: u16,
    calendar_id: *const c_char,
    time_zone_id: *const c_char,
    offset_nanoseconds: i64, // Optional offset for conflict resolution, 0 if ignored? 
    // Spec: needs disambiguation options if offset is ignored/provided
) -> TemporalResult {
    // Constructing ZDT from components usually requires creating a PlainDateTime first, 
    // then converting to ZDT with timezone and disambiguation.
    
    let calendar = if !calendar_id.is_null() {
        match parse_c_str(calendar_id, "calendar id") {
            Ok(s) => match Calendar::from_str(s) {
                Ok(c) => c,
                Err(e) => return TemporalResult::range_error(&format!("Invalid calendar: {}", e)),
            },
            Err(e) => return e,
        }
    } else {
        Calendar::default()
    };

    let pdt = match PlainDateTime::new(
        year, month, day, 
        hour, minute, second, 
        millisecond, microsecond, nanosecond, 
        calendar
    ) {
        Ok(d) => d,
        Err(e) => return TemporalResult::range_error(&format!("Invalid components: {}", e)),
    };

    let tz_str = if !time_zone_id.is_null() {
        match parse_c_str(time_zone_id, "timezone id") {
            Ok(s) => s,
            Err(e) => return e,
        }
    } else {
        return TemporalResult::type_error("Timezone ID is required");
    };

    let tz = match TimeZone::try_from_str(tz_str) {
        Ok(t) => t,
        Err(e) => return TemporalResult::range_error(&format!("Invalid timezone: {}", e)),
    };

    // We create ZDT from PDT + TZ. 
    // TC39 `from` usually takes an object with components and options.
    // Here we assume standard construction (compatible disambiguation).
    
    match pdt.to_zoned_date_time(tz, Disambiguation::Compatible) { // None = compatible/default
        Ok(zdt) => match zdt.to_ixdtf_string(DisplayOffset::Auto, DisplayTimeZone::Auto, DisplayCalendar::Auto, ToStringRoundingOptions::default()) {
            Ok(s) => TemporalResult::success(s),
            Err(e) => TemporalResult::range_error(&format!("Failed to format zoned date time: {}", e)),
        },
        Err(e) => TemporalResult::range_error(&format!("Failed to create zoned date time: {}", e)),
    }
}

/// Gets components from a ZonedDateTime string.
#[no_mangle]
pub extern "C" fn temporal_zoned_date_time_get_components(
    s: *const c_char,
    out: *mut ZonedDateTimeComponents,
) {
    if out.is_null() { return; }
    unsafe { *out = ZonedDateTimeComponents::default(); }
    if s.is_null() { return; }

    let zdt = match parse_zoned_date_time(s, "zoned date time") {
        Ok(z) => z,
        Err(_) => return,
    };

    unsafe {
        (*out).year = zdt.year();
        (*out).month = zdt.month();
        (*out).day = zdt.day();
        (*out).day_of_week = zdt.day_of_week();
        (*out).day_of_year = zdt.day_of_year();
        (*out).week_of_year = zdt.week_of_year().unwrap_or(0) as u16;
        (*out).year_of_week = zdt.year_of_week().unwrap_or(0);
        (*out).days_in_week = zdt.days_in_week();
        (*out).days_in_month = zdt.days_in_month();
        (*out).days_in_year = zdt.days_in_year();
        (*out).months_in_year = zdt.months_in_year();
        (*out).in_leap_year = if zdt.in_leap_year() { 1 } else { 0 };
        
        (*out).hour = zdt.hour();
        (*out).minute = zdt.minute();
        (*out).second = zdt.second();
        (*out).millisecond = zdt.millisecond();
        (*out).microsecond = zdt.microsecond();
        (*out).nanosecond = zdt.nanosecond();
        
        (*out).offset_nanoseconds = zdt.offset_nanoseconds() as i64;
        
        (*out).is_valid = 1;
    }
}

/// Gets the epoch values.
#[no_mangle]
pub extern "C" fn temporal_zoned_date_time_epoch_milliseconds(s: *const c_char) -> TemporalResult {
    let zdt = match parse_zoned_date_time(s, "zoned date time") {
        Ok(z) => z,
        Err(e) => return e,
    };
    TemporalResult::success(zdt.epoch_milliseconds().to_string())
}

#[no_mangle]
pub extern "C" fn temporal_zoned_date_time_epoch_nanoseconds(s: *const c_char) -> TemporalResult {
    let zdt = match parse_zoned_date_time(s, "zoned date time") {
        Ok(z) => z,
        Err(e) => return e,
    };
    TemporalResult::success(zdt.epoch_nanoseconds().0.to_string())
}

/// Gets the calendar ID.
#[no_mangle]
pub extern "C" fn temporal_zoned_date_time_get_calendar(s: *const c_char) -> TemporalResult {
    let zdt = match parse_zoned_date_time(s, "zoned date time") {
        Ok(z) => z,
        Err(e) => return e,
    };
    TemporalResult::success(zdt.calendar().identifier().to_string())
}

/// Gets the TimeZone ID.
#[no_mangle]
pub extern "C" fn temporal_zoned_date_time_get_time_zone(s: *const c_char) -> TemporalResult {
    let zdt = match parse_zoned_date_time(s, "zoned date time") {
        Ok(z) => z,
        Err(e) => return e,
    };
    match zdt.time_zone().identifier() {
        Ok(id) => TemporalResult::success(id),
        Err(e) => TemporalResult::range_error(&format!("Failed to get timezone id: {}", e)),
    }
}

/// Gets the offset string.
#[no_mangle]
pub extern "C" fn temporal_zoned_date_time_get_offset(s: *const c_char) -> TemporalResult {
    let zdt = match parse_zoned_date_time(s, "zoned date time") {
        Ok(z) => z,
        Err(e) => return e,
    };
    TemporalResult::success(zdt.offset().to_string())
}

/// Adds a duration.
#[no_mangle]
pub extern "C" fn temporal_zoned_date_time_add(
    zdt_str: *const c_char,
    duration_str: *const c_char,
) -> TemporalResult {
    let zdt = match parse_zoned_date_time(zdt_str, "zoned date time") {
        Ok(z) => z,
        Err(e) => return e,
    };
    let duration = match parse_duration(duration_str, "duration") {
        Ok(d) => d,
        Err(e) => return e,
    };

    match zdt.add(&duration, Some(Overflow::Reject)) {
        Ok(result) => match result.to_ixdtf_string(DisplayOffset::Auto, DisplayTimeZone::Auto, DisplayCalendar::Auto, ToStringRoundingOptions::default()) {
            Ok(s) => TemporalResult::success(s),
            Err(e) => TemporalResult::range_error(&format!("Failed to format: {}", e)),
        },
        Err(e) => TemporalResult::range_error(&format!("Failed to add duration: {}", e)),
    }
}

/// Subtracts a duration.
#[no_mangle]
pub extern "C" fn temporal_zoned_date_time_subtract(
    zdt_str: *const c_char,
    duration_str: *const c_char,
) -> TemporalResult {
    let zdt = match parse_zoned_date_time(zdt_str, "zoned date time") {
        Ok(z) => z,
        Err(e) => return e,
    };
    let duration = match parse_duration(duration_str, "duration") {
        Ok(d) => d,
        Err(e) => return e,
    };

    match zdt.subtract(&duration, Some(Overflow::Reject)) {
        Ok(result) => match result.to_ixdtf_string(DisplayOffset::Auto, DisplayTimeZone::Auto, DisplayCalendar::Auto, ToStringRoundingOptions::default()) {
            Ok(s) => TemporalResult::success(s),
            Err(e) => TemporalResult::range_error(&format!("Failed to format: {}", e)),
        },
        Err(e) => TemporalResult::range_error(&format!("Failed to subtract duration: {}", e)),
    }
}

/// Compares two ZonedDateTimes.
#[no_mangle]
pub extern "C" fn temporal_zoned_date_time_compare(
    a: *const c_char,
    b: *const c_char,
) -> CompareResult {
    let zdt_a = match parse_zoned_date_time(a, "first zoned date time") {
        Ok(z) => z,
        Err(e) => return CompareResult::range_error(
            &unsafe { std::ffi::CStr::from_ptr(e.error_message) }.to_string_lossy()
        ),
    };
    let zdt_b = match parse_zoned_date_time(b, "second zoned date time") {
        Ok(z) => z,
        Err(e) => return CompareResult::range_error(
            &unsafe { std::ffi::CStr::from_ptr(e.error_message) }.to_string_lossy()
        ),
    };

    CompareResult::success(zdt_a.epoch_nanoseconds().0.cmp(&zdt_b.epoch_nanoseconds().0) as i32)
}

/// Returns a new ZonedDateTime with updated fields.
#[no_mangle]
pub extern "C" fn temporal_zoned_date_time_with(
    zdt_str: *const c_char,
    year: i32,
    month: i32,
    day: i32,
    hour: i32,
    minute: i32,
    second: i32,
    millisecond: i32,
    microsecond: i32,
    nanosecond: i32,
    offset_ns: i64, // Used for disambiguation if provided
    calendar_id: *const c_char,
    time_zone_id: *const c_char,
) -> TemporalResult {
    let zdt = match parse_zoned_date_time(zdt_str, "zoned date time") {
        Ok(z) => z,
        Err(e) => return e,
    };
    
    // This is complex. `with` works on PlainDateTime components then resolves.
    // We need to implement partial update logic similar to PlainDateTime but then re-resolve.
    // For simplicity, we can extract current components, overlay new ones, create new ZDT.
    
    let current_pdt = zdt.to_plain_date_time();
    
    let new_year = if year == i32::MIN { current_pdt.year() } else { year };
    let new_month = if month == i32::MIN { current_pdt.month() } else { month as u8 };
    let new_day = if day == i32::MIN { current_pdt.day() } else { day as u8 };
    
    let new_hour = if hour == i32::MIN { current_pdt.hour() } else { hour as u8 };
    let new_minute = if minute == i32::MIN { current_pdt.minute() } else { minute as u8 };
    let new_second = if second == i32::MIN { current_pdt.second() } else { second as u8 };
    let new_millisecond = if millisecond == i32::MIN { current_pdt.millisecond() } else { millisecond as u16 };
    let new_microsecond = if microsecond == i32::MIN { current_pdt.microsecond() } else { microsecond as u16 };
    let new_nanosecond = if nanosecond == i32::MIN { current_pdt.nanosecond() } else { nanosecond as u16 };

    let new_calendar = if !calendar_id.is_null() {
        match parse_c_str(calendar_id, "calendar id") {
            Ok(s) => match Calendar::from_str(s) {
                Ok(c) => c,
                Err(e) => return TemporalResult::range_error(&format!("Invalid calendar: {}", e)),
            },
            Err(e) => return e,
        }
    } else {
        zdt.calendar().clone()
    };
    
    let new_timezone = if !time_zone_id.is_null() {
        match parse_c_str(time_zone_id, "timezone id") {
            Ok(s) => match TimeZone::try_from_str(s) {
                Ok(t) => t,
                Err(e) => return TemporalResult::range_error(&format!("Invalid timezone: {}", e)),
            },
            Err(e) => return e,
        }
    } else {
        zdt.time_zone().clone()
    };

    let pdt = match PlainDateTime::new(
        new_year, new_month, new_day, 
        new_hour, new_minute, new_second, 
        new_millisecond, new_microsecond, new_nanosecond, 
        new_calendar
    ) {
        Ok(d) => d,
        Err(e) => return TemporalResult::range_error(&format!("Invalid components: {}", e)),
    };
    
    match pdt.to_zoned_date_time(new_timezone, Disambiguation::Compatible) {
        Ok(new_zdt) => match new_zdt.to_ixdtf_string(DisplayOffset::Auto, DisplayTimeZone::Auto, DisplayCalendar::Auto, ToStringRoundingOptions::default()) {
            Ok(s) => TemporalResult::success(s),
            Err(e) => TemporalResult::range_error(&format!("Failed to format: {}", e)),
        },
        Err(e) => TemporalResult::range_error(&format!("Failed to create zoned date time: {}", e)),
    }
}

/// Computes difference (until).
#[no_mangle]
pub extern "C" fn temporal_zoned_date_time_until(
    one_str: *const c_char,
    two_str: *const c_char,
) -> TemporalResult {
    let one = match parse_zoned_date_time(one_str, "first zoned date time") {
        Ok(z) => z,
        Err(e) => return e,
    };
    let two = match parse_zoned_date_time(two_str, "second zoned date time") {
        Ok(z) => z,
        Err(e) => return e,
    };

    match one.until(&two, Default::default()) {
        Ok(d) => TemporalResult::success(d.to_string()),
        Err(e) => TemporalResult::range_error(&format!("Failed to compute difference: {}", e)),
    }
}

/// Computes difference (since).
#[no_mangle]
pub extern "C" fn temporal_zoned_date_time_since(
    one_str: *const c_char,
    two_str: *const c_char,
) -> TemporalResult {
    let one = match parse_zoned_date_time(one_str, "first zoned date time") {
        Ok(z) => z,
        Err(e) => return e,
    };
    let two = match parse_zoned_date_time(two_str, "second zoned date time") {
        Ok(z) => z,
        Err(e) => return e,
    };

    match one.since(&two, Default::default()) {
        Ok(d) => TemporalResult::success(d.to_string()),
        Err(e) => TemporalResult::range_error(&format!("Failed to compute difference: {}", e)),
    }
}

/// Rounds the ZonedDateTime.
#[no_mangle]
pub extern "C" fn temporal_zoned_date_time_round(
    zdt_str: *const c_char,
    smallest_unit: *const c_char,
    rounding_increment: i64,
    rounding_mode: *const c_char,
) -> TemporalResult {
    let zdt = match parse_zoned_date_time(zdt_str, "zoned date time") {
        Ok(z) => z,
        Err(e) => return e,
    };

    let unit = if !smallest_unit.is_null() {
        let s = match parse_c_str(smallest_unit, "smallest unit") {
            Ok(s) => s,
            Err(e) => return e,
        };
        match Unit::from_str(s) {
            Ok(u) => u,
            Err(_) => return TemporalResult::range_error(&format!("Invalid smallest unit: {}", s)),
        }
    } else {
        return TemporalResult::type_error("smallestUnit is required");
    };

    let mode = if !rounding_mode.is_null() {
        let s = match parse_c_str(rounding_mode, "rounding mode") {
            Ok(s) => s,
            Err(e) => return e,
        };
        match RoundingMode::from_str(s) {
            Ok(m) => m,
            Err(_) => return TemporalResult::range_error(&format!("Invalid rounding mode: {}", s)),
        }
    } else {
        RoundingMode::HalfExpand
    };

    let increment = if rounding_increment > 0 {
        rounding_increment as u32
    } else {
        1
    };
    
    let increment_opt = match RoundingIncrement::try_new(increment) {
        Ok(i) => i,
        Err(e) => return TemporalResult::range_error(&format!("Invalid rounding increment: {}", e)),
    };

    let mut options = RoundingOptions::default();
    options.smallest_unit = Some(unit);
    options.rounding_mode = Some(mode);
    options.increment = Some(increment_opt);

    match zdt.round(options) {
        Ok(result) => match result.to_ixdtf_string(DisplayOffset::Auto, DisplayTimeZone::Auto, DisplayCalendar::Auto, ToStringRoundingOptions::default()) {
            Ok(s) => TemporalResult::success(s),
            Err(e) => TemporalResult::range_error(&format!("Failed to format: {}", e)),
        },
        Err(e) => TemporalResult::range_error(&format!("Failed to round: {}", e)),
    }
}

/// Converts to Instant.
#[no_mangle]
pub extern "C" fn temporal_zoned_date_time_to_instant(s: *const c_char) -> TemporalResult {
    let zdt = match parse_zoned_date_time(s, "zoned date time") {
        Ok(z) => z,
        Err(e) => return e,
    };
    let provider = CompiledTzdbProvider::default();
    match zdt.to_instant().to_ixdtf_string_with_provider(None, Default::default(), &provider) {
        Ok(s) => TemporalResult::success(s),
        Err(e) => TemporalResult::range_error(&format!("Failed to convert to instant: {}", e)),
    }
}

/// Converts to PlainDate.
#[no_mangle]
pub extern "C" fn temporal_zoned_date_time_to_plain_date(s: *const c_char) -> TemporalResult {
    let zdt = match parse_zoned_date_time(s, "zoned date time") {
        Ok(z) => z,
        Err(e) => return e,
    };
    TemporalResult::success(zdt.to_plain_date().to_ixdtf_string(DisplayCalendar::Auto))
}

/// Converts to PlainTime.
#[no_mangle]
pub extern "C" fn temporal_zoned_date_time_to_plain_time(s: *const c_char) -> TemporalResult {
    let zdt = match parse_zoned_date_time(s, "zoned date time") {
        Ok(z) => z,
        Err(e) => return e,
    };
    match zdt.to_plain_time().to_ixdtf_string(ToStringRoundingOptions::default()) {
        Ok(s) => TemporalResult::success(s),
        Err(e) => TemporalResult::range_error(&format!("Failed to convert to plain time: {}", e)),
    }
}

/// Converts to PlainDateTime.
#[no_mangle]
pub extern "C" fn temporal_zoned_date_time_to_plain_date_time(s: *const c_char) -> TemporalResult {
    let zdt = match parse_zoned_date_time(s, "zoned date time") {
        Ok(z) => z,
        Err(e) => return e,
    };
    match zdt.to_plain_date_time().to_ixdtf_string(ToStringRoundingOptions::default(), DisplayCalendar::Auto) {
        Ok(s) => TemporalResult::success(s),
        Err(e) => TemporalResult::range_error(&format!("Failed to convert to plain date time: {}", e)),
    }
}

// Helper functions for ZonedDateTime/TimeZone
fn parse_time_zone(s: *const c_char, param_name: &str) -> Result<TimeZone, TemporalResult> {
    let str_val = parse_c_str(s, param_name)?;
    TimeZone::try_from_str(str_val)
        .map_err(|e| TemporalResult::range_error(&format!("Invalid timezone '{}': {}", str_val, e)))
}

fn parse_zoned_date_time(s: *const c_char, param_name: &str) -> Result<ZonedDateTime, TemporalResult> {
    let str_val = parse_c_str(s, param_name)?;
    ZonedDateTime::from_utf8(str_val.as_bytes(), Disambiguation::Compatible, OffsetDisambiguation::Reject)
        .map_err(|e| TemporalResult::range_error(&format!("Invalid zoned date time '{}': {}", str_val, e)))
}

#[cfg(target_os = "android")]

mod android {
    use jni::objects::{JClass, JString};
    use jni::sys::{jint, jlong, jlongArray, jstring};
    use jni::JNIEnv;

    use super::{
        get_instant_now_string, get_now_plain_date_string, get_now_plain_date_time_string,
        get_now_plain_time_string, get_now_zoned_date_time_string,
    };
    use temporal_rs::{
        options::{DisplayCalendar, ToStringRoundingOptions, Overflow, DisplayOffset, DisplayTimeZone, Disambiguation, OffsetDisambiguation},
        Calendar, Duration, Instant, PlainDate, PlainDateTime, PlainMonthDay, PlainTime,
        PlainYearMonth, TimeZone, ZonedDateTime, TemporalError,
    };
    use std::str::FromStr;
    use std::ptr;

    use timezone_provider::tzif::CompiledTzdbProvider;
    
    const RANGE_ERROR_CLASS: &str = "java/lang/RuntimeException";
    const TYPE_ERROR_CLASS: &str = "java/lang/RuntimeException";

    /// Throws a RangeError exception
    fn throw_range_error(env: &mut JNIEnv, message: &str) {
        let _ = env.throw_new(RANGE_ERROR_CLASS, &format!("[RangeError] {}", message));
    }

    /// Throws a TypeError exception
    fn throw_type_error(env: &mut JNIEnv, message: &str) {
        let _ = env.throw_new(TYPE_ERROR_CLASS, &format!("[TypeError] {}", message));
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

    /// JNI function for `com.temporal.TemporalNative.nowZonedDateTimeISO()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_nowZonedDateTimeISO(
        mut env: JNIEnv,
        _class: JClass,
        tz_id: JString,
    ) -> jstring {
        let tz_str = parse_jstring(&mut env, &tz_id, "timezone id");
        let tz_val = match tz_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        
        match get_now_zoned_date_time_string(&tz_val) {
            Ok(s) => env
                .new_string(s)
                .map(|js| js.into_raw())
                .unwrap_or(ptr::null_mut()),
            Err(e) => {
                throw_range_error(&mut env, &format!("Failed to get zoned date time: {}", e));
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

    /// Parses a PlainDate string, throwing RangeError if invalid
    fn parse_plain_date(env: &mut JNIEnv, s: &JString, name: &str) -> Option<PlainDate> {
        let s_str = parse_jstring(env, s, name)?;
        match PlainDate::from_str(&s_str) {
            Ok(d) => Some(d),
            Err(e) => {
                throw_range_error(env, &format!("Invalid plain date '{}': {}", s_str, e));
                None
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.plainDateFromString()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainDateFromString(
        mut env: JNIEnv,
        _class: JClass,
        s: JString,
    ) -> jstring {
        let date = match parse_plain_date(&mut env, &s, "plain date string") {
            Some(d) => d,
            None => return ptr::null_mut(),
        };
        env.new_string(date.to_ixdtf_string(DisplayCalendar::Auto))
            .map(|js| js.into_raw())
            .unwrap_or_else(|_| {
                throw_range_error(&mut env, "Failed to create result string");
                ptr::null_mut()
            })
    }

    /// JNI function for `com.temporal.TemporalNative.plainDateFromComponents()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainDateFromComponents(
        mut env: JNIEnv,
        _class: JClass,
        year: jint,
        month: jint,
        day: jint,
        calendar_id: JString,
    ) -> jstring {
        let calendar = if !calendar_id.is_null() {
            let id_str = parse_jstring(&mut env, &calendar_id, "calendar id");
            match id_str {
                Some(s) => match Calendar::from_str(&s) {
                    Ok(c) => c,
                    Err(e) => {
                        throw_range_error(&mut env, &format!("Invalid calendar: {}", e));
                        return ptr::null_mut();
                    }
                },
                None => return ptr::null_mut(),
            }
        } else {
            Calendar::default()
        };

        match PlainDate::new(year, month as u8, day as u8, calendar) {
            Ok(date) => env
                .new_string(date.to_ixdtf_string(DisplayCalendar::Auto))
                .map(|js| js.into_raw())
                .unwrap_or_else(|_| {
                    throw_range_error(&mut env, "Failed to create result string");
                    ptr::null_mut()
                }),
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid plain date components: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.plainDateGetAllComponents()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainDateGetAllComponents(
        mut env: JNIEnv,
        _class: JClass,
        s: JString,
    ) -> jlongArray {
        let date = match parse_plain_date(&mut env, &s, "plain date string") {
            Some(d) => d,
            None => return ptr::null_mut(),
        };

        let components: [i64; 12] = [
            date.year() as i64,
            date.month() as i64,
            date.day() as i64,
            date.day_of_week() as i64,
            date.day_of_year() as i64,
            date.week_of_year().unwrap_or(0) as i64,
            date.year_of_week().unwrap_or(0) as i64,
            date.days_in_week() as i64,
            date.days_in_month() as i64,
            date.days_in_year() as i64,
            date.months_in_year() as i64,
            if date.in_leap_year() { 1 } else { 0 },
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

    /// JNI function for `com.temporal.TemporalNative.plainDateGetMonthCode()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainDateGetMonthCode(
        mut env: JNIEnv,
        _class: JClass,
        s: JString,
    ) -> jstring {
        let date = match parse_plain_date(&mut env, &s, "plain date string") {
            Some(d) => d,
            None => return ptr::null_mut(),
        };
        env.new_string(date.month_code().as_str())
            .map(|js| js.into_raw())
            .unwrap_or_else(|_| {
                throw_range_error(&mut env, "Failed to create result string");
                ptr::null_mut()
            })
    }

    /// JNI function for `com.temporal.TemporalNative.plainDateGetCalendar()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainDateGetCalendar(
        mut env: JNIEnv,
        _class: JClass,
        s: JString,
    ) -> jstring {
        let date = match parse_plain_date(&mut env, &s, "plain date string") {
            Some(d) => d,
            None => return ptr::null_mut(),
        };
        env.new_string(date.calendar().identifier())
            .map(|js| js.into_raw())
            .unwrap_or_else(|_| {
                throw_range_error(&mut env, "Failed to create result string");
                ptr::null_mut()
            })
    }

    /// JNI function for `com.temporal.TemporalNative.plainDateAdd()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainDateAdd(
        mut env: JNIEnv,
        _class: JClass,
        date_str: JString,
        duration_str: JString,
    ) -> jstring {
        let date = match parse_plain_date(&mut env, &date_str, "plain date") {
            Some(d) => d,
            None => return ptr::null_mut(),
        };
        let duration = match parse_duration(&mut env, &duration_str, "duration") {
            Some(d) => d,
            None => return ptr::null_mut(),
        };

        match date.add(&duration, None) {
            Ok(result) => env
                .new_string(result.to_ixdtf_string(DisplayCalendar::Auto))
                .map(|js| js.into_raw())
                .unwrap_or_else(|_| {
                    throw_range_error(&mut env, "Failed to create result string");
                    ptr::null_mut()
                }),
            Err(e) => {
                throw_range_error(&mut env, &format!("Failed to add duration: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.plainDateSubtract()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainDateSubtract(
        mut env: JNIEnv,
        _class: JClass,
        date_str: JString,
        duration_str: JString,
    ) -> jstring {
        let date = match parse_plain_date(&mut env, &date_str, "plain date") {
            Some(d) => d,
            None => return ptr::null_mut(),
        };
        let duration = match parse_duration(&mut env, &duration_str, "duration") {
            Some(d) => d,
            None => return ptr::null_mut(),
        };

        match date.subtract(&duration, None) {
            Ok(result) => env
                .new_string(result.to_ixdtf_string(DisplayCalendar::Auto))
                .map(|js| js.into_raw())
                .unwrap_or_else(|_| {
                    throw_range_error(&mut env, "Failed to create result string");
                    ptr::null_mut()
                }),
            Err(e) => {
                throw_range_error(&mut env, &format!("Failed to subtract duration: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.plainDateCompare()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainDateCompare(
        mut env: JNIEnv,
        _class: JClass,
        a: JString,
        b: JString,
    ) -> jint {
        let date_a = match parse_plain_date(&mut env, &a, "first plain date") {
            Some(d) => d,
            None => return 0,
        };
        let date_b = match parse_plain_date(&mut env, &b, "second plain date") {
            Some(d) => d,
            None => return 0,
        };

        // Fallback to string comparison for now
        let s_a = date_a.to_ixdtf_string(DisplayCalendar::Never);
        let s_b = date_b.to_ixdtf_string(DisplayCalendar::Never);

        s_a.cmp(&s_b) as jint
    }

    /// JNI function for `com.temporal.TemporalNative.plainDateWith()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainDateWith(
        mut env: JNIEnv,
        _class: JClass,
        date_str: JString,
        year: jint,
        month: jint,
        day: jint,
        calendar_id: JString,
    ) -> jstring {
        let date = match parse_plain_date(&mut env, &date_str, "plain date") {
            Some(d) => d,
            None => return ptr::null_mut(),
        };

        let new_year = if year == i32::MIN { date.year() } else { year };
        let new_month = if month == i32::MIN { date.month() } else { month as u8 };
        let new_day = if day == i32::MIN { date.day() } else { day as u8 };

        let new_calendar = if !calendar_id.is_null() {
            let id_str = parse_jstring(&mut env, &calendar_id, "calendar id");
            match id_str {
                Some(s) => match Calendar::from_str(&s) {
                    Ok(c) => c,
                    Err(e) => {
                        throw_range_error(&mut env, &format!("Invalid calendar: {}", e));
                        return ptr::null_mut();
                    }
                },
                None => return ptr::null_mut(),
            }
        } else {
            date.calendar().clone()
        };

        match PlainDate::new(new_year, new_month, new_day, new_calendar) {
            Ok(new_date) => env
                .new_string(new_date.to_ixdtf_string(DisplayCalendar::Auto))
                .map(|js| js.into_raw())
                .unwrap_or_else(|_| {
                    throw_range_error(&mut env, "Failed to create result string");
                    ptr::null_mut()
                }),
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid date components: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.plainDateUntil()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainDateUntil(
        mut env: JNIEnv,
        _class: JClass,
        one: JString,
        two: JString,
    ) -> jstring {
        let d1 = match parse_plain_date(&mut env, &one, "first plain date") {
            Some(d) => d,
            None => return ptr::null_mut(),
        };
        let d2 = match parse_plain_date(&mut env, &two, "second plain date") {
            Some(d) => d,
            None => return ptr::null_mut(),
        };

        match d1.until(&d2, Default::default()) {
            Ok(d) => env
                .new_string(d.to_string())
                .map(|js| js.into_raw())
                .unwrap_or_else(|_| {
                    throw_range_error(&mut env, "Failed to create result string");
                    ptr::null_mut()
                }),
            Err(e) => {
                throw_range_error(&mut env, &format!("Failed to compute until: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.plainDateSince()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainDateSince(
        mut env: JNIEnv,
        _class: JClass,
        one: JString,
        two: JString,
    ) -> jstring {
        let d1 = match parse_plain_date(&mut env, &one, "first plain date") {
            Some(d) => d,
            None => return ptr::null_mut(),
        };
        let d2 = match parse_plain_date(&mut env, &two, "second plain date") {
            Some(d) => d,
            None => return ptr::null_mut(),
        };

        match d1.since(&d2, Default::default()) {
            Ok(d) => env
                .new_string(d.to_string())
                .map(|js| js.into_raw())
                .unwrap_or_else(|_| {
                    throw_range_error(&mut env, "Failed to create result string");
                    ptr::null_mut()
                }),
            Err(e) => {
                throw_range_error(&mut env, &format!("Failed to compute since: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.plainDateTimeFromString()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainDateTimeFromString(
        mut env: JNIEnv,
        _class: JClass,
        s: JString,
    ) -> jstring {
        let s_str = parse_jstring(&mut env, &s, "plain date time string");
        let s_val = match s_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        match PlainDateTime::from_str(&s_val) {
            Ok(dt) => match dt.to_ixdtf_string(ToStringRoundingOptions::default(), DisplayCalendar::Auto) {
                Ok(s) => env
                    .new_string(s)
                    .map(|js| js.into_raw())
                    .unwrap_or(ptr::null_mut()),
                Err(e) => {
                    throw_range_error(&mut env, &format!("Failed to format plain date time: {}", e));
                    ptr::null_mut()
                }
            },
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid plain date time '{}': {}", s_val, e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.plainDateTimeFromComponents()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainDateTimeFromComponents(
        mut env: JNIEnv,
        _class: JClass,
        year: jint,
        month: jint,
        day: jint,
        hour: jint,
        minute: jint,
        second: jint,
        millisecond: jint,
        microsecond: jint,
        nanosecond: jint,
        calendar_id: JString,
    ) -> jstring {
        let calendar = if !calendar_id.is_null() {
            let id_str = parse_jstring(&mut env, &calendar_id, "calendar id");
            match id_str {
                Some(s) => match Calendar::from_str(&s) {
                    Ok(c) => c,
                    Err(e) => {
                        throw_range_error(&mut env, &format!("Invalid calendar: {}", e));
                        return ptr::null_mut();
                    }
                },
                None => return ptr::null_mut(),
            }
        } else {
            Calendar::default()
        };

        match PlainDateTime::new(
            year,
            month as u8,
            day as u8,
            hour as u8,
            minute as u8,
            second as u8,
            millisecond as u16,
            microsecond as u16,
            nanosecond as u16,
            calendar
        ) {
            Ok(dt) => match dt.to_ixdtf_string(ToStringRoundingOptions::default(), DisplayCalendar::Auto) {
                Ok(s) => env
                    .new_string(s)
                    .map(|js| js.into_raw())
                    .unwrap_or(ptr::null_mut()),
                Err(e) => {
                    throw_range_error(&mut env, &format!("Failed to format plain date time: {}", e));
                    ptr::null_mut()
                }
            },
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid plain date time components: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.plainDateTimeGetAllComponents()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainDateTimeGetAllComponents(
        mut env: JNIEnv,
        _class: JClass,
        s: JString,
    ) -> jlongArray {
        let s_str = parse_jstring(&mut env, &s, "plain date time string");
        let s_val = match s_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        
        let dt = match PlainDateTime::from_str(&s_val) {
            Ok(d) => d,
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid plain date time: {}", e));
                return ptr::null_mut();
            }
        };

        let components: [i64; 18] = [
            dt.year() as i64,
            dt.month() as i64,
            dt.day() as i64,
            dt.day_of_week() as i64,
            dt.day_of_year() as i64,
            dt.week_of_year().unwrap_or(0) as i64,
            dt.year_of_week().unwrap_or(0) as i64,
            dt.days_in_week() as i64,
            dt.days_in_month() as i64,
            dt.days_in_year() as i64,
            dt.months_in_year() as i64,
            if dt.in_leap_year() { 1 } else { 0 },
            dt.hour() as i64,
            dt.minute() as i64,
            dt.second() as i64,
            dt.millisecond() as i64,
            dt.microsecond() as i64,
            dt.nanosecond() as i64,
        ];

        match env.new_long_array(18) {
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

    /// JNI function for `com.temporal.TemporalNative.plainDateTimeGetMonthCode()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainDateTimeGetMonthCode(
        mut env: JNIEnv,
        _class: JClass,
        s: JString,
    ) -> jstring {
        let s_str = parse_jstring(&mut env, &s, "plain date time string");
        let s_val = match s_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        match PlainDateTime::from_str(&s_val) {
            Ok(dt) => env.new_string(dt.month_code().as_str())
                .map(|js| js.into_raw())
                .unwrap_or(ptr::null_mut()),
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid plain date time: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.plainDateTimeGetCalendar()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainDateTimeGetCalendar(
        mut env: JNIEnv,
        _class: JClass,
        s: JString,
    ) -> jstring {
        let s_str = parse_jstring(&mut env, &s, "plain date time string");
        let s_val = match s_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        match PlainDateTime::from_str(&s_val) {
            Ok(dt) => env.new_string(dt.calendar().identifier())
                .map(|js| js.into_raw())
                .unwrap_or(ptr::null_mut()),
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid plain date time: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.plainDateTimeAdd()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainDateTimeAdd(
        mut env: JNIEnv,
        _class: JClass,
        dt_str: JString,
        duration_str: JString,
    ) -> jstring {
        let dt_s = parse_jstring(&mut env, &dt_str, "plain date time");
        let dt_val = match dt_s {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        let dt = match PlainDateTime::from_str(&dt_val) {
            Ok(d) => d,
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid plain date time: {}", e));
                return ptr::null_mut();
            }
        };

        let dur_s = parse_jstring(&mut env, &duration_str, "duration");
        let dur_val = match dur_s {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        let duration = match Duration::from_str(&dur_val) {
            Ok(d) => d,
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid duration: {}", e));
                return ptr::null_mut();
            }
        };

        match dt.add(&duration, None) {
            Ok(result) => match result.to_ixdtf_string(ToStringRoundingOptions::default(), DisplayCalendar::Auto) {
                Ok(s) => env
                    .new_string(s)
                    .map(|js| js.into_raw())
                    .unwrap_or(ptr::null_mut()),
                Err(e) => {
                    throw_range_error(&mut env, &format!("Failed to format result: {}", e));
                    ptr::null_mut()
                }
            },
            Err(e) => {
                throw_range_error(&mut env, &format!("Failed to add duration: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.plainDateTimeSubtract()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainDateTimeSubtract(
        mut env: JNIEnv,
        _class: JClass,
        dt_str: JString,
        duration_str: JString,
    ) -> jstring {
        let dt_s = parse_jstring(&mut env, &dt_str, "plain date time");
        let dt_val = match dt_s {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        let dt = match PlainDateTime::from_str(&dt_val) {
            Ok(d) => d,
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid plain date time: {}", e));
                return ptr::null_mut();
            }
        };

        let dur_s = parse_jstring(&mut env, &duration_str, "duration");
        let dur_val = match dur_s {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        let duration = match Duration::from_str(&dur_val) {
            Ok(d) => d,
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid duration: {}", e));
                return ptr::null_mut();
            }
        };

        match dt.subtract(&duration, None) {
            Ok(result) => match result.to_ixdtf_string(ToStringRoundingOptions::default(), DisplayCalendar::Auto) {
                Ok(s) => env
                    .new_string(s)
                    .map(|js| js.into_raw())
                    .unwrap_or(ptr::null_mut()),
                Err(e) => {
                    throw_range_error(&mut env, &format!("Failed to format result: {}", e));
                    ptr::null_mut()
                }
            },
            Err(e) => {
                throw_range_error(&mut env, &format!("Failed to subtract duration: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.plainDateTimeCompare()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainDateTimeCompare(
        mut env: JNIEnv,
        _class: JClass,
        a: JString,
        b: JString,
    ) -> jint {
        let a_str = parse_jstring(&mut env, &a, "first plain date time");
        let a_val = match a_str {
            Some(s) => s,
            None => return 0,
        };
        let dt_a = match PlainDateTime::from_str(&a_val) {
            Ok(d) => d,
            Err(_) => return 0,
        };

        let b_str = parse_jstring(&mut env, &b, "second plain date time");
        let b_val = match b_str {
            Some(s) => s,
            None => return 0,
        };
        let dt_b = match PlainDateTime::from_str(&b_val) {
            Ok(d) => d,
            Err(_) => return 0,
        };

        dt_a.compare_iso(&dt_b) as jint
    }

    /// JNI function for `com.temporal.TemporalNative.plainDateTimeWith()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainDateTimeWith(
        mut env: JNIEnv,
        _class: JClass,
        dt_str: JString,
        year: jint,
        month: jint,
        day: jint,
        hour: jint,
        minute: jint,
        second: jint,
        millisecond: jint,
        microsecond: jint,
        nanosecond: jint,
        calendar_id: JString,
    ) -> jstring {
        let s_str = parse_jstring(&mut env, &dt_str, "plain date time");
        let s_val = match s_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        let dt = match PlainDateTime::from_str(&s_val) {
            Ok(d) => d,
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid plain date time: {}", e));
                return ptr::null_mut();
            }
        };

        let new_year = if year == i32::MIN { dt.year() } else { year };
        let new_month = if month == i32::MIN { dt.month() } else { month as u8 };
        let new_day = if day == i32::MIN { dt.day() } else { day as u8 };
        
        let new_hour = if hour == i32::MIN { dt.hour() } else { hour as u8 };
        let new_minute = if minute == i32::MIN { dt.minute() } else { minute as u8 };
        let new_second = if second == i32::MIN { dt.second() } else { second as u8 };
        let new_millisecond = if millisecond == i32::MIN { dt.millisecond() } else { millisecond as u16 };
        let new_microsecond = if microsecond == i32::MIN { dt.microsecond() } else { microsecond as u16 };
        let new_nanosecond = if nanosecond == i32::MIN { dt.nanosecond() } else { nanosecond as u16 };

        let new_calendar = if !calendar_id.is_null() {
            let id_str = parse_jstring(&mut env, &calendar_id, "calendar id");
            match id_str {
                Some(s) => match Calendar::from_str(&s) {
                    Ok(c) => c,
                    Err(e) => {
                        throw_range_error(&mut env, &format!("Invalid calendar: {}", e));
                        return ptr::null_mut();
                    }
                },
                None => return ptr::null_mut(),
            }
        } else {
            dt.calendar().clone()
        };

        match PlainDateTime::new(
            new_year, new_month, new_day,
            new_hour, new_minute, new_second,
            new_millisecond, new_microsecond, new_nanosecond,
            new_calendar
        ) {
             Ok(new_dt) => match new_dt.to_ixdtf_string(ToStringRoundingOptions::default(), DisplayCalendar::Auto) {
                 Ok(s) => env
                    .new_string(s)
                    .map(|js| js.into_raw())
                    .unwrap_or(ptr::null_mut()),
                 Err(e) => {
                     throw_range_error(&mut env, &format!("Failed to format plain date time: {}", e));
                     ptr::null_mut()
                 }
             },
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid date components: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.plainDateTimeUntil()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainDateTimeUntil(
        mut env: JNIEnv,
        _class: JClass,
        one: JString,
        two: JString,
    ) -> jstring {
        let one_str = parse_jstring(&mut env, &one, "first plain date time");
        let one_val = match one_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        let dt1 = match PlainDateTime::from_str(&one_val) {
            Ok(d) => d,
            Err(_) => return ptr::null_mut(),
        };

        let two_str = parse_jstring(&mut env, &two, "second plain date time");
        let two_val = match two_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        let dt2 = match PlainDateTime::from_str(&two_val) {
            Ok(d) => d,
            Err(_) => return ptr::null_mut(),
        };

        match dt1.until(&dt2, Default::default()) {
            Ok(d) => env
                .new_string(d.to_string())
                .map(|js| js.into_raw())
                .unwrap_or(ptr::null_mut()),
            Err(e) => {
                throw_range_error(&mut env, &format!("Failed to compute until: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.plainDateTimeSince()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainDateTimeSince(
        mut env: JNIEnv,
        _class: JClass,
        one: JString,
        two: JString,
    ) -> jstring {
        let one_str = parse_jstring(&mut env, &one, "first plain date time");
        let one_val = match one_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        let dt1 = match PlainDateTime::from_str(&one_val) {
            Ok(d) => d,
            Err(_) => return ptr::null_mut(),
        };

        let two_str = parse_jstring(&mut env, &two, "second plain date time");
        let two_val = match two_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        let dt2 = match PlainDateTime::from_str(&two_val) {
            Ok(d) => d,
            Err(_) => return ptr::null_mut(),
        };

        match dt1.since(&dt2, Default::default()) {
            Ok(d) => env
                .new_string(d.to_string())
                .map(|js| js.into_raw())
                .unwrap_or(ptr::null_mut()),
            Err(e) => {
                throw_range_error(&mut env, &format!("Failed to compute since: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.plainYearMonthFromString()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainYearMonthFromString(
        mut env: JNIEnv,
        _class: JClass,
        s: JString,
    ) -> jstring {
        let s_str = parse_jstring(&mut env, &s, "plain year month string");
        let s_val = match s_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        match PlainYearMonth::from_str(&s_val) {
            Ok(ym) => env.new_string(ym.to_ixdtf_string(DisplayCalendar::Auto))
                .map(|js| js.into_raw())
                .unwrap_or(ptr::null_mut()),
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid plain year month '{}': {}", s_val, e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.plainYearMonthFromComponents()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainYearMonthFromComponents(
        mut env: JNIEnv,
        _class: JClass,
        year: jint,
        month: jint,
        calendar_id: JString,
        _reference_day: jint,
    ) -> jstring {
        let calendar = if !calendar_id.is_null() {
            let id_str = parse_jstring(&mut env, &calendar_id, "calendar id");
            match id_str {
                Some(s) => match Calendar::from_str(&s) {
                    Ok(c) => c,
                    Err(e) => {
                        throw_range_error(&mut env, &format!("Invalid calendar: {}", e));
                        return ptr::null_mut();
                    }
                },
                None => return ptr::null_mut(),
            }
        } else {
            Calendar::default()
        };

        match PlainYearMonth::new(year, month as u8, None, calendar) {
            Ok(ym) => env.new_string(ym.to_ixdtf_string(DisplayCalendar::Auto))
                .map(|js| js.into_raw())
                .unwrap_or(ptr::null_mut()),
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid plain year month components: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.plainYearMonthGetAllComponents()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainYearMonthGetAllComponents(
        mut env: JNIEnv,
        _class: JClass,
        s: JString,
    ) -> jlongArray {
        let s_str = parse_jstring(&mut env, &s, "plain year month string");
        let s_val = match s_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        
        let ym: PlainYearMonth = match PlainYearMonth::from_str(&s_val) {
            Ok(y) => y,
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid plain year month: {}", e));
                return ptr::null_mut();
            }
        };

        let components: [i64; 8] = [
            ym.year() as i64,
            ym.month() as i64,
            0, // PlainYearMonth does not have a day
            ym.days_in_month() as i64,
            ym.days_in_year() as i64,
            ym.months_in_year() as i64,
            if ym.in_leap_year() { 1 } else { 0 },
            ym.era_year().unwrap_or(0) as i64,
        ];

        match env.new_long_array(8) {
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

    /// JNI function for `com.temporal.TemporalNative.plainYearMonthGetMonthCode()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainYearMonthGetMonthCode(
        mut env: JNIEnv,
        _class: JClass,
        s: JString,
    ) -> jstring {
        let s_str = parse_jstring(&mut env, &s, "plain year month string");
        let s_val = match s_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        match PlainYearMonth::from_str(&s_val) {
            Ok(ym) => env.new_string(ym.month_code().as_str())
                .map(|js| js.into_raw())
                .unwrap_or(ptr::null_mut()),
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid plain year month: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.plainYearMonthGetCalendar()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainYearMonthGetCalendar(
        mut env: JNIEnv,
        _class: JClass,
        s: JString,
    ) -> jstring {
        let s_str = parse_jstring(&mut env, &s, "plain year month string");
        let s_val = match s_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        match PlainYearMonth::from_str(&s_val) {
            Ok(ym) => env.new_string(ym.calendar().identifier())
                .map(|js| js.into_raw())
                .unwrap_or(ptr::null_mut()),
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid plain year month: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.plainYearMonthAdd()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainYearMonthAdd(
        mut env: JNIEnv,
        _class: JClass,
        ym_str: JString,
        duration_str: JString,
    ) -> jstring {
        let ym_s = parse_jstring(&mut env, &ym_str, "plain year month");
        let ym_val = match ym_s {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        let ym = match PlainYearMonth::from_str(&ym_val) {
            Ok(y) => y,
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid plain year month: {}", e));
                return ptr::null_mut();
            }
        };

        let dur_s = parse_jstring(&mut env, &duration_str, "duration");
        let dur_val = match dur_s {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        let duration = match Duration::from_str(&dur_val) {
            Ok(d) => d,
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid duration: {}", e));
                return ptr::null_mut();
            }
        };

        match ym.add(&duration, Overflow::Reject) {
            Ok(result) => env.new_string(result.to_ixdtf_string(DisplayCalendar::Auto))
                .map(|js| js.into_raw())
                .unwrap_or(ptr::null_mut()),
            Err(e) => {
                throw_range_error(&mut env, &format!("Failed to add duration: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.plainYearMonthSubtract()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainYearMonthSubtract(
        mut env: JNIEnv,
        _class: JClass,
        ym_str: JString,
        duration_str: JString,
    ) -> jstring {
        let ym_s = parse_jstring(&mut env, &ym_str, "plain year month");
        let ym_val = match ym_s {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        let ym = match PlainYearMonth::from_str(&ym_val) {
            Ok(y) => y,
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid plain year month: {}", e));
                return ptr::null_mut();
            }
        };

        let dur_s = parse_jstring(&mut env, &duration_str, "duration");
        let dur_val = match dur_s {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        let duration = match Duration::from_str(&dur_val) {
            Ok(d) => d,
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid duration: {}", e));
                return ptr::null_mut();
            }
        };

        match ym.subtract(&duration, Overflow::Reject) {
            Ok(result) => env.new_string(result.to_ixdtf_string(DisplayCalendar::Auto))
                .map(|js| js.into_raw())
                .unwrap_or(ptr::null_mut()),
            Err(e) => {
                throw_range_error(&mut env, &format!("Failed to subtract duration: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.plainYearMonthCompare()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainYearMonthCompare(
        mut env: JNIEnv,
        _class: JClass,
        a: JString,
        b: JString,
    ) -> jint {
        let a_str = parse_jstring(&mut env, &a, "first plain year month");
        let a_val = match a_str {
            Some(s) => s,
            None => return 0,
        };
        let ym_a: PlainYearMonth = match PlainYearMonth::from_str(&a_val) {
            Ok(y) => y,
            Err(_) => return 0,
        };

        let b_str = parse_jstring(&mut env, &b, "second plain year month");
        let b_val = match b_str {
            Some(s) => s,
            None => return 0,
        };
        let ym_b: PlainYearMonth = match PlainYearMonth::from_str(&b_val) {
            Ok(y) => y,
            Err(_) => return 0,
        };

        // Fallback to string comparison for now
        let s_a = ym_a.to_ixdtf_string(DisplayCalendar::Never);
        let s_b = ym_b.to_ixdtf_string(DisplayCalendar::Never);

        s_a.cmp(&s_b) as jint
    }

    /// JNI function for `com.temporal.TemporalNative.plainYearMonthWith()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainYearMonthWith(
        mut env: JNIEnv,
        _class: JClass,
        ym_str: JString,
        year: jint,
        month: jint,
        calendar_id: JString,
    ) -> jstring {
        let ym_s = parse_jstring(&mut env, &ym_str, "plain year month");
        let ym_val = match ym_s {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        let ym: PlainYearMonth = match PlainYearMonth::from_str(&ym_val) {
            Ok(y) => y,
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid plain year month: {}", e));
                return ptr::null_mut();
            }
        };

        let new_year = if year == i32::MIN { ym.year() } else { year };
        let new_month = if month == i32::MIN { ym.month() } else { month as u8 };

        let new_calendar = if !calendar_id.is_null() {
            let id_str = parse_jstring(&mut env, &calendar_id, "calendar id");
            match id_str {
                Some(s) => match Calendar::from_str(&s) {
                    Ok(c) => c,
                    Err(e) => {
                        throw_range_error(&mut env, &format!("Invalid calendar: {}", e));
                        return ptr::null_mut();
                    }
                },
                None => return ptr::null_mut(),
            }
        } else {
            ym.calendar().clone()
        };

        match PlainYearMonth::new(new_year, new_month, None, new_calendar) {
            Ok(new_ym) => env.new_string(new_ym.to_ixdtf_string(DisplayCalendar::Auto))
                .map(|js| js.into_raw())
                .unwrap_or(ptr::null_mut()),
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid components: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.plainYearMonthUntil()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainYearMonthUntil(
        mut env: JNIEnv,
        _class: JClass,
        one: JString,
        two: JString,
    ) -> jstring {
        let one_str = parse_jstring(&mut env, &one, "first plain year month");
        let one_val = match one_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        let ym1: PlainYearMonth = match PlainYearMonth::from_str(&one_val) {
            Ok(y) => y,
            Err(_) => return ptr::null_mut(),
        };

        let two_str = parse_jstring(&mut env, &two, "second plain year month");
        let two_val = match two_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        let ym2: PlainYearMonth = match PlainYearMonth::from_str(&two_val) {
            Ok(y) => y,
            Err(_) => return ptr::null_mut(),
        };

        match ym1.until(&ym2, Default::default()) {
            Ok(d) => env.new_string(d.to_string())
                .map(|js| js.into_raw())
                .unwrap_or(ptr::null_mut()),
            Err(e) => {
                throw_range_error(&mut env, &format!("Failed to compute until: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.plainYearMonthSince()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainYearMonthSince(
        mut env: JNIEnv,
        _class: JClass,
        one: JString,
        two: JString,
    ) -> jstring {
        let one_str = parse_jstring(&mut env, &one, "first plain year month");
        let one_val = match one_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        let ym1: PlainYearMonth = match PlainYearMonth::from_str(&one_val) {
            Ok(y) => y,
            Err(_) => return ptr::null_mut(),
        };

        let two_str = parse_jstring(&mut env, &two, "second plain year month");
        let two_val = match two_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        let ym2: PlainYearMonth = match PlainYearMonth::from_str(&two_val) {
            Ok(y) => y,
            Err(_) => return ptr::null_mut(),
        };

        match ym1.since(&ym2, Default::default()) {
            Ok(d) => env.new_string(d.to_string())
                .map(|js| js.into_raw())
                .unwrap_or(ptr::null_mut()),
            Err(e) => {
                throw_range_error(&mut env, &format!("Failed to compute since: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.plainYearMonthToPlainDate()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainYearMonthToPlainDate(
        mut env: JNIEnv,
        _class: JClass,
        ym_str: JString,
        day: jint,
    ) -> jstring {
        let ym_s = parse_jstring(&mut env, &ym_str, "plain year month");
        let ym_val = match ym_s {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        let ym: PlainYearMonth = match PlainYearMonth::from_str(&ym_val) {
            Ok(y) => y,
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid plain year month: {}", e));
                return ptr::null_mut();
            }
        };

        match PlainDate::new(ym.year(), ym.month(), day as u8, ym.calendar().clone()) {
            Ok(d) => env.new_string(d.to_ixdtf_string(DisplayCalendar::Auto))
                .map(|js| js.into_raw())
                .unwrap_or(ptr::null_mut()),
            Err(e) => {
                throw_range_error(&mut env, &format!("Failed to convert to plain date: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.plainMonthDayFromString()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainMonthDayFromString(
        mut env: JNIEnv,
        _class: JClass,
        s: JString,
    ) -> jstring {
        let s_str = parse_jstring(&mut env, &s, "plain month day string");
        let s_val = match s_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        match PlainMonthDay::from_str(&s_val) {
            Ok(md) => env.new_string(md.to_ixdtf_string(DisplayCalendar::Auto))
                .map(|js| js.into_raw())
                .unwrap_or(ptr::null_mut()),
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid plain month day '{}': {}", s_val, e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.plainMonthDayFromComponents()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainMonthDayFromComponents(
        mut env: JNIEnv,
        _class: JClass,
        month: jint,
        day: jint,
        calendar_id: JString,
        _reference_year: jint,
    ) -> jstring {
        let calendar = if !calendar_id.is_null() {
            let id_str = parse_jstring(&mut env, &calendar_id, "calendar id");
            match id_str {
                Some(s) => match Calendar::from_str(&s) {
                    Ok(c) => c,
                    Err(e) => {
                        throw_range_error(&mut env, &format!("Invalid calendar: {}", e));
                        return ptr::null_mut();
                    }
                },
                None => return ptr::null_mut(),
            }
        } else {
            Calendar::default()
        };

        match PlainMonthDay::new_with_overflow(month as u8, day as u8, calendar, Overflow::Reject, None) {
            Ok(md) => env.new_string(md.to_ixdtf_string(DisplayCalendar::Auto))
                .map(|js| js.into_raw())
                .unwrap_or(ptr::null_mut()),
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid plain month day components: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.plainMonthDayGetAllComponents()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainMonthDayGetAllComponents(
        mut env: JNIEnv,
        _class: JClass,
        s: JString,
    ) -> jlongArray {
        let s_str = parse_jstring(&mut env, &s, "plain month day string");
        let s_val = match s_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        
        let md = match PlainMonthDay::from_str(&s_val) {
            Ok(m) => m,
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid plain month day: {}", e));
                return ptr::null_mut();
            }
        };

        let components: [i64; 2] = [
            md.calendar().month(&md.iso) as i64,
            md.day() as i64,
        ];

        match env.new_long_array(2) {
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

    /// JNI function for `com.temporal.TemporalNative.plainMonthDayGetMonthCode()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainMonthDayGetMonthCode(
        mut env: JNIEnv,
        _class: JClass,
        s: JString,
    ) -> jstring {
        let s_str = parse_jstring(&mut env, &s, "plain month day string");
        let s_val = match s_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        match PlainMonthDay::from_str(&s_val) {
            Ok(md) => env.new_string(md.month_code().as_str())
                .map(|js| js.into_raw())
                .unwrap_or(ptr::null_mut()),
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid plain month day: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.plainMonthDayGetCalendar()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainMonthDayGetCalendar(
        mut env: JNIEnv,
        _class: JClass,
        s: JString,
    ) -> jstring {
        let s_str = parse_jstring(&mut env, &s, "plain month day string");
        let s_val = match s_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        match PlainMonthDay::from_str(&s_val) {
            Ok(md) => env.new_string(md.calendar().identifier())
                .map(|js| js.into_raw())
                .unwrap_or(ptr::null_mut()),
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid plain month day: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.plainMonthDayToPlainDate()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_plainMonthDayToPlainDate(
        mut env: JNIEnv,
        _class: JClass,
        md_str: JString,
        year: jint,
    ) -> jstring {
        let md_s = parse_jstring(&mut env, &md_str, "plain month day");
        let md_val = match md_s {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        let md = match PlainMonthDay::from_str(&md_val) {
            Ok(m) => m,
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid plain month day: {}", e));
                return ptr::null_mut();
            }
        };

        match PlainDate::new(year, md.calendar().month(&md.iso), md.day(), md.calendar().clone()) {
            Ok(d) => env.new_string(d.to_ixdtf_string(DisplayCalendar::Auto))
                .map(|js| js.into_raw())
                .unwrap_or(ptr::null_mut()),
            Err(e) => {
                throw_range_error(&mut env, &format!("Failed to convert to plain date: {}", e));
                ptr::null_mut()
            }
        }
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
        env: JNIEnv,
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

    /// JNI function for `com.temporal.TemporalNative.timeZoneFromString()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_timeZoneFromString(
        mut env: JNIEnv,
        _class: JClass,
        s: JString,
    ) -> jstring {
        let s_str = parse_jstring(&mut env, &s, "timezone string");
        let s_val = match s_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        match TimeZone::try_from_str(&s_val) {
            Ok(tz) => match tz.identifier() {
                Ok(id) => env.new_string(id)
                    .map(|js| js.into_raw())
                    .unwrap_or(ptr::null_mut()),
                Err(e) => {
                    throw_range_error(&mut env, &format!("Failed to get timezone id: {}", e));
                    ptr::null_mut()
                }
            },
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid timezone '{}': {}", s_val, e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.timeZoneGetId()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_timeZoneGetId(
        env: JNIEnv,
        _class: JClass,
        s: JString,
    ) -> jstring {
        Java_com_temporal_TemporalNative_timeZoneFromString(env, _class, s)
    }

    /// JNI function for `com.temporal.TemporalNative.timeZoneGetOffsetNanosecondsFor()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_timeZoneGetOffsetNanosecondsFor(
        mut env: JNIEnv,
        _class: JClass,
        tz_id: JString,
        instant_str: JString,
    ) -> jlong {
        let tz_s = parse_jstring(&mut env, &tz_id, "timezone");
        let tz_val = match tz_s {
            Some(s) => s,
            None => return 0,
        };
        let tz = match TimeZone::try_from_str(&tz_val) {
            Ok(t) => t,
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid timezone: {}", e));
                return 0;
            }
        };

        let inst_s = parse_jstring(&mut env, &instant_str, "instant");
        let inst_val = match inst_s {
            Some(s) => s,
            None => return 0,
        };
        let instant = match Instant::from_str(&inst_val) {
            Ok(i) => i,
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid instant: {}", e));
                return 0;
            }
        };

        match ZonedDateTime::try_new(instant.epoch_nanoseconds().0, tz, Calendar::default()) {
            Ok(zdt) => zdt.offset_nanoseconds() as jlong,
            Err(e) => {
                throw_range_error(&mut env, &format!("Failed to get offset: {}", e));
                0
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.timeZoneGetOffsetStringFor()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_timeZoneGetOffsetStringFor(
        mut env: JNIEnv,
        _class: JClass,
        tz_id: JString,
        instant_str: JString,
    ) -> jstring {
        let tz_s = parse_jstring(&mut env, &tz_id, "timezone");
        let tz_val = match tz_s {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        let tz = match TimeZone::try_from_str(&tz_val) {
            Ok(t) => t,
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid timezone: {}", e));
                return ptr::null_mut();
            }
        };

        let inst_s = parse_jstring(&mut env, &instant_str, "instant");
        let inst_val = match inst_s {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        let instant = match Instant::from_str(&inst_val) {
            Ok(i) => i,
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid instant: {}", e));
                return ptr::null_mut();
            }
        };

        match ZonedDateTime::try_new(instant.epoch_nanoseconds().0, tz, Calendar::default()) {
            Ok(zdt) => env.new_string(zdt.offset().to_string())
                .map(|js| js.into_raw())
                .unwrap_or(ptr::null_mut()),
            Err(e) => {
                throw_range_error(&mut env, &format!("Failed to get offset string: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.timeZoneGetPlainDateTimeFor()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_timeZoneGetPlainDateTimeFor(
        mut env: JNIEnv,
        _class: JClass,
        tz_id: JString,
        instant_str: JString,
        calendar_id: JString,
    ) -> jstring {
        let tz_s = parse_jstring(&mut env, &tz_id, "timezone");
        let tz_val = match tz_s {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        let tz = match TimeZone::try_from_str(&tz_val) {
            Ok(t) => t,
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid timezone: {}", e));
                return ptr::null_mut();
            }
        };

        let inst_s = parse_jstring(&mut env, &instant_str, "instant");
        let inst_val = match inst_s {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        let instant = match Instant::from_str(&inst_val) {
            Ok(i) => i,
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid instant: {}", e));
                return ptr::null_mut();
            }
        };

        let calendar = if !calendar_id.is_null() {
            let id_str = parse_jstring(&mut env, &calendar_id, "calendar id");
            match id_str {
                Some(s) => match Calendar::from_str(&s) {
                    Ok(c) => c,
                    Err(e) => {
                        throw_range_error(&mut env, &format!("Invalid calendar: {}", e));
                        return ptr::null_mut();
                    }
                },
                None => return ptr::null_mut(),
            }
        } else {
            Calendar::default()
        };

        match ZonedDateTime::try_new(instant.epoch_nanoseconds().0, tz, calendar) {
            Ok(zdt) => {
                let dt = zdt.to_plain_date_time();
                match dt.to_ixdtf_string(ToStringRoundingOptions::default(), DisplayCalendar::Auto) {
                    Ok(s) => env.new_string(s)
                        .map(|js| js.into_raw())
                        .unwrap_or(ptr::null_mut()),
                    Err(e) => {
                        throw_range_error(&mut env, &format!("Failed to format plain date time: {}", e));
                        ptr::null_mut()
                    }
                }
            },
            Err(e) => {
                throw_range_error(&mut env, &format!("Failed to get plain date time: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.timeZoneGetInstantFor()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_timeZoneGetInstantFor(
        mut env: JNIEnv,
        _class: JClass,
        tz_id: JString,
        dt_str: JString,
        disambiguation: JString,
    ) -> jstring {
        let tz_s = parse_jstring(&mut env, &tz_id, "timezone");
        let tz_val = match tz_s {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        let tz = match TimeZone::try_from_str(&tz_val) {
            Ok(t) => t,
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid timezone: {}", e));
                return ptr::null_mut();
            }
        };

        let dt_s = parse_jstring(&mut env, &dt_str, "plain date time");
        let dt_val = match dt_s {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        let dt = match PlainDateTime::from_str(&dt_val) {
            Ok(d) => d,
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid plain date time: {}", e));
                return ptr::null_mut();
            }
        };

        // Disambiguation handling... assumes Compatible default or parse string
        let disambig_enum = if !disambiguation.is_null() {
            match parse_jstring(&mut env, &disambiguation, "disambiguation") {
                Some(s) => match s.as_str() {
                    "compatible" => Disambiguation::Compatible,
                    "earlier" => Disambiguation::Earlier,
                    "later" => Disambiguation::Later,
                    "reject" => Disambiguation::Reject,
                    _ => Disambiguation::Compatible,
                },
                None => return ptr::null_mut(),
            }
        } else {
            Disambiguation::Compatible
        };

        match dt.to_zoned_date_time(tz, disambig_enum) {
            Ok(zdt) => {
                let instant = zdt.to_instant();
                let provider = CompiledTzdbProvider::default();
                match instant.to_ixdtf_string_with_provider(None, Default::default(), &provider) {
                    Ok(s) => env.new_string(s)
                        .map(|js| js.into_raw())
                        .unwrap_or(ptr::null_mut()),
                    Err(e) => {
                        throw_range_error(&mut env, &format!("Failed to format instant: {}", e));
                        ptr::null_mut()
                    }
                }
            },
            Err(e) => {
                throw_range_error(&mut env, &format!("Failed to get instant: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.timeZoneGetNextTransition()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_timeZoneGetNextTransition(
        mut env: JNIEnv,
        _class: JClass,
        tz_id: JString,
        instant_str: JString,
    ) -> jstring {
        let tz_s = parse_jstring(&mut env, &tz_id, "timezone");
        let tz_val = match tz_s {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        let tz = match TimeZone::try_from_str(&tz_val) {
            Ok(t) => t,
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid timezone: {}", e));
                return ptr::null_mut();
            }
        };

        let inst_s = parse_jstring(&mut env, &instant_str, "instant");
        let inst_val = match inst_s {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        let instant = match Instant::from_str(&inst_val) {
            Ok(i) => i,
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid instant: {}", e));
                return ptr::null_mut();
            }
        };

        // TODO: Implement using provider directly
        match Ok::<Option<Instant>, TemporalError>(None) {
            Ok(Some(i)) => {
                let provider = CompiledTzdbProvider::default();
                match i.to_ixdtf_string_with_provider(None, Default::default(), &provider) {
                    Ok(s) => env.new_string(s)
                        .map(|js| js.into_raw())
                        .unwrap_or(ptr::null_mut()),
                    Err(e) => {
                        throw_range_error(&mut env, &format!("Failed to format instant: {}", e));
                        ptr::null_mut()
                    }
                }
            },
            Ok(None) => ptr::null_mut(), // Return null
            Err(e) => {
                throw_range_error(&mut env, &format!("Failed to get next transition: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.timeZoneGetPreviousTransition()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_timeZoneGetPreviousTransition(
        mut env: JNIEnv,
        _class: JClass,
        tz_id: JString,
        instant_str: JString,
    ) -> jstring {
        let tz_s = parse_jstring(&mut env, &tz_id, "timezone");
        let tz_val = match tz_s {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        let tz = match TimeZone::try_from_str(&tz_val) {
            Ok(t) => t,
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid timezone: {}", e));
                return ptr::null_mut();
            }
        };

        let inst_s = parse_jstring(&mut env, &instant_str, "instant");
        let inst_val = match inst_s {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        let instant = match Instant::from_str(&inst_val) {
            Ok(i) => i,
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid instant: {}", e));
                return ptr::null_mut();
            }
        };

        // TODO: Implement using provider directly
        match Ok::<Option<Instant>, TemporalError>(None) {
            Ok(Some(i)) => {
                let provider = CompiledTzdbProvider::default();
                match i.to_ixdtf_string_with_provider(None, Default::default(), &provider) {
                    Ok(s) => env.new_string(s)
                        .map(|js| js.into_raw())
                        .unwrap_or(ptr::null_mut()),
                    Err(e) => {
                        throw_range_error(&mut env, &format!("Failed to format instant: {}", e));
                        ptr::null_mut()
                    }
                }
            },
            Ok(None) => ptr::null_mut(),
            Err(e) => {
                throw_range_error(&mut env, &format!("Failed to get previous transition: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.zonedDateTimeFromString()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_zonedDateTimeFromString(
        mut env: JNIEnv,
        _class: JClass,
        s: JString,
    ) -> jstring {
        let s_str = parse_jstring(&mut env, &s, "zoned date time string");
        let s_val = match s_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        
        match ZonedDateTime::from_utf8(s_val.as_bytes(), Disambiguation::Compatible, OffsetDisambiguation::Reject) {
            Ok(zdt) => match zdt.to_ixdtf_string(DisplayOffset::Auto, DisplayTimeZone::Auto, DisplayCalendar::Auto, ToStringRoundingOptions::default()) {
                Ok(s) => env.new_string(s)
                    .map(|js| js.into_raw())
                    .unwrap_or(ptr::null_mut()),
                Err(e) => {
                    throw_range_error(&mut env, &format!("Failed to format zoned date time: {}", e));
                    ptr::null_mut()
                }
            },
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid zoned date time '{}': {}", s_val, e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.zonedDateTimeFromComponents()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_zonedDateTimeFromComponents(
        mut env: JNIEnv,
        _class: JClass,
        year: jint,
        month: jint,
        day: jint,
        hour: jint,
        minute: jint,
        second: jint,
        millisecond: jint,
        microsecond: jint,
        nanosecond: jint,
        calendar_id: JString,
        time_zone_id: JString,
        offset_nanoseconds: jlong,
    ) -> jstring {
        let calendar = if !calendar_id.is_null() {
            let id_str = parse_jstring(&mut env, &calendar_id, "calendar id");
            match id_str {
                Some(s) => match Calendar::from_str(&s) {
                    Ok(c) => c,
                    Err(e) => {
                        throw_range_error(&mut env, &format!("Invalid calendar: {}", e));
                        return ptr::null_mut();
                    }
                },
                None => return ptr::null_mut(),
            }
        } else {
            Calendar::default()
        };

        let pdt = match PlainDateTime::new(
            year, month as u8, day as u8, 
            hour as u8, minute as u8, second as u8, 
            millisecond as u16, microsecond as u16, nanosecond as u16, 
            calendar
        ) {
            Ok(d) => d,
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid components: {}", e));
                return ptr::null_mut();
            }
        };

        let tz_s = parse_jstring(&mut env, &time_zone_id, "timezone id");
        let tz_val = match tz_s {
            Some(s) => s,
            None => {
                throw_type_error(&mut env, "Timezone ID is required");
                return ptr::null_mut();
            }
        };

        let tz = match TimeZone::try_from_str(&tz_val) {
            Ok(t) => t,
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid timezone: {}", e));
                return ptr::null_mut();
            }
        };

        match pdt.to_zoned_date_time(tz, Disambiguation::Compatible) {
            Ok(zdt) => match zdt.to_ixdtf_string(DisplayOffset::Auto, DisplayTimeZone::Auto, DisplayCalendar::Auto, ToStringRoundingOptions::default()) {
                Ok(s) => env.new_string(s)
                    .map(|js| js.into_raw())
                    .unwrap_or(ptr::null_mut()),
                Err(e) => {
                    throw_range_error(&mut env, &format!("Failed to format zoned date time: {}", e));
                    ptr::null_mut()
                }
            },
            Err(e) => {
                throw_range_error(&mut env, &format!("Failed to create zoned date time: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.zonedDateTimeGetAllComponents()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_zonedDateTimeGetAllComponents(
        mut env: JNIEnv,
        _class: JClass,
        s: JString,
    ) -> jlongArray {
        let s_str = parse_jstring(&mut env, &s, "zoned date time string");
        let s_val = match s_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        
        // Use default provider
        let zdt = match ZonedDateTime::from_utf8(s_val.as_bytes(), Disambiguation::Compatible, OffsetDisambiguation::Reject) {
            Ok(z) => z,
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid zoned date time: {}", e));
                return ptr::null_mut();
            }
        };

        let components: [i64; 19] = [
            zdt.year() as i64,
            zdt.month() as i64,
            zdt.day() as i64,
            zdt.day_of_week() as i64,
            zdt.day_of_year() as i64,
            zdt.week_of_year().unwrap_or(0) as i64,
            zdt.year_of_week().unwrap_or(0) as i64,
            zdt.days_in_week() as i64,
            zdt.days_in_month() as i64,
            zdt.days_in_year() as i64,
            zdt.months_in_year() as i64,
            if zdt.in_leap_year() { 1 } else { 0 },
            zdt.hour() as i64,
            zdt.minute() as i64,
            zdt.second() as i64,
            zdt.millisecond() as i64,
            zdt.microsecond() as i64,
            zdt.nanosecond() as i64,
            zdt.offset_nanoseconds() as i64,
        ];

        match env.new_long_array(19) {
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

    /// JNI function for `com.temporal.TemporalNative.zonedDateTimeEpochMilliseconds()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_zonedDateTimeEpochMilliseconds(
        mut env: JNIEnv,
        _class: JClass,
        s: JString,
    ) -> jstring {
        let s_str = parse_jstring(&mut env, &s, "zoned date time string");
        let s_val = match s_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        let zdt = match ZonedDateTime::from_utf8(s_val.as_bytes(), Disambiguation::Compatible, OffsetDisambiguation::Reject) {
            Ok(z) => z,
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid zoned date time: {}", e));
                return ptr::null_mut();
            }
        };
        env.new_string(zdt.epoch_milliseconds().to_string())
            .map(|js| js.into_raw())
            .unwrap_or(ptr::null_mut())
    }

    /// JNI function for `com.temporal.TemporalNative.zonedDateTimeEpochNanoseconds()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_zonedDateTimeEpochNanoseconds(
        mut env: JNIEnv,
        _class: JClass,
        s: JString,
    ) -> jstring {
        let s_str = parse_jstring(&mut env, &s, "zoned date time string");
        let s_val = match s_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        let zdt = match ZonedDateTime::from_utf8(s_val.as_bytes(), Disambiguation::Compatible, OffsetDisambiguation::Reject) {
            Ok(z) => z,
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid zoned date time: {}", e));
                return ptr::null_mut();
            }
        };
        env.new_string(zdt.epoch_nanoseconds().0.to_string())
            .map(|js| js.into_raw())
            .unwrap_or(ptr::null_mut())
    }

    /// JNI function for `com.temporal.TemporalNative.zonedDateTimeGetCalendar()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_zonedDateTimeGetCalendar(
        mut env: JNIEnv,
        _class: JClass,
        s: JString,
    ) -> jstring {
        let s_str = parse_jstring(&mut env, &s, "zoned date time string");
        let s_val = match s_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        match ZonedDateTime::from_utf8(s_val.as_bytes(), Disambiguation::Compatible, OffsetDisambiguation::Reject) {
            Ok(z) => env.new_string(z.calendar().identifier())
                .map(|js| js.into_raw())
                .unwrap_or(ptr::null_mut()),
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid zoned date time: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.zonedDateTimeGetTimeZone()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_zonedDateTimeGetTimeZone(
        mut env: JNIEnv,
        _class: JClass,
        s: JString,
    ) -> jstring {
        let s_str = parse_jstring(&mut env, &s, "zoned date time string");
        let s_val = match s_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        match ZonedDateTime::from_utf8(s_val.as_bytes(), Disambiguation::Compatible, OffsetDisambiguation::Reject) {
            Ok(z) => match z.time_zone().identifier() {
                Ok(id) => env.new_string(id)
                    .map(|js| js.into_raw())
                    .unwrap_or(ptr::null_mut()),
                Err(e) => {
                    throw_range_error(&mut env, &format!("Failed to get identifier: {}", e));
                    ptr::null_mut()
                }
            },
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid zoned date time: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.zonedDateTimeGetOffset()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_zonedDateTimeGetOffset(
        mut env: JNIEnv,
        _class: JClass,
        s: JString,
    ) -> jstring {
        let s_str = parse_jstring(&mut env, &s, "zoned date time string");
        let s_val = match s_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        match ZonedDateTime::from_utf8(s_val.as_bytes(), Disambiguation::Compatible, OffsetDisambiguation::Reject) {
            Ok(z) => env.new_string(z.offset().to_string())
                .map(|js| js.into_raw())
                .unwrap_or(ptr::null_mut()),
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid zoned date time: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.zonedDateTimeAdd()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_zonedDateTimeAdd(
        mut env: JNIEnv,
        _class: JClass,
        zdt_str: JString,
        duration_str: JString,
    ) -> jstring {
        let zdt_s = parse_jstring(&mut env, &zdt_str, "zoned date time");
        let zdt_val = match zdt_s {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        let zdt = match ZonedDateTime::from_utf8(zdt_val.as_bytes(), Disambiguation::Compatible, OffsetDisambiguation::Reject) {
            Ok(z) => z,
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid zoned date time: {}", e));
                return ptr::null_mut();
            }
        };

        let dur_s = parse_jstring(&mut env, &duration_str, "duration");
        let dur_val = match dur_s {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        let duration = match Duration::from_str(&dur_val) {
            Ok(d) => d,
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid duration: {}", e));
                return ptr::null_mut();
            }
        };

        match zdt.add(&duration, Some(Overflow::Reject)) {
            Ok(result) => match result.to_ixdtf_string(DisplayOffset::Auto, DisplayTimeZone::Auto, DisplayCalendar::Auto, ToStringRoundingOptions::default()) {
                Ok(s) => env.new_string(s)
                    .map(|js| js.into_raw())
                    .unwrap_or(ptr::null_mut()),
                Err(e) => {
                    throw_range_error(&mut env, &format!("Failed to format result: {}", e));
                    ptr::null_mut()
                }
            },
            Err(e) => {
                throw_range_error(&mut env, &format!("Failed to add duration: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.zonedDateTimeSubtract()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_zonedDateTimeSubtract(
        mut env: JNIEnv,
        _class: JClass,
        zdt_str: JString,
        duration_str: JString,
    ) -> jstring {
        let zdt_s = parse_jstring(&mut env, &zdt_str, "zoned date time");
        let zdt_val = match zdt_s {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        let zdt = match ZonedDateTime::from_utf8(zdt_val.as_bytes(), Disambiguation::Compatible, OffsetDisambiguation::Reject) {
            Ok(z) => z,
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid zoned date time: {}", e));
                return ptr::null_mut();
            }
        };

        let dur_s = parse_jstring(&mut env, &duration_str, "duration");
        let dur_val = match dur_s {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        let duration = match Duration::from_str(&dur_val) {
            Ok(d) => d,
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid duration: {}", e));
                return ptr::null_mut();
            }
        };

        match zdt.subtract(&duration, Some(Overflow::Reject)) {
            Ok(result) => match result.to_ixdtf_string(DisplayOffset::Auto, DisplayTimeZone::Auto, DisplayCalendar::Auto, ToStringRoundingOptions::default()) {
                Ok(s) => env.new_string(s)
                    .map(|js| js.into_raw())
                    .unwrap_or(ptr::null_mut()),
                Err(e) => {
                    throw_range_error(&mut env, &format!("Failed to format result: {}", e));
                    ptr::null_mut()
                }
            },
            Err(e) => {
                throw_range_error(&mut env, &format!("Failed to subtract duration: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.zonedDateTimeCompare()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_zonedDateTimeCompare(
        mut env: JNIEnv,
        _class: JClass,
        a: JString,
        b: JString,
    ) -> jint {
        let a_str = parse_jstring(&mut env, &a, "first zoned date time");
        let a_val = match a_str {
            Some(s) => s,
            None => return 0,
        };
        let zdt_a = match ZonedDateTime::from_utf8(a_val.as_bytes(), Disambiguation::Compatible, OffsetDisambiguation::Reject) {
            Ok(z) => z,
            Err(_) => return 0,
        };

        let b_str = parse_jstring(&mut env, &b, "second zoned date time");
        let b_val = match b_str {
            Some(s) => s,
            None => return 0,
        };
        let zdt_b = match ZonedDateTime::from_utf8(b_val.as_bytes(), Disambiguation::Compatible, OffsetDisambiguation::Reject) {
            Ok(z) => z,
            Err(_) => return 0,
        };

        zdt_a.epoch_nanoseconds().0.cmp(&zdt_b.epoch_nanoseconds().0) as jint
    }

    /// JNI function for `com.temporal.TemporalNative.zonedDateTimeWith()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_zonedDateTimeWith(
        mut env: JNIEnv,
        _class: JClass,
        zdt_str: JString,
        year: jint,
        month: jint,
        day: jint,
        hour: jint,
        minute: jint,
        second: jint,
        millisecond: jint,
        microsecond: jint,
        nanosecond: jint,
        _offset_ns: jlong,
        calendar_id: JString,
        time_zone_id: JString,
    ) -> jstring {
        let zdt_s = parse_jstring(&mut env, &zdt_str, "zoned date time");
        let zdt_val = match zdt_s {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        let zdt = match ZonedDateTime::from_utf8(zdt_val.as_bytes(), Disambiguation::Compatible, OffsetDisambiguation::Reject) {
            Ok(z) => z,
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid zoned date time: {}", e));
                return ptr::null_mut();
            }
        };
        
        let current_pdt = zdt.to_plain_date_time();
    
        let new_year = if year == i32::MIN { current_pdt.year() } else { year };
        let new_month = if month == i32::MIN { current_pdt.month() } else { month as u8 };
        let new_day = if day == i32::MIN { current_pdt.day() } else { day as u8 };
        
        let new_hour = if hour == i32::MIN { current_pdt.hour() } else { hour as u8 };
        let new_minute = if minute == i32::MIN { current_pdt.minute() } else { minute as u8 };
        let new_second = if second == i32::MIN { current_pdt.second() } else { second as u8 };
        let new_millisecond = if millisecond == i32::MIN { current_pdt.millisecond() } else { millisecond as u16 };
        let new_microsecond = if microsecond == i32::MIN { current_pdt.microsecond() } else { microsecond as u16 };
        let new_nanosecond = if nanosecond == i32::MIN { current_pdt.nanosecond() } else { nanosecond as u16 };

        let new_calendar = if !calendar_id.is_null() {
            let id_str = parse_jstring(&mut env, &calendar_id, "calendar id");
            match id_str {
                Some(s) => match Calendar::from_str(&s) {
                    Ok(c) => c,
                    Err(e) => {
                        throw_range_error(&mut env, &format!("Invalid calendar: {}", e));
                        return ptr::null_mut();
                    }
                },
                None => return ptr::null_mut(),
            }
        } else {
            zdt.calendar().clone()
        };
        
        let new_timezone = if !time_zone_id.is_null() {
            let id_str = parse_jstring(&mut env, &time_zone_id, "timezone id");
            match id_str {
                Some(s) => match TimeZone::try_from_str(&s) {
                    Ok(t) => t,
                    Err(e) => {
                        throw_range_error(&mut env, &format!("Invalid timezone: {}", e));
                        return ptr::null_mut();
                    }
                },
                None => return ptr::null_mut(),
            }
        } else {
            zdt.time_zone().clone()
        };

        let pdt = match PlainDateTime::new(
            new_year, new_month, new_day, 
            new_hour, new_minute, new_second, 
            new_millisecond, new_microsecond, new_nanosecond, 
            new_calendar
        ) {
            Ok(d) => d,
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid components: {}", e));
                return ptr::null_mut();
            }
        };
        
        match pdt.to_zoned_date_time(new_timezone, Disambiguation::Compatible) {
            Ok(new_zdt) => match new_zdt.to_ixdtf_string(DisplayOffset::Auto, DisplayTimeZone::Auto, DisplayCalendar::Auto, ToStringRoundingOptions::default()) {
                Ok(s) => env.new_string(s)
                    .map(|js| js.into_raw())
                    .unwrap_or(ptr::null_mut()),
                Err(e) => {
                    throw_range_error(&mut env, &format!("Failed to format: {}", e));
                    ptr::null_mut()
                }
            },
            Err(e) => {
                throw_range_error(&mut env, &format!("Failed to create zoned date time: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.zonedDateTimeUntil()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_zonedDateTimeUntil(
        mut env: JNIEnv,
        _class: JClass,
        one: JString,
        two: JString,
    ) -> jstring {
        let one_str = parse_jstring(&mut env, &one, "first zoned date time");
        let one_val = match one_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        let zdt1 = match ZonedDateTime::from_utf8(one_val.as_bytes(), Disambiguation::Compatible, OffsetDisambiguation::Reject) {
            Ok(z) => z,
            Err(_) => return ptr::null_mut(),
        };

        let two_str = parse_jstring(&mut env, &two, "second zoned date time");
        let two_val = match two_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        let zdt2 = match ZonedDateTime::from_utf8(two_val.as_bytes(), Disambiguation::Compatible, OffsetDisambiguation::Reject) {
            Ok(z) => z,
            Err(_) => return ptr::null_mut(),
        };

        match zdt1.until(&zdt2, Default::default()) {
            Ok(d) => env.new_string(d.to_string())
                .map(|js| js.into_raw())
                .unwrap_or(ptr::null_mut()),
            Err(e) => {
                throw_range_error(&mut env, &format!("Failed to compute until: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.zonedDateTimeSince()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_zonedDateTimeSince(
        mut env: JNIEnv,
        _class: JClass,
        one: JString,
        two: JString,
    ) -> jstring {
        let one_str = parse_jstring(&mut env, &one, "first zoned date time");
        let one_val = match one_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        let zdt1 = match ZonedDateTime::from_utf8(one_val.as_bytes(), Disambiguation::Compatible, OffsetDisambiguation::Reject) {
            Ok(z) => z,
            Err(_) => return ptr::null_mut(),
        };

        let two_str = parse_jstring(&mut env, &two, "second zoned date time");
        let two_val = match two_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        let zdt2 = match ZonedDateTime::from_utf8(two_val.as_bytes(), Disambiguation::Compatible, OffsetDisambiguation::Reject) {
            Ok(z) => z,
            Err(_) => return ptr::null_mut(),
        };

        match zdt1.since(&zdt2, Default::default()) {
            Ok(d) => env.new_string(d.to_string())
                .map(|js| js.into_raw())
                .unwrap_or(ptr::null_mut()),
            Err(e) => {
                throw_range_error(&mut env, &format!("Failed to compute since: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.zonedDateTimeToInstant()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_zonedDateTimeToInstant(
        mut env: JNIEnv,
        _class: JClass,
        s: JString,
    ) -> jstring {
        let s_str = parse_jstring(&mut env, &s, "zoned date time string");
        let s_val = match s_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        match ZonedDateTime::from_utf8(s_val.as_bytes(), Disambiguation::Compatible, OffsetDisambiguation::Reject) {
            Ok(zdt) => {
                let provider = CompiledTzdbProvider::default();
                match zdt.to_instant().to_ixdtf_string_with_provider(None, Default::default(), &provider) {
                    Ok(s) => env.new_string(s)
                        .map(|js| js.into_raw())
                        .unwrap_or(ptr::null_mut()),
                    Err(e) => {
                        throw_range_error(&mut env, &format!("Failed to format instant: {}", e));
                        ptr::null_mut()
                    }
                }
            },
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid zoned date time: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.zonedDateTimeToPlainDate()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_zonedDateTimeToPlainDate(
        mut env: JNIEnv,
        _class: JClass,
        s: JString,
    ) -> jstring {
        let s_str = parse_jstring(&mut env, &s, "zoned date time string");
        let s_val = match s_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        match ZonedDateTime::from_utf8(s_val.as_bytes(), Disambiguation::Compatible, OffsetDisambiguation::Reject) {
            Ok(zdt) => env.new_string(zdt.to_plain_date().to_ixdtf_string(DisplayCalendar::Auto))
                .map(|js| js.into_raw())
                .unwrap_or(ptr::null_mut()),
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid zoned date time: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.zonedDateTimeToPlainTime()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_zonedDateTimeToPlainTime(
        mut env: JNIEnv,
        _class: JClass,
        s: JString,
    ) -> jstring {
        let s_str = parse_jstring(&mut env, &s, "zoned date time string");
        let s_val = match s_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        match ZonedDateTime::from_utf8(s_val.as_bytes(), Disambiguation::Compatible, OffsetDisambiguation::Reject) {
            Ok(zdt) => match zdt.to_plain_time().to_ixdtf_string(ToStringRoundingOptions::default()) {
                Ok(s) => env.new_string(s)
                    .map(|js| js.into_raw())
                    .unwrap_or(ptr::null_mut()),
                Err(e) => {
                    throw_range_error(&mut env, &format!("Failed to format plain time: {}", e));
                    ptr::null_mut()
                }
            },
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid zoned date time: {}", e));
                ptr::null_mut()
            }
        }
    }

    /// JNI function for `com.temporal.TemporalNative.zonedDateTimeToPlainDateTime()`
    #[no_mangle]
    pub extern "system" fn Java_com_temporal_TemporalNative_zonedDateTimeToPlainDateTime(
        mut env: JNIEnv,
        _class: JClass,
        s: JString,
    ) -> jstring {
        let s_str = parse_jstring(&mut env, &s, "zoned date time string");
        let s_val = match s_str {
            Some(s) => s,
            None => return ptr::null_mut(),
        };
        match ZonedDateTime::from_utf8(s_val.as_bytes(), Disambiguation::Compatible, OffsetDisambiguation::Reject) {
            Ok(zdt) => match zdt.to_plain_date_time().to_ixdtf_string(ToStringRoundingOptions::default(), DisplayCalendar::Auto) {
                Ok(s) => env.new_string(s)
                    .map(|js| js.into_raw())
                    .unwrap_or(ptr::null_mut()),
                Err(e) => {
                    throw_range_error(&mut env, &format!("Failed to format plain date time: {}", e));
                    ptr::null_mut()
                }
            },
            Err(e) => {
                throw_range_error(&mut env, &format!("Invalid zoned date time: {}", e));
                ptr::null_mut()
            }
        }
    }
}

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

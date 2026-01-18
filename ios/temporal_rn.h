/* temporal-rn C bindings */
#ifndef TEMPORAL_RN_H
#define TEMPORAL_RN_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// ============================================================================
// Error Types (matching TC39 Temporal)
// ============================================================================

/**
 * Error types matching TC39 Temporal specification.
 */
typedef enum {
    TEMPORAL_ERROR_NONE = 0,       // No error
    TEMPORAL_ERROR_RANGE = 1,      // RangeError - value out of range or invalid format
    TEMPORAL_ERROR_TYPE = 2,       // TypeError - wrong type or invalid argument
} TemporalErrorType;

/**
 * Result structure for operations that can fail.
 */
typedef struct {
    char *value;           // Result value (NULL if error)
    int32_t error_type;    // Error type (0 = success)
    char *error_message;   // Error message (NULL if success)
} TemporalResult;

/**
 * Compare result structure for duration comparison.
 */
typedef struct {
    int32_t value;         // -1, 0, or 1
    int32_t error_type;    // Error type (0 = success)
    char *error_message;   // Error message (NULL if success)
} CompareResult;

/**
 * Frees a TemporalResult's allocated strings.
 */
void temporal_free_result(TemporalResult *result);

/**
 * Frees a CompareResult's allocated strings.
 */
void temporal_free_compare_result(CompareResult *result);

// ============================================================================
// Instant API
// ============================================================================

/**
 * Returns the current instant as an ISO 8601 string (e.g., "2024-01-15T10:30:45.123Z").
 * The caller is responsible for freeing the returned string using `temporal_free_string`.
 *
 * Returns NULL on error.
 */
char *temporal_instant_now(void);

/**
 * Parses an ISO 8601 string into an Instant and returns the normalized string.
 */
TemporalResult temporal_instant_from_string(const char *s);

/**
 * Creates an Instant from epoch milliseconds.
 */
TemporalResult temporal_instant_from_epoch_milliseconds(int64_t ms);

/**
 * Creates an Instant from epoch nanoseconds (string input).
 */
TemporalResult temporal_instant_from_epoch_nanoseconds(const char *ns_str);

/**
 * Returns the epoch milliseconds of an Instant (as string).
 */
TemporalResult temporal_instant_epoch_milliseconds(const char *s);

/**
 * Returns the epoch nanoseconds of an Instant (as string).
 */
TemporalResult temporal_instant_epoch_nanoseconds(const char *s);

/**
 * Adds a duration to an instant.
 */
TemporalResult temporal_instant_add(const char *instant_str, const char *duration_str);

/**
 * Subtracts a duration from an instant.
 */
TemporalResult temporal_instant_subtract(const char *instant_str, const char *duration_str);

/**
 * Compares two instants.
 */
CompareResult temporal_instant_compare(const char *a, const char *b);

// ============================================================================
// Now API
// ============================================================================

/**
 * Returns the current plain date time as an ISO 8601 string.
 */
TemporalResult temporal_now_plain_date_time_iso(const char *tz_id);

/**
 * Returns the current plain date as an ISO 8601 string.
 */
TemporalResult temporal_now_plain_date_iso(const char *tz_id);

/**
 * Returns the current plain time as an ISO 8601 string.
 */
TemporalResult temporal_now_plain_time_iso(const char *tz_id);

// ============================================================================
// PlainTime API
// ============================================================================

typedef struct {
    uint8_t hour;
    uint8_t minute;
    uint8_t second;
    uint16_t millisecond;
    uint16_t microsecond;
    uint16_t nanosecond;
    int8_t is_valid;
} PlainTimeComponents;

/**
 * Parses an ISO 8601 string into a PlainTime and returns the normalized string.
 */
TemporalResult temporal_plain_time_from_string(const char *s);

/**
 * Creates a PlainTime from individual components.
 */
TemporalResult temporal_plain_time_from_components(
    uint8_t hour,
    uint8_t minute,
    uint8_t second,
    uint16_t millisecond,
    uint16_t microsecond,
    uint16_t nanosecond
);

/**
 * Gets all component values from a PlainTime string.
 */
void temporal_plain_time_get_components(const char *s, PlainTimeComponents *out);

/**
 * Adds a duration to a PlainTime.
 */
TemporalResult temporal_plain_time_add(const char *time_str, const char *duration_str);

/**
 * Subtracts a duration from a PlainTime.
 */
TemporalResult temporal_plain_time_subtract(const char *time_str, const char *duration_str);

/**
 * Compares two PlainTime objects.
 */
CompareResult temporal_plain_time_compare(const char *a, const char *b);

// ============================================================================
// PlainDate API
// ============================================================================

typedef struct {
    int32_t year;
    uint8_t month;
    uint8_t day;
    uint16_t day_of_week;
    uint16_t day_of_year;
    uint16_t week_of_year;
    int32_t year_of_week;
    uint16_t days_in_week;
    uint16_t days_in_month;
    uint16_t days_in_year;
    uint16_t months_in_year;
    int8_t in_leap_year;
    int8_t is_valid;
} PlainDateComponents;

TemporalResult temporal_plain_date_from_string(const char *s);
TemporalResult temporal_plain_date_from_components(int32_t year, uint8_t month, uint8_t day, const char *calendar_id);
void temporal_plain_date_get_components(const char *s, PlainDateComponents *out);
TemporalResult temporal_plain_date_get_month_code(const char *s);
TemporalResult temporal_plain_date_get_calendar(const char *s);
TemporalResult temporal_plain_date_add(const char *date_str, const char *duration_str);
TemporalResult temporal_plain_date_subtract(const char *date_str, const char *duration_str);
CompareResult temporal_plain_date_compare(const char *a, const char *b);
TemporalResult temporal_plain_date_with(const char *date_str, int32_t year, int32_t month, int32_t day, const char *calendar_id);
TemporalResult temporal_plain_date_until(const char *one_str, const char *two_str);
TemporalResult temporal_plain_date_since(const char *one_str, const char *two_str);

// ============================================================================
// PlainDateTime API
// ============================================================================

typedef struct {
    int32_t year;
    uint8_t month;
    uint8_t day;
    uint16_t day_of_week;
    uint16_t day_of_year;
    uint16_t week_of_year;
    int32_t year_of_week;
    uint16_t days_in_week;
    uint16_t days_in_month;
    uint16_t days_in_year;
    uint16_t months_in_year;
    int8_t in_leap_year;
    uint8_t hour;
    uint8_t minute;
    uint8_t second;
    uint16_t millisecond;
    uint16_t microsecond;
    uint16_t nanosecond;
    int8_t is_valid;
} PlainDateTimeComponents;

TemporalResult temporal_plain_date_time_from_string(const char *s);
TemporalResult temporal_plain_date_time_from_components(
    int32_t year, uint8_t month, uint8_t day,
    uint8_t hour, uint8_t minute, uint8_t second,
    uint16_t millisecond, uint16_t microsecond, uint16_t nanosecond,
    const char *calendar_id
);
void temporal_plain_date_time_get_components(const char *s, PlainDateTimeComponents *out);
TemporalResult temporal_plain_date_time_get_month_code(const char *s);
TemporalResult temporal_plain_date_time_get_calendar(const char *s);
TemporalResult temporal_plain_date_time_add(const char *dt_str, const char *duration_str);
TemporalResult temporal_plain_date_time_subtract(const char *dt_str, const char *duration_str);
CompareResult temporal_plain_date_time_compare(const char *a, const char *b);
TemporalResult temporal_plain_date_time_with(
    const char *dt_str,
    int32_t year, int32_t month, int32_t day,
    int32_t hour, int32_t minute, int32_t second,
    int32_t millisecond, int32_t microsecond, int32_t nanosecond,
    const char *calendar_id
);
TemporalResult temporal_plain_date_time_until(const char *one_str, const char *two_str);
TemporalResult temporal_plain_date_time_since(const char *one_str, const char *two_str);

// ============================================================================
// PlainYearMonth API
// ============================================================================

typedef struct {
    int32_t year;
    uint8_t month;
    uint8_t day;
    uint16_t days_in_month;
    uint16_t days_in_year;
    uint16_t months_in_year;
    int8_t in_leap_year;
    int32_t era_year;
    int8_t is_valid;
} PlainYearMonthComponents;

TemporalResult temporal_plain_year_month_from_string(const char *s);
TemporalResult temporal_plain_year_month_from_components(
    int32_t year, uint8_t month, const char *calendar_id, uint8_t reference_day
);
void temporal_plain_year_month_get_components(const char *s, PlainYearMonthComponents *out);
TemporalResult temporal_plain_year_month_get_month_code(const char *s);
TemporalResult temporal_plain_year_month_get_calendar(const char *s);
TemporalResult temporal_plain_year_month_add(const char *ym_str, const char *duration_str);
TemporalResult temporal_plain_year_month_subtract(const char *ym_str, const char *duration_str);
CompareResult temporal_plain_year_month_compare(const char *a, const char *b);
TemporalResult temporal_plain_year_month_with(
    const char *ym_str, int32_t year, int32_t month, const char *calendar_id
);
TemporalResult temporal_plain_year_month_until(const char *one_str, const char *two_str);
TemporalResult temporal_plain_year_month_since(const char *one_str, const char *two_str);
TemporalResult temporal_plain_year_month_to_plain_date(const char *ym_str, int32_t day);

// ============================================================================
// PlainMonthDay API
// ============================================================================

typedef struct {
    uint8_t month;
    uint8_t day;
    int8_t is_valid;
} PlainMonthDayComponents;

TemporalResult temporal_plain_month_day_from_string(const char *s);
TemporalResult temporal_plain_month_day_from_components(
    uint8_t month, uint8_t day, const char *calendar_id, int32_t reference_year
);
void temporal_plain_month_day_get_components(const char *s, PlainMonthDayComponents *out);
TemporalResult temporal_plain_month_day_get_month_code(const char *s);
TemporalResult temporal_plain_month_day_get_calendar(const char *s);
TemporalResult temporal_plain_month_day_to_plain_date(const char *md_str, int32_t year);

// ============================================================================
// Calendar API
// ============================================================================

/**
 * Gets a Calendar from a string identifier.
 */
TemporalResult temporal_calendar_from(const char *id);

/**
 * Gets the identifier of a calendar.
 */
TemporalResult temporal_calendar_id(const char *id);

/**
 * Frees a string allocated by temporal functions.
 */
void temporal_free_string(char *s);

// ============================================================================
// Duration API
// ============================================================================

/**
 * Duration component structure for FFI.
 */
typedef struct {
    int64_t years;
    int64_t months;
    int64_t weeks;
    int64_t days;
    int64_t hours;
    int64_t minutes;
    int64_t seconds;
    int64_t milliseconds;
    int64_t microseconds;
    int64_t nanoseconds;
    int8_t sign;
    int8_t is_valid;
} DurationComponents;

/**
 * Parses an ISO 8601 duration string and returns a TemporalResult.
 */
TemporalResult temporal_duration_from_string(const char *s);

/**
 * Creates a duration from individual component values.
 * Returns a TemporalResult with the ISO string representation.
 */
TemporalResult temporal_duration_from_components(
    int64_t years,
    int64_t months,
    int64_t weeks,
    int64_t days,
    int64_t hours,
    int64_t minutes,
    int64_t seconds,
    int64_t milliseconds,
    int64_t microseconds,
    int64_t nanoseconds
);

/**
 * Gets all component values from a duration string in a single call.
 * Sets out->is_valid to 1 on success, 0 on error.
 */
void temporal_duration_get_components(const char *s, DurationComponents *out);

/**
 * Adds two durations and returns a TemporalResult.
 */
TemporalResult temporal_duration_add(const char *a, const char *b);

/**
 * Subtracts duration b from a and returns a TemporalResult.
 */
TemporalResult temporal_duration_subtract(const char *a, const char *b);

/**
 * Negates a duration and returns a TemporalResult.
 */
TemporalResult temporal_duration_negated(const char *s);

/**
 * Gets the absolute value of a duration and returns a TemporalResult.
 */
TemporalResult temporal_duration_abs(const char *s);

/**
 * Compares two durations and returns a CompareResult.
 * Value is -1, 0, or 1 for less than, equal, or greater than.
 * Note: Comparing durations with years, months, or weeks requires relativeTo (not yet supported).
 */
CompareResult temporal_duration_compare(const char *a, const char *b);

/**
 * Creates a new duration by replacing specified components.
 * Pass INT64_MIN for components that should not be changed.
 */
TemporalResult temporal_duration_with(
    const char *original,
    int64_t years,
    int64_t months,
    int64_t weeks,
    int64_t days,
    int64_t hours,
    int64_t minutes,
    int64_t seconds,
    int64_t milliseconds,
    int64_t microseconds,
    int64_t nanoseconds
);

#ifdef __cplusplus
}
#endif

#endif /* TEMPORAL_RN_H */

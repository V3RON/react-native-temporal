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

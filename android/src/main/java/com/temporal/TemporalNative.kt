package com.temporal

/**
 * JNI bridge to the temporal_rn Rust library.
 */
object TemporalNative {
    init {
        System.loadLibrary("temporal_rn")
    }

    /**
     * Returns the current instant as an ISO 8601 string.
     * Example: "2024-01-15T10:30:45.123456789Z"
     * Throws TemporalRangeError on failure.
     */
    @Throws(TemporalRangeError::class)
    external fun instantNow(): String

    @Throws(TemporalRangeError::class, TemporalTypeError::class)
    external fun instantFromString(s: String): String

    @Throws(TemporalRangeError::class)
    external fun instantFromEpochMilliseconds(ms: Long): String

    @Throws(TemporalRangeError::class, TemporalTypeError::class)
    external fun instantFromEpochNanoseconds(nsStr: String): String

    @Throws(TemporalRangeError::class, TemporalTypeError::class)
    external fun instantEpochMilliseconds(s: String): String

    @Throws(TemporalRangeError::class, TemporalTypeError::class)
    external fun instantEpochNanoseconds(s: String): String

    @Throws(TemporalRangeError::class, TemporalTypeError::class)
    external fun instantAdd(instant: String, duration: String): String

    @Throws(TemporalRangeError::class, TemporalTypeError::class)
    external fun instantSubtract(instant: String, duration: String): String

    @Throws(TemporalRangeError::class, TemporalTypeError::class)
    external fun instantCompare(one: String, two: String): Int

    /**
     * Now API
     */

    @Throws(TemporalRangeError::class, TemporalTypeError::class)
    external fun nowPlainDateTimeISO(tzId: String): String

    @Throws(TemporalRangeError::class, TemporalTypeError::class)
    external fun nowPlainDateISO(tzId: String): String

    @Throws(TemporalRangeError::class, TemporalTypeError::class)
    external fun nowPlainTimeISO(tzId: String): String

    /**
     * PlainTime API
     */

    @Throws(TemporalRangeError::class, TemporalTypeError::class)
    external fun plainTimeFromString(s: String): String

    @Throws(TemporalRangeError::class, TemporalTypeError::class)
    external fun plainTimeFromComponents(
        hour: Int,
        minute: Int,
        second: Int,
        millisecond: Int,
        microsecond: Int,
        nanosecond: Int
    ): String

    @Throws(TemporalRangeError::class, TemporalTypeError::class)
    external fun plainTimeGetAllComponents(s: String): LongArray

    @Throws(TemporalRangeError::class, TemporalTypeError::class)
    external fun plainTimeAdd(time: String, duration: String): String

    @Throws(TemporalRangeError::class, TemporalTypeError::class)
    external fun plainTimeSubtract(time: String, duration: String): String

    @Throws(TemporalRangeError::class, TemporalTypeError::class)
    external fun plainTimeCompare(one: String, two: String): Int

    /**
     * Calendar API
     */

    @Throws(TemporalRangeError::class, TemporalTypeError::class)
    external fun calendarFrom(id: String): String

    @Throws(TemporalRangeError::class, TemporalTypeError::class)
    external fun calendarId(id: String): String

    /**
     * Duration API
     */

    /**
     * Parses an ISO 8601 duration string and returns its normalized string representation.
     * Throws TemporalRangeError for invalid format, TemporalTypeError for null input.
     */
    @Throws(TemporalRangeError::class, TemporalTypeError::class)
    external fun durationFromString(input: String): String

    /**
     * Creates a duration from individual component values.
     * Returns the ISO 8601 string representation.
     * Throws TemporalRangeError if values have mixed signs or are invalid.
     */
    @Throws(TemporalRangeError::class)
    external fun durationFromComponents(
        years: Long,
        months: Long,
        weeks: Long,
        days: Long,
        hours: Long,
        minutes: Long,
        seconds: Long,
        milliseconds: Long,
        microseconds: Long,
        nanoseconds: Long
    ): String

    /**
     * Gets all component values from a duration string in a single call.
     * Returns a long array: [years, months, weeks, days, hours, minutes, seconds,
     *                        milliseconds, microseconds, nanoseconds, sign, blank]
     * Throws TemporalRangeError for invalid duration, TemporalTypeError for null input.
     */
    @Throws(TemporalRangeError::class, TemporalTypeError::class)
    external fun durationGetAllComponents(durationStr: String): LongArray

    /**
     * Adds two durations and returns the result as an ISO string.
     * Throws TemporalRangeError on error, TemporalTypeError for null input.
     */
    @Throws(TemporalRangeError::class, TemporalTypeError::class)
    external fun durationAdd(a: String, b: String): String

    /**
     * Subtracts duration b from a and returns the result as an ISO string.
     * Throws TemporalRangeError on error, TemporalTypeError for null input.
     */
    @Throws(TemporalRangeError::class, TemporalTypeError::class)
    external fun durationSubtract(a: String, b: String): String

    /**
     * Negates a duration and returns the result as an ISO string.
     * Throws TemporalRangeError on error, TemporalTypeError for null input.
     */
    @Throws(TemporalRangeError::class, TemporalTypeError::class)
    external fun durationNegated(input: String): String

    /**
     * Gets the absolute value of a duration and returns the result as an ISO string.
     * Throws TemporalRangeError on error, TemporalTypeError for null input.
     */
    @Throws(TemporalRangeError::class, TemporalTypeError::class)
    external fun durationAbs(input: String): String

    /**
     * Compares two durations. Returns -1, 0, or 1.
     * Note: Durations with years, months, or weeks cannot be compared without relativeTo.
     * Throws TemporalRangeError if comparison is not possible, TemporalTypeError for null input.
     */
    @Throws(TemporalRangeError::class, TemporalTypeError::class)
    external fun durationCompare(a: String, b: String): Int

    /**
     * Creates a new duration by replacing specified components.
     * Pass -9007199254740991 (Number.MIN_SAFE_INTEGER) for components that should not be changed.
     * Throws TemporalRangeError for invalid values, TemporalTypeError for null input.
     */
    @Throws(TemporalRangeError::class, TemporalTypeError::class)
    external fun durationWith(
        original: String,
        years: Long,
        months: Long,
        weeks: Long,
        days: Long,
        hours: Long,
        minutes: Long,
        seconds: Long,
        milliseconds: Long,
        microseconds: Long,
        nanoseconds: Long
    ): String
}

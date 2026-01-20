#import "Temporal.h"
#import "temporal_rn.h"
#import <React/RCTUtils.h>

// Helper macros for throwing errors with type markers for JS to parse
#define THROW_RANGE_ERROR(msg) \
    @throw [NSException exceptionWithName:@"RangeError" \
                                   reason:[@"[RangeError] " stringByAppendingString:(msg)] \
                                 userInfo:nil]

#define THROW_TYPE_ERROR(msg) \
    @throw [NSException exceptionWithName:@"TypeError" \
                                   reason:[@"[TypeError] " stringByAppendingString:(msg)] \
                                 userInfo:nil]

// Helper to throw appropriate JS exception based on error type
static void throwTemporalError(TemporalResult *result) {
    if (result->error_type == TEMPORAL_ERROR_NONE) {
        return;
    }

    NSString *baseMessage = result->error_message
        ? [NSString stringWithUTF8String:result->error_message]
        : @"Unknown error";

    int errorType = result->error_type;

    // Free the result before throwing
    temporal_free_result(result);

    if (errorType == TEMPORAL_ERROR_RANGE) {
        THROW_RANGE_ERROR(baseMessage);
    } else {
        THROW_TYPE_ERROR(baseMessage);
    }
}

// Helper to throw appropriate JS exception based on CompareResult error type
static void throwCompareError(CompareResult *result) {
    if (result->error_type == TEMPORAL_ERROR_NONE) {
        return;
    }

    NSString *baseMessage = result->error_message
        ? [NSString stringWithUTF8String:result->error_message]
        : @"Unknown error";

    int errorType = result->error_type;

    // Free the result before throwing
    temporal_free_compare_result(result);

    if (errorType == TEMPORAL_ERROR_RANGE) {
        THROW_RANGE_ERROR(baseMessage);
    } else {
        THROW_TYPE_ERROR(baseMessage);
    }
}

// Helper to extract value from result, throwing on error
static NSString *extractResultValue(TemporalResult result) {
    if (result.error_type != TEMPORAL_ERROR_NONE) {
        throwTemporalError(&result);
        return @""; // Never reached
    }

    NSString *value = result.value
        ? [NSString stringWithUTF8String:result.value]
        : @"";

    temporal_free_result(&result);
    return value;
}

@implementation Temporal

- (NSNumber *)multiply:(double)a b:(double)b {
    NSNumber *result = @(a * b);
    return result;
}

- (NSString *)instantNow {
    char *result = temporal_instant_now();
    if (result == NULL) {
        THROW_RANGE_ERROR(@"Failed to get current instant");
    }
    NSString *nsResult = [NSString stringWithUTF8String:result];
    temporal_free_string(result);
    return nsResult;
}

- (NSString *)instantFromString:(NSString *)s {
    if (s == nil) {
        THROW_TYPE_ERROR(@"Instant string cannot be null");
    }
    const char *sCStr = [s UTF8String];
    if (sCStr == NULL) {
        THROW_TYPE_ERROR(@"Invalid instant string encoding");
    }
    TemporalResult result = temporal_instant_from_string(sCStr);
    return extractResultValue(result);
}

- (NSString *)instantFromEpochMilliseconds:(double)ms {
    TemporalResult result = temporal_instant_from_epoch_milliseconds((int64_t)ms);
    return extractResultValue(result);
}

- (NSString *)instantFromEpochNanoseconds:(NSString *)nsStr {
    if (nsStr == nil) {
        THROW_TYPE_ERROR(@"Nanoseconds string cannot be null");
    }
    const char *nsCStr = [nsStr UTF8String];
    if (nsCStr == NULL) {
        THROW_TYPE_ERROR(@"Invalid nanoseconds string encoding");
    }
    TemporalResult result = temporal_instant_from_epoch_nanoseconds(nsCStr);
    return extractResultValue(result);
}

- (double)instantEpochMilliseconds:(NSString *)instant {
    if (instant == nil) {
        THROW_TYPE_ERROR(@"Instant string cannot be null");
    }
    const char *sCStr = [instant UTF8String];
    if (sCStr == NULL) {
        THROW_TYPE_ERROR(@"Invalid instant string encoding");
    }
    TemporalResult result = temporal_instant_epoch_milliseconds(sCStr);
    NSString *val = extractResultValue(result);
    return [val doubleValue];
}

- (NSString *)instantEpochNanoseconds:(NSString *)instant {
    if (instant == nil) {
        THROW_TYPE_ERROR(@"Instant string cannot be null");
    }
    const char *sCStr = [instant UTF8String];
    if (sCStr == NULL) {
        THROW_TYPE_ERROR(@"Invalid instant string encoding");
    }
    TemporalResult result = temporal_instant_epoch_nanoseconds(sCStr);
    return extractResultValue(result);
}

- (NSString *)instantAdd:(NSString *)instant duration:(NSString *)duration {
    if (instant == nil || duration == nil) {
        THROW_TYPE_ERROR(@"Arguments cannot be null");
    }
    const char *iCStr = [instant UTF8String];
    const char *dCStr = [duration UTF8String];
    if (iCStr == NULL || dCStr == NULL) {
        THROW_TYPE_ERROR(@"Invalid string encoding");
    }
    TemporalResult result = temporal_instant_add(iCStr, dCStr);
    return extractResultValue(result);
}

- (NSString *)instantSubtract:(NSString *)instant duration:(NSString *)duration {
    if (instant == nil || duration == nil) {
        THROW_TYPE_ERROR(@"Arguments cannot be null");
    }
    const char *iCStr = [instant UTF8String];
    const char *dCStr = [duration UTF8String];
    if (iCStr == NULL || dCStr == NULL) {
        THROW_TYPE_ERROR(@"Invalid string encoding");
    }
    TemporalResult result = temporal_instant_subtract(iCStr, dCStr);
    return extractResultValue(result);
}

- (double)instantCompare:(NSString *)one two:(NSString *)two {
    if (one == nil || two == nil) {
        THROW_TYPE_ERROR(@"Arguments cannot be null");
    }
    const char *aCStr = [one UTF8String];
    const char *bCStr = [two UTF8String];
    if (aCStr == NULL || bCStr == NULL) {
        THROW_TYPE_ERROR(@"Invalid string encoding");
    }
    CompareResult result = temporal_instant_compare(aCStr, bCStr);
    if (result.error_type != TEMPORAL_ERROR_NONE) {
        throwCompareError(&result);
        return 0;
    }
    double val = (double)result.value;
    temporal_free_compare_result(&result);
    return val;
}

- (NSString *)instantUntil:(NSString *)one two:(NSString *)two largestUnit:(NSString *)largestUnit smallestUnit:(NSString *)smallestUnit roundingIncrement:(double)roundingIncrement roundingMode:(NSString *)roundingMode {
    if (!one || !two) THROW_TYPE_ERROR(@"Arguments cannot be null");
    const char *largestCStr = largestUnit ? [largestUnit UTF8String] : NULL;
    const char *smallestCStr = smallestUnit ? [smallestUnit UTF8String] : NULL;
    const char *modeCStr = roundingMode ? [roundingMode UTF8String] : NULL;
    
    TemporalResult result = temporal_instant_until([one UTF8String], [two UTF8String], largestCStr, smallestCStr, (int64_t)roundingIncrement, modeCStr);
    return extractResultValue(result);
}

- (NSString *)instantSince:(NSString *)one two:(NSString *)two largestUnit:(NSString *)largestUnit smallestUnit:(NSString *)smallestUnit roundingIncrement:(double)roundingIncrement roundingMode:(NSString *)roundingMode {
    if (!one || !two) THROW_TYPE_ERROR(@"Arguments cannot be null");
    const char *largestCStr = largestUnit ? [largestUnit UTF8String] : NULL;
    const char *smallestCStr = smallestUnit ? [smallestUnit UTF8String] : NULL;
    const char *modeCStr = roundingMode ? [roundingMode UTF8String] : NULL;
    
    TemporalResult result = temporal_instant_since([one UTF8String], [two UTF8String], largestCStr, smallestCStr, (int64_t)roundingIncrement, modeCStr);
    return extractResultValue(result);
}

- (NSString *)instantRound:(NSString *)instantStr smallestUnit:(NSString *)smallestUnit roundingIncrement:(double)roundingIncrement roundingMode:(NSString *)roundingMode {
    if (!instantStr || !smallestUnit) THROW_TYPE_ERROR(@"Arguments cannot be null");
    const char *smallestCStr = [smallestUnit UTF8String];
    const char *modeCStr = roundingMode ? [roundingMode UTF8String] : NULL;
    
    TemporalResult result = temporal_instant_round([instantStr UTF8String], smallestCStr, (int64_t)roundingIncrement, modeCStr);
    return extractResultValue(result);
}

- (NSString *)instantToZonedDateTime:(NSString *)instantStr calendarId:(NSString *)calendarId timeZoneId:(NSString *)timeZoneId {
    if (!instantStr || !timeZoneId) THROW_TYPE_ERROR(@"Arguments cannot be null");
    const char *calendarCStr = calendarId ? [calendarId UTF8String] : NULL;
    
    TemporalResult result = temporal_instant_to_zoned_date_time([instantStr UTF8String], calendarCStr, [timeZoneId UTF8String]);
    return extractResultValue(result);
}

// Now methods

- (NSString *)nowTimeZoneId {
    return [[NSTimeZone localTimeZone] name];
}

- (NSString *)nowPlainDateTimeISO:(NSString *)tz {
    NSString *tzId = tz ?: [[NSTimeZone localTimeZone] name];
    TemporalResult result = temporal_now_plain_date_time_iso([tzId UTF8String]);
    return extractResultValue(result);
}

- (NSString *)nowPlainDateISO:(NSString *)tz {
    NSString *tzId = tz ?: [[NSTimeZone localTimeZone] name];
    TemporalResult result = temporal_now_plain_date_iso([tzId UTF8String]);
    return extractResultValue(result);
}

- (NSString *)nowPlainTimeISO:(NSString *)tz {
    NSString *tzId = tz ?: [[NSTimeZone localTimeZone] name];
    TemporalResult result = temporal_now_plain_time_iso([tzId UTF8String]);
    return extractResultValue(result);
}

- (NSString *)nowZonedDateTimeISO:(NSString *)tz {
    NSString *tzId = tz ?: [[NSTimeZone localTimeZone] name];
    TemporalResult result = temporal_now_zoned_date_time_iso([tzId UTF8String]);
    return extractResultValue(result);
}

// PlainTime methods

- (NSString *)plainTimeFromString:(NSString *)s {
    if (s == nil) {
        THROW_TYPE_ERROR(@"PlainTime string cannot be null");
    }
    const char *sCStr = [s UTF8String];
    if (sCStr == NULL) {
        THROW_TYPE_ERROR(@"Invalid plain time string encoding");
    }
    TemporalResult result = temporal_plain_time_from_string(sCStr);
    return extractResultValue(result);
}

- (NSString *)plainTimeFromComponents:(double)hour
                               minute:(double)minute
                               second:(double)second
                          millisecond:(double)millisecond
                          microsecond:(double)microsecond
                           nanosecond:(double)nanosecond {
    TemporalResult result = temporal_plain_time_from_components(
        (uint8_t)hour,
        (uint8_t)minute,
        (uint8_t)second,
        (uint16_t)millisecond,
        (uint16_t)microsecond,
        (uint16_t)nanosecond
    );
    return extractResultValue(result);
}

- (NSDictionary *)plainTimeGetAllComponents:(NSString *)plainTimeStr {
    if (plainTimeStr == nil) {
        THROW_TYPE_ERROR(@"PlainTime string cannot be null");
    }
    const char *sCStr = [plainTimeStr UTF8String];
    if (sCStr == NULL) {
        THROW_TYPE_ERROR(@"Invalid plain time string encoding");
    }

    PlainTimeComponents components;
    temporal_plain_time_get_components(sCStr, &components);

    if (components.is_valid == 0) {
        THROW_RANGE_ERROR(@"Invalid plain time");
    }

    return @{
        @"hour": @(components.hour),
        @"minute": @(components.minute),
        @"second": @(components.second),
        @"millisecond": @(components.millisecond),
        @"microsecond": @(components.microsecond),
        @"nanosecond": @(components.nanosecond)
    };
}

- (NSString *)plainTimeAdd:(NSString *)plainTime duration:(NSString *)duration {
    if (plainTime == nil || duration == nil) {
        THROW_TYPE_ERROR(@"Arguments cannot be null");
    }
    const char *tCStr = [plainTime UTF8String];
    const char *dCStr = [duration UTF8String];
    if (tCStr == NULL || dCStr == NULL) {
        THROW_TYPE_ERROR(@"Invalid string encoding");
    }
    TemporalResult result = temporal_plain_time_add(tCStr, dCStr);
    return extractResultValue(result);
}

- (NSString *)plainTimeSubtract:(NSString *)plainTime duration:(NSString *)duration {
    if (plainTime == nil || duration == nil) {
        THROW_TYPE_ERROR(@"Arguments cannot be null");
    }
    const char *tCStr = [plainTime UTF8String];
    const char *dCStr = [duration UTF8String];
    if (tCStr == NULL || dCStr == NULL) {
        THROW_TYPE_ERROR(@"Invalid string encoding");
    }
    TemporalResult result = temporal_plain_time_subtract(tCStr, dCStr);
    return extractResultValue(result);
}

- (double)plainTimeCompare:(NSString *)one two:(NSString *)two {
    if (one == nil || two == nil) {
        THROW_TYPE_ERROR(@"Arguments cannot be null");
    }
    const char *aCStr = [one UTF8String];
    const char *bCStr = [two UTF8String];
    if (aCStr == NULL || bCStr == NULL) {
        THROW_TYPE_ERROR(@"Invalid string encoding");
    }
    CompareResult result = temporal_plain_time_compare(aCStr, bCStr);
    if (result.error_type != TEMPORAL_ERROR_NONE) {
        throwCompareError(&result);
        return 0;
    }
    double val = (double)result.value;
    temporal_free_compare_result(&result);
    return val;
}

- (NSString *)plainTimeUntil:(NSString *)one two:(NSString *)two largestUnit:(NSString *)largestUnit smallestUnit:(NSString *)smallestUnit roundingIncrement:(double)roundingIncrement roundingMode:(NSString *)roundingMode {
    if (!one || !two) THROW_TYPE_ERROR(@"Arguments cannot be null");
    const char *largestCStr = largestUnit ? [largestUnit UTF8String] : NULL;
    const char *smallestCStr = smallestUnit ? [smallestUnit UTF8String] : NULL;
    const char *modeCStr = roundingMode ? [roundingMode UTF8String] : NULL;
    
    TemporalResult result = temporal_plain_time_until([one UTF8String], [two UTF8String], largestCStr, smallestCStr, (int64_t)roundingIncrement, modeCStr);
    return extractResultValue(result);
}

- (NSString *)plainTimeSince:(NSString *)one two:(NSString *)two largestUnit:(NSString *)largestUnit smallestUnit:(NSString *)smallestUnit roundingIncrement:(double)roundingIncrement roundingMode:(NSString *)roundingMode {
    if (!one || !two) THROW_TYPE_ERROR(@"Arguments cannot be null");
    const char *largestCStr = largestUnit ? [largestUnit UTF8String] : NULL;
    const char *smallestCStr = smallestUnit ? [smallestUnit UTF8String] : NULL;
    const char *modeCStr = roundingMode ? [roundingMode UTF8String] : NULL;
    
    TemporalResult result = temporal_plain_time_since([one UTF8String], [two UTF8String], largestCStr, smallestCStr, (int64_t)roundingIncrement, modeCStr);
    return extractResultValue(result);
}

- (NSString *)plainTimeRound:(NSString *)timeStr smallestUnit:(NSString *)smallestUnit roundingIncrement:(double)roundingIncrement roundingMode:(NSString *)roundingMode {
    if (!timeStr || !smallestUnit) THROW_TYPE_ERROR(@"Arguments cannot be null");
    const char *smallestCStr = [smallestUnit UTF8String];
    const char *modeCStr = roundingMode ? [roundingMode UTF8String] : NULL;
    
    TemporalResult result = temporal_plain_time_round([timeStr UTF8String], smallestCStr, (int64_t)roundingIncrement, modeCStr);
    return extractResultValue(result);
}

// PlainDate methods

- (NSString *)plainDateFromString:(NSString *)s {
    if (s == nil) {
        THROW_TYPE_ERROR(@"PlainDate string cannot be null");
    }
    const char *sCStr = [s UTF8String];
    if (sCStr == NULL) {
        THROW_TYPE_ERROR(@"Invalid plain date string encoding");
    }
    TemporalResult result = temporal_plain_date_from_string(sCStr);
    return extractResultValue(result);
}

- (NSString *)plainDateFromComponents:(double)year
                               month:(double)month
                                 day:(double)day
                          calendarId:(NSString *)calendarId {
    const char *cIdCStr = calendarId ? [calendarId UTF8String] : NULL;
    TemporalResult result = temporal_plain_date_from_components((int32_t)year, (uint8_t)month, (uint8_t)day, cIdCStr);
    return extractResultValue(result);
}

- (NSArray<NSNumber *> *)plainDateGetAllComponents:(NSString *)s {
    if (s == nil) {
        THROW_TYPE_ERROR(@"PlainDate string cannot be null");
    }
    const char *sCStr = [s UTF8String];
    if (sCStr == NULL) {
        THROW_TYPE_ERROR(@"Invalid plain date string encoding");
    }

    PlainDateComponents c;
    temporal_plain_date_get_components(sCStr, &c);

    if (c.is_valid == 0) {
        THROW_RANGE_ERROR(@"Invalid plain date");
    }

    return @[
        @(c.year), @(c.month), @(c.day),
        @(c.day_of_week), @(c.day_of_year), @(c.week_of_year), @(c.year_of_week),
        @(c.days_in_week), @(c.days_in_month), @(c.days_in_year), @(c.months_in_year),
        @(c.in_leap_year)
    ];
}

- (NSString *)plainDateGetMonthCode:(NSString *)s {
    if (s == nil) return @"";
    TemporalResult result = temporal_plain_date_get_month_code([s UTF8String]);
    return extractResultValue(result);
}

- (NSString *)plainDateGetCalendar:(NSString *)s {
    if (s == nil) return @"";
    TemporalResult result = temporal_plain_date_get_calendar([s UTF8String]);
    return extractResultValue(result);
}

- (NSString *)plainDateAdd:(NSString *)s duration:(NSString *)duration {
    if (!s || !duration) THROW_TYPE_ERROR(@"Arguments cannot be null");
    TemporalResult result = temporal_plain_date_add([s UTF8String], [duration UTF8String]);
    return extractResultValue(result);
}

- (NSString *)plainDateSubtract:(NSString *)s duration:(NSString *)duration {
    if (!s || !duration) THROW_TYPE_ERROR(@"Arguments cannot be null");
    TemporalResult result = temporal_plain_date_subtract([s UTF8String], [duration UTF8String]);
    return extractResultValue(result);
}

- (double)plainDateCompare:(NSString *)a b:(NSString *)b {
    if (!a || !b) THROW_TYPE_ERROR(@"Arguments cannot be null");
    CompareResult result = temporal_plain_date_compare([a UTF8String], [b UTF8String]);
    if (result.error_type != TEMPORAL_ERROR_NONE) {
        throwCompareError(&result);
        return 0;
    }
    double val = (double)result.value;
    temporal_free_compare_result(&result);
    return val;
}

- (NSString *)plainDateWith:(NSString *)s
                       year:(double)year
                      month:(double)month
                        day:(double)day
                 calendarId:(NSString *)calendarId {
    const char *cIdCStr = calendarId ? [calendarId UTF8String] : NULL;
    TemporalResult result = temporal_plain_date_with([s UTF8String], (int32_t)year, (int32_t)month, (int32_t)day, cIdCStr);
    return extractResultValue(result);
}

- (NSString *)plainDateUntil:(NSString *)one two:(NSString *)two {
    if (!one || !two) THROW_TYPE_ERROR(@"Arguments cannot be null");
    TemporalResult result = temporal_plain_date_until([one UTF8String], [two UTF8String]);
    return extractResultValue(result);
}

- (NSString *)plainDateSince:(NSString *)one two:(NSString *)two {
    if (!one || !two) THROW_TYPE_ERROR(@"Arguments cannot be null");
    TemporalResult result = temporal_plain_date_since([one UTF8String], [two UTF8String]);
    return extractResultValue(result);
}

// PlainDateTime methods

- (NSString *)plainDateTimeFromString:(NSString *)s {
    if (s == nil) {
        THROW_TYPE_ERROR(@"PlainDateTime string cannot be null");
    }
    const char *sCStr = [s UTF8String];
    if (sCStr == NULL) {
        THROW_TYPE_ERROR(@"Invalid plain date time string encoding");
    }
    TemporalResult result = temporal_plain_date_time_from_string(sCStr);
    return extractResultValue(result);
}

- (NSString *)plainDateTimeFromComponents:(double)year
                                    month:(double)month
                                      day:(double)day
                                     hour:(double)hour
                                   minute:(double)minute
                                   second:(double)second
                              millisecond:(double)millisecond
                              microsecond:(double)microsecond
                               nanosecond:(double)nanosecond
                               calendarId:(NSString *)calendarId {
    const char *cIdCStr = calendarId ? [calendarId UTF8String] : NULL;
    TemporalResult result = temporal_plain_date_time_from_components(
        (int32_t)year, (uint8_t)month, (uint8_t)day,
        (uint8_t)hour, (uint8_t)minute, (uint8_t)second,
        (uint16_t)millisecond, (uint16_t)microsecond, (uint16_t)nanosecond,
        cIdCStr
    );
    return extractResultValue(result);
}

- (NSArray<NSNumber *> *)plainDateTimeGetAllComponents:(NSString *)s {
    if (s == nil) {
        THROW_TYPE_ERROR(@"PlainDateTime string cannot be null");
    }
    const char *sCStr = [s UTF8String];
    if (sCStr == NULL) {
        THROW_TYPE_ERROR(@"Invalid plain date time string encoding");
    }

    PlainDateTimeComponents c;
    temporal_plain_date_time_get_components(sCStr, &c);

    if (c.is_valid == 0) {
        THROW_RANGE_ERROR(@"Invalid plain date time");
    }

    return @[
        @(c.year), @(c.month), @(c.day),
        @(c.day_of_week), @(c.day_of_year), @(c.week_of_year), @(c.year_of_week),
        @(c.days_in_week), @(c.days_in_month), @(c.days_in_year), @(c.months_in_year),
        @(c.in_leap_year),
        @(c.hour), @(c.minute), @(c.second),
        @(c.millisecond), @(c.microsecond), @(c.nanosecond)
    ];
}

- (NSString *)plainDateTimeGetMonthCode:(NSString *)s {
    if (s == nil) return @"";
    TemporalResult result = temporal_plain_date_time_get_month_code([s UTF8String]);
    return extractResultValue(result);
}

- (NSString *)plainDateTimeGetCalendar:(NSString *)s {
    if (s == nil) return @"";
    TemporalResult result = temporal_plain_date_time_get_calendar([s UTF8String]);
    return extractResultValue(result);
}

- (NSString *)plainDateTimeAdd:(NSString *)s duration:(NSString *)duration {
    if (!s || !duration) THROW_TYPE_ERROR(@"Arguments cannot be null");
    TemporalResult result = temporal_plain_date_time_add([s UTF8String], [duration UTF8String]);
    return extractResultValue(result);
}

- (NSString *)plainDateTimeSubtract:(NSString *)s duration:(NSString *)duration {
    if (!s || !duration) THROW_TYPE_ERROR(@"Arguments cannot be null");
    TemporalResult result = temporal_plain_date_time_subtract([s UTF8String], [duration UTF8String]);
    return extractResultValue(result);
}

- (double)plainDateTimeCompare:(NSString *)a b:(NSString *)b {
    if (!a || !b) THROW_TYPE_ERROR(@"Arguments cannot be null");
    CompareResult result = temporal_plain_date_time_compare([a UTF8String], [b UTF8String]);
    if (result.error_type != TEMPORAL_ERROR_NONE) {
        throwCompareError(&result);
        return 0;
    }
    double val = (double)result.value;
    temporal_free_compare_result(&result);
    return val;
}

- (NSString *)plainDateTimeWith:(NSString *)s
                           year:(double)year
                          month:(double)month
                            day:(double)day
                           hour:(double)hour
                         minute:(double)minute
                         second:(double)second
                    millisecond:(double)millisecond
                    microsecond:(double)microsecond
                     nanosecond:(double)nanosecond
                     calendarId:(NSString *)calendarId {
    const char *cIdCStr = calendarId ? [calendarId UTF8String] : NULL;
    TemporalResult result = temporal_plain_date_time_with(
        [s UTF8String],
        (int32_t)year, (int32_t)month, (int32_t)day,
        (int32_t)hour, (int32_t)minute, (int32_t)second,
        (int32_t)millisecond, (int32_t)microsecond, (int32_t)nanosecond,
        cIdCStr
    );
    return extractResultValue(result);
}

- (NSString *)plainDateTimeUntil:(NSString *)one two:(NSString *)two {
    if (!one || !two) THROW_TYPE_ERROR(@"Arguments cannot be null");
    TemporalResult result = temporal_plain_date_time_until([one UTF8String], [two UTF8String]);
    return extractResultValue(result);
}

- (NSString *)plainDateTimeSince:(NSString *)one two:(NSString *)two {
    if (!one || !two) THROW_TYPE_ERROR(@"Arguments cannot be null");
    TemporalResult result = temporal_plain_date_time_since([one UTF8String], [two UTF8String]);
    return extractResultValue(result);
}

// PlainYearMonth methods

- (NSString *)plainYearMonthFromString:(NSString *)s {
    if (s == nil) THROW_TYPE_ERROR(@"String cannot be null");
    TemporalResult result = temporal_plain_year_month_from_string([s UTF8String]);
    return extractResultValue(result);
}

- (NSString *)plainYearMonthFromComponents:(double)year month:(double)month calendarId:(NSString *)calendarId referenceDay:(double)referenceDay {
    const char *cIdCStr = calendarId ? [calendarId UTF8String] : NULL;
    TemporalResult result = temporal_plain_year_month_from_components(
        (int32_t)year, (uint8_t)month, cIdCStr, (uint8_t)referenceDay
    );
    return extractResultValue(result);
}

- (NSArray<NSNumber *> *)plainYearMonthGetAllComponents:(NSString *)s {
    if (s == nil) THROW_TYPE_ERROR(@"String cannot be null");
    PlainYearMonthComponents c;
    temporal_plain_year_month_get_components([s UTF8String], &c);
    
    if (c.is_valid == 0) {
        THROW_RANGE_ERROR(@"Invalid plain year month");
    }
    
    return @[
        @(c.year), @(c.month), @(c.day),
        @(c.days_in_month), @(c.days_in_year), @(c.months_in_year),
        @(c.in_leap_year), @(c.era_year)
    ];
}

- (NSString *)plainYearMonthGetMonthCode:(NSString *)s {
    if (s == nil) return @"";
    TemporalResult result = temporal_plain_year_month_get_month_code([s UTF8String]);
    return extractResultValue(result);
}

- (NSString *)plainYearMonthGetCalendar:(NSString *)s {
    if (s == nil) return @"";
    TemporalResult result = temporal_plain_year_month_get_calendar([s UTF8String]);
    return extractResultValue(result);
}

- (NSString *)plainYearMonthAdd:(NSString *)ym duration:(NSString *)duration {
    if (!ym || !duration) THROW_TYPE_ERROR(@"Arguments cannot be null");
    TemporalResult result = temporal_plain_year_month_add([ym UTF8String], [duration UTF8String]);
    return extractResultValue(result);
}

- (NSString *)plainYearMonthSubtract:(NSString *)ym duration:(NSString *)duration {
    if (!ym || !duration) THROW_TYPE_ERROR(@"Arguments cannot be null");
    TemporalResult result = temporal_plain_year_month_subtract([ym UTF8String], [duration UTF8String]);
    return extractResultValue(result);
}

- (double)plainYearMonthCompare:(NSString *)a b:(NSString *)b {
    if (!a || !b) THROW_TYPE_ERROR(@"Arguments cannot be null");
    CompareResult result = temporal_plain_year_month_compare([a UTF8String], [b UTF8String]);
    if (result.error_type != TEMPORAL_ERROR_NONE) {
        throwCompareError(&result);
        return 0;
    }
    double val = (double)result.value;
    temporal_free_compare_result(&result);
    return val;
}

- (NSString *)plainYearMonthWith:(NSString *)ym year:(double)year month:(double)month calendarId:(NSString *)calendarId {
    const char *cIdCStr = calendarId ? [calendarId UTF8String] : NULL;
    TemporalResult result = temporal_plain_year_month_with(
        [ym UTF8String], (int32_t)year, (int32_t)month, cIdCStr
    );
    return extractResultValue(result);
}

- (NSString *)plainYearMonthUntil:(NSString *)one two:(NSString *)two {
    if (!one || !two) THROW_TYPE_ERROR(@"Arguments cannot be null");
    TemporalResult result = temporal_plain_year_month_until([one UTF8String], [two UTF8String]);
    return extractResultValue(result);
}

- (NSString *)plainYearMonthSince:(NSString *)one two:(NSString *)two {
    if (!one || !two) THROW_TYPE_ERROR(@"Arguments cannot be null");
    TemporalResult result = temporal_plain_year_month_since([one UTF8String], [two UTF8String]);
    return extractResultValue(result);
}

- (NSString *)plainYearMonthToPlainDate:(NSString *)ym day:(double)day {
    if (ym == nil) THROW_TYPE_ERROR(@"String cannot be null");
    TemporalResult result = temporal_plain_year_month_to_plain_date([ym UTF8String], (int32_t)day);
    return extractResultValue(result);
}

// PlainMonthDay methods

- (NSString *)plainMonthDayFromString:(NSString *)s {
    if (s == nil) THROW_TYPE_ERROR(@"String cannot be null");
    TemporalResult result = temporal_plain_month_day_from_string([s UTF8String]);
    return extractResultValue(result);
}

- (NSString *)plainMonthDayFromComponents:(double)month day:(double)day calendarId:(NSString *)calendarId referenceYear:(double)referenceYear {
    const char *cIdCStr = calendarId ? [calendarId UTF8String] : NULL;
    TemporalResult result = temporal_plain_month_day_from_components(
        (uint8_t)month, (uint8_t)day, cIdCStr, (int32_t)referenceYear
    );
    return extractResultValue(result);
}

- (NSArray<NSNumber *> *)plainMonthDayGetAllComponents:(NSString *)s {
    if (s == nil) THROW_TYPE_ERROR(@"String cannot be null");
    PlainMonthDayComponents c;
    temporal_plain_month_day_get_components([s UTF8String], &c);
    
    if (c.is_valid == 0) {
        THROW_RANGE_ERROR(@"Invalid plain month day");
    }
    
    return @[@(c.month), @(c.day)];
}

- (NSString *)plainMonthDayGetMonthCode:(NSString *)s {
    if (s == nil) return @"";
    TemporalResult result = temporal_plain_month_day_get_month_code([s UTF8String]);
    return extractResultValue(result);
}

- (NSString *)plainMonthDayGetCalendar:(NSString *)s {
    if (s == nil) return @"";
    TemporalResult result = temporal_plain_month_day_get_calendar([s UTF8String]);
    return extractResultValue(result);
}

- (NSString *)plainMonthDayToPlainDate:(NSString *)md year:(double)year {
    if (md == nil) THROW_TYPE_ERROR(@"String cannot be null");
    TemporalResult result = temporal_plain_month_day_to_plain_date([md UTF8String], (int32_t)year);
    return extractResultValue(result);
}

// Calendar methods

- (NSString *)calendarFrom:(NSString *)id {
    if (id == nil) {
        THROW_TYPE_ERROR(@"Calendar identifier cannot be null");
    }
    const char *idCStr = [id UTF8String];
    if (idCStr == NULL) {
        THROW_TYPE_ERROR(@"Invalid calendar identifier encoding");
    }
    TemporalResult result = temporal_calendar_from(idCStr);
    return extractResultValue(result);
}

- (NSString *)calendarId:(NSString *)id {
    if (id == nil) {
        THROW_TYPE_ERROR(@"Calendar identifier cannot be null");
    }
    const char *idCStr = [id UTF8String];
    if (idCStr == NULL) {
        THROW_TYPE_ERROR(@"Invalid calendar identifier encoding");
    }
    TemporalResult result = temporal_calendar_id(idCStr);
    return extractResultValue(result);
}

// Duration methods

- (NSString *)durationFromString:(NSString *)input {
    if (input == nil) {
        THROW_TYPE_ERROR(@"Duration string cannot be null");
    }
    const char *inputCStr = [input UTF8String];
    if (inputCStr == NULL) {
        THROW_TYPE_ERROR(@"Invalid duration string encoding");
    }

    TemporalResult result = temporal_duration_from_string(inputCStr);
    return extractResultValue(result);
}

- (NSString *)durationFromComponents:(double)years
                              months:(double)months
                               weeks:(double)weeks
                                days:(double)days
                               hours:(double)hours
                             minutes:(double)minutes
                             seconds:(double)seconds
                        milliseconds:(double)milliseconds
                        microseconds:(double)microseconds
                         nanoseconds:(double)nanoseconds {
    TemporalResult result = temporal_duration_from_components(
        (int64_t)years,
        (int64_t)months,
        (int64_t)weeks,
        (int64_t)days,
        (int64_t)hours,
        (int64_t)minutes,
        (int64_t)seconds,
        (int64_t)milliseconds,
        (int64_t)microseconds,
        (int64_t)nanoseconds
    );
    return extractResultValue(result);
}

- (NSArray<NSNumber *> *)durationGetAllComponents:(NSString *)durationStr {
    if (durationStr == nil) {
        THROW_TYPE_ERROR(@"Duration string cannot be null");
    }
    const char *durationCStr = [durationStr UTF8String];
    if (durationCStr == NULL) {
        THROW_TYPE_ERROR(@"Invalid duration string encoding");
    }

    DurationComponents components;
    temporal_duration_get_components(durationCStr, &components);

    if (components.is_valid == 0) {
        THROW_RANGE_ERROR([NSString stringWithFormat:@"Invalid duration: %@", durationStr]);
    }

    // Return array: [years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds, sign, blank]
    return @[
        @(components.years),
        @(components.months),
        @(components.weeks),
        @(components.days),
        @(components.hours),
        @(components.minutes),
        @(components.seconds),
        @(components.milliseconds),
        @(components.microseconds),
        @(components.nanoseconds),
        @(components.sign),
        @(components.sign == 0 ? 1 : 0)  // blank = true if sign is 0
    ];
}

- (NSString *)durationAdd:(NSString *)a b:(NSString *)b {
    if (a == nil || b == nil) {
        THROW_TYPE_ERROR(@"Duration arguments cannot be null");
    }
    const char *aCStr = [a UTF8String];
    const char *bCStr = [b UTF8String];
    if (aCStr == NULL || bCStr == NULL) {
        THROW_TYPE_ERROR(@"Invalid duration string encoding");
    }

    TemporalResult result = temporal_duration_add(aCStr, bCStr);
    return extractResultValue(result);
}

- (NSString *)durationSubtract:(NSString *)a b:(NSString *)b {
    if (a == nil || b == nil) {
        THROW_TYPE_ERROR(@"Duration arguments cannot be null");
    }
    const char *aCStr = [a UTF8String];
    const char *bCStr = [b UTF8String];
    if (aCStr == NULL || bCStr == NULL) {
        THROW_TYPE_ERROR(@"Invalid duration string encoding");
    }

    TemporalResult result = temporal_duration_subtract(aCStr, bCStr);
    return extractResultValue(result);
}

- (NSString *)durationNegated:(NSString *)input {
    if (input == nil) {
        THROW_TYPE_ERROR(@"Duration string cannot be null");
    }
    const char *inputCStr = [input UTF8String];
    if (inputCStr == NULL) {
        THROW_TYPE_ERROR(@"Invalid duration string encoding");
    }

    TemporalResult result = temporal_duration_negated(inputCStr);
    return extractResultValue(result);
}

- (NSString *)durationAbs:(NSString *)input {
    if (input == nil) {
        THROW_TYPE_ERROR(@"Duration string cannot be null");
    }
    const char *inputCStr = [input UTF8String];
    if (inputCStr == NULL) {
        THROW_TYPE_ERROR(@"Invalid duration string encoding");
    }

    TemporalResult result = temporal_duration_abs(inputCStr);
    return extractResultValue(result);
}

- (double)durationCompare:(NSString *)a b:(NSString *)b {
    if (a == nil || b == nil) {
        THROW_TYPE_ERROR(@"Duration arguments cannot be null");
    }
    const char *aCStr = [a UTF8String];
    const char *bCStr = [b UTF8String];
    if (aCStr == NULL || bCStr == NULL) {
        THROW_TYPE_ERROR(@"Invalid duration string encoding");
    }

    CompareResult result = temporal_duration_compare(aCStr, bCStr);

    if (result.error_type != TEMPORAL_ERROR_NONE) {
        throwCompareError(&result);
        return 0; // Never reached
    }

    double value = (double)result.value;
    temporal_free_compare_result(&result);
    return value;
}

- (NSString *)durationWith:(NSString *)original
                     years:(double)years
                    months:(double)months
                     weeks:(double)weeks
                      days:(double)days
                     hours:(double)hours
                   minutes:(double)minutes
                   seconds:(double)seconds
              milliseconds:(double)milliseconds
              microseconds:(double)microseconds
               nanoseconds:(double)nanoseconds {
    if (original == nil) {
        THROW_TYPE_ERROR(@"Duration string cannot be null");
    }
    const char *originalCStr = [original UTF8String];
    if (originalCStr == NULL) {
        THROW_TYPE_ERROR(@"Invalid duration string encoding");
    }

    TemporalResult result = temporal_duration_with(
        originalCStr,
        (int64_t)years,
        (int64_t)months,
        (int64_t)weeks,
        (int64_t)days,
        (int64_t)hours,
        (int64_t)minutes,
        (int64_t)seconds,
        (int64_t)milliseconds,
        (int64_t)microseconds,
        (int64_t)nanoseconds
    );
    return extractResultValue(result);
}


// TimeZone methods

- (NSString *)timeZoneFromString:(NSString *)s {
    if (s == nil) THROW_TYPE_ERROR(@"String cannot be null");
    TemporalResult result = temporal_time_zone_from_string([s UTF8String]);
    return extractResultValue(result);
}

- (NSString *)timeZoneGetId:(NSString *)s {
    if (s == nil) THROW_TYPE_ERROR(@"String cannot be null");
    TemporalResult result = temporal_time_zone_get_id([s UTF8String]);
    return extractResultValue(result);
}

- (double)timeZoneGetOffsetNanosecondsFor:(NSString *)tzId instant:(NSString *)instantStr {
    if (!tzId || !instantStr) THROW_TYPE_ERROR(@"Arguments cannot be null");
    TemporalResult result = temporal_time_zone_get_offset_nanoseconds_for([tzId UTF8String], [instantStr UTF8String]);
    NSString *val = extractResultValue(result);
    return [val doubleValue];
}

- (NSString *)timeZoneGetOffsetStringFor:(NSString *)tzId instant:(NSString *)instantStr {
    if (!tzId || !instantStr) THROW_TYPE_ERROR(@"Arguments cannot be null");
    TemporalResult result = temporal_time_zone_get_offset_string_for([tzId UTF8String], [instantStr UTF8String]);
    return extractResultValue(result);
}

- (NSString *)timeZoneGetPlainDateTimeFor:(NSString *)tzId instant:(NSString *)instantStr calendarId:(NSString *)calendarId {
    if (!tzId || !instantStr) THROW_TYPE_ERROR(@"Arguments cannot be null");
    const char *cIdCStr = calendarId ? [calendarId UTF8String] : NULL;
    TemporalResult result = temporal_time_zone_get_plain_date_time_for([tzId UTF8String], [instantStr UTF8String], cIdCStr);
    return extractResultValue(result);
}

- (NSString *)timeZoneGetInstantFor:(NSString *)tzId dt:(NSString *)dtStr disambiguation:(NSString *)disambiguation {
    if (!tzId || !dtStr) THROW_TYPE_ERROR(@"Arguments cannot be null");
    const char *disCStr = disambiguation ? [disambiguation UTF8String] : NULL;
    TemporalResult result = temporal_time_zone_get_instant_for([tzId UTF8String], [dtStr UTF8String], disCStr);
    return extractResultValue(result);
}

- (NSString *)timeZoneGetNextTransition:(NSString *)tzId instant:(NSString *)instantStr {
    if (!tzId || !instantStr) THROW_TYPE_ERROR(@"Arguments cannot be null");
    TemporalResult result = temporal_time_zone_get_next_transition([tzId UTF8String], [instantStr UTF8String]);
    NSString *val = extractResultValue(result);
    return [val length] > 0 ? val : nil;
}

- (NSString *)timeZoneGetPreviousTransition:(NSString *)tzId instant:(NSString *)instantStr {
    if (!tzId || !instantStr) THROW_TYPE_ERROR(@"Arguments cannot be null");
    TemporalResult result = temporal_time_zone_get_previous_transition([tzId UTF8String], [instantStr UTF8String]);
    NSString *val = extractResultValue(result);
    return [val length] > 0 ? val : nil;
}

// ZonedDateTime methods

- (NSString *)zonedDateTimeFromString:(NSString *)s {
    if (s == nil) THROW_TYPE_ERROR(@"String cannot be null");
    TemporalResult result = temporal_zoned_date_time_from_string([s UTF8String]);
    return extractResultValue(result);
}

- (NSString *)zonedDateTimeFromComponents:(double)year month:(double)month day:(double)day hour:(double)hour minute:(double)minute second:(double)second millisecond:(double)millisecond microsecond:(double)microsecond nanosecond:(double)nanosecond calendarId:(NSString *)calendarId timeZoneId:(NSString *)timeZoneId offsetNanoseconds:(double)offsetNanoseconds {
    const char *cIdCStr = calendarId ? [calendarId UTF8String] : NULL;
    const char *tzIdCStr = timeZoneId ? [timeZoneId UTF8String] : NULL;
    
    TemporalResult result = temporal_zoned_date_time_from_components(
        (int32_t)year, (uint8_t)month, (uint8_t)day,
        (uint8_t)hour, (uint8_t)minute, (uint8_t)second,
        (uint16_t)millisecond, (uint16_t)microsecond, (uint16_t)nanosecond,
        cIdCStr, tzIdCStr, (int64_t)offsetNanoseconds
    );
    return extractResultValue(result);
}

- (NSArray<NSNumber *> *)zonedDateTimeGetAllComponents:(NSString *)s {
    if (s == nil) THROW_TYPE_ERROR(@"String cannot be null");
    
    ZonedDateTimeComponents c;
    temporal_zoned_date_time_get_components([s UTF8String], &c);
    
    if (c.is_valid == 0) {
        THROW_RANGE_ERROR(@"Invalid zoned date time");
    }
    
    return @[
        @(c.year), @(c.month), @(c.day),
        @(c.day_of_week), @(c.day_of_year), @(c.week_of_year), @(c.year_of_week),
        @(c.days_in_week), @(c.days_in_month), @(c.days_in_year), @(c.months_in_year),
        @(c.in_leap_year),
        @(c.hour), @(c.minute), @(c.second),
        @(c.millisecond), @(c.microsecond), @(c.nanosecond),
        @(c.offset_nanoseconds)
    ];
}

- (double)zonedDateTimeEpochMilliseconds:(NSString *)s {
    if (s == nil) THROW_TYPE_ERROR(@"String cannot be null");
    TemporalResult result = temporal_zoned_date_time_epoch_milliseconds([s UTF8String]);
    NSString *val = extractResultValue(result);
    return [val doubleValue];
}

- (NSString *)zonedDateTimeEpochNanoseconds:(NSString *)s {
    if (s == nil) THROW_TYPE_ERROR(@"String cannot be null");
    TemporalResult result = temporal_zoned_date_time_epoch_nanoseconds([s UTF8String]);
    return extractResultValue(result);
}

- (NSString *)zonedDateTimeGetCalendar:(NSString *)s {
    if (s == nil) return @"";
    TemporalResult result = temporal_zoned_date_time_get_calendar([s UTF8String]);
    return extractResultValue(result);
}

- (NSString *)zonedDateTimeGetTimeZone:(NSString *)s {
    if (s == nil) return @"";
    TemporalResult result = temporal_zoned_date_time_get_time_zone([s UTF8String]);
    return extractResultValue(result);
}

- (NSString *)zonedDateTimeGetOffset:(NSString *)s {
    if (s == nil) return @"";
    TemporalResult result = temporal_zoned_date_time_get_offset([s UTF8String]);
    return extractResultValue(result);
}

- (NSString *)zonedDateTimeAdd:(NSString *)zdt duration:(NSString *)duration {
    if (!zdt || !duration) THROW_TYPE_ERROR(@"Arguments cannot be null");
    TemporalResult result = temporal_zoned_date_time_add([zdt UTF8String], [duration UTF8String]);
    return extractResultValue(result);
}

- (NSString *)zonedDateTimeSubtract:(NSString *)zdt duration:(NSString *)duration {
    if (!zdt || !duration) THROW_TYPE_ERROR(@"Arguments cannot be null");
    TemporalResult result = temporal_zoned_date_time_subtract([zdt UTF8String], [duration UTF8String]);
    return extractResultValue(result);
}

- (double)zonedDateTimeCompare:(NSString *)a b:(NSString *)b {
    if (!a || !b) THROW_TYPE_ERROR(@"Arguments cannot be null");
    CompareResult result = temporal_zoned_date_time_compare([a UTF8String], [b UTF8String]);
    if (result.error_type != TEMPORAL_ERROR_NONE) {
        throwCompareError(&result);
        return 0;
    }
    double val = (double)result.value;
    temporal_free_compare_result(&result);
    return val;
}

- (NSString *)zonedDateTimeWith:(NSString *)zdt year:(double)year month:(double)month day:(double)day hour:(double)hour minute:(double)minute second:(double)second millisecond:(double)millisecond microsecond:(double)microsecond nanosecond:(double)nanosecond offsetNs:(double)offsetNs calendarId:(NSString *)calendarId timeZoneId:(NSString *)timeZoneId {
    if (zdt == nil) THROW_TYPE_ERROR(@"String cannot be null");
    const char *cIdCStr = calendarId ? [calendarId UTF8String] : NULL;
    const char *tzIdCStr = timeZoneId ? [timeZoneId UTF8String] : NULL;
    
    TemporalResult result = temporal_zoned_date_time_with(
        [zdt UTF8String],
        (int32_t)year, (int32_t)month, (int32_t)day,
        (int32_t)hour, (int32_t)minute, (int32_t)second,
        (int32_t)millisecond, (int32_t)microsecond, (int32_t)nanosecond,
        (int64_t)offsetNs,
        cIdCStr, tzIdCStr
    );
    return extractResultValue(result);
}

- (NSString *)zonedDateTimeUntil:(NSString *)one two:(NSString *)two {
    if (!one || !two) THROW_TYPE_ERROR(@"Arguments cannot be null");
    TemporalResult result = temporal_zoned_date_time_until([one UTF8String], [two UTF8String]);
    return extractResultValue(result);
}

- (NSString *)zonedDateTimeSince:(NSString *)one two:(NSString *)two {
    if (!one || !two) THROW_TYPE_ERROR(@"Arguments cannot be null");
    TemporalResult result = temporal_zoned_date_time_since([one UTF8String], [two UTF8String]);
    return extractResultValue(result);
}

- (NSString *)zonedDateTimeRound:(NSString *)zdt smallestUnit:(NSString *)smallestUnit roundingIncrement:(double)roundingIncrement roundingMode:(NSString *)roundingMode {
    if (!zdt || !smallestUnit) THROW_TYPE_ERROR(@"Arguments cannot be null");
    const char *zdtCStr = [zdt UTF8String];
    const char *unitCStr = [smallestUnit UTF8String];
    const char *modeCStr = roundingMode ? [roundingMode UTF8String] : NULL;
    
    TemporalResult result = temporal_zoned_date_time_round(zdtCStr, unitCStr, (int64_t)roundingIncrement, modeCStr);
    return extractResultValue(result);
}

- (NSString *)zonedDateTimeToInstant:(NSString *)s {
    if (s == nil) THROW_TYPE_ERROR(@"String cannot be null");
    TemporalResult result = temporal_zoned_date_time_to_instant([s UTF8String]);
    return extractResultValue(result);
}

- (NSString *)zonedDateTimeToPlainDate:(NSString *)s {
    if (s == nil) THROW_TYPE_ERROR(@"String cannot be null");
    TemporalResult result = temporal_zoned_date_time_to_plain_date([s UTF8String]);
    return extractResultValue(result);
}

- (NSString *)zonedDateTimeToPlainTime:(NSString *)s {
    if (s == nil) THROW_TYPE_ERROR(@"String cannot be null");
    TemporalResult result = temporal_zoned_date_time_to_plain_time([s UTF8String]);
    return extractResultValue(result);
}

- (NSString *)zonedDateTimeToPlainDateTime:(NSString *)s {
    if (s == nil) THROW_TYPE_ERROR(@"String cannot be null");
    TemporalResult result = temporal_zoned_date_time_to_plain_date_time([s UTF8String]);
    return extractResultValue(result);
}

- (std::shared_ptr<facebook::react::TurboModule>)getTurboModule:

    (const facebook::react::ObjCTurboModule::InitParams &)params
{
    return std::make_shared<facebook::react::NativeTemporalSpecJSI>(params);
}

+ (NSString *)moduleName
{
    return @"Temporal";
}

@end

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

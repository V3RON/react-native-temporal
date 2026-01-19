import { TurboModuleRegistry, type TurboModule } from 'react-native';

export interface Spec extends TurboModule {
  multiply(a: number, b: number): number;
  instantNow(): string;
  instantFromString(s: string): string;
  instantFromEpochMilliseconds(ms: number): string;
  instantFromEpochNanoseconds(ns: string): string;
  instantEpochMilliseconds(s: string): number;
  instantEpochNanoseconds(s: string): string;
  instantAdd(instant: string, duration: string): string;
  instantSubtract(instant: string, duration: string): string;
  instantCompare(one: string, two: string): number;

  // Now methods
  nowTimeZoneId(): string;
  nowPlainDateTimeISO(tz?: string): string;
  nowPlainDateISO(tz?: string): string;
  nowPlainTimeISO(tz?: string): string;
  nowZonedDateTimeISO(tz?: string): string;

  // PlainTime methods
  plainTimeFromString(s: string): string;
  plainTimeFromComponents(
    hour: number,
    minute: number,
    second: number,
    millisecond: number,
    microsecond: number,
    nanosecond: number
  ): string;
  plainTimeGetAllComponents(s: string): number[];
  plainTimeAdd(time: string, duration: string): string;
  plainTimeSubtract(time: string, duration: string): string;
  plainTimeCompare(one: string, two: string): number;

  // PlainDate methods
  plainDateFromString(s: string): string;
  plainDateFromComponents(
    year: number,
    month: number,
    day: number,
    calendarId: string | null
  ): string;
  plainDateGetAllComponents(s: string): number[];
  plainDateGetMonthCode(s: string): string;
  plainDateGetCalendar(s: string): string;
  plainDateAdd(date: string, duration: string): string;
  plainDateSubtract(date: string, duration: string): string;
  plainDateCompare(a: string, b: string): number;
  plainDateWith(
    date: string,
    year: number,
    month: number,
    day: number,
    calendarId: string | null
  ): string;
  plainDateUntil(one: string, two: string): string;
  plainDateSince(one: string, two: string): string;

  // PlainDateTime methods
  plainDateTimeFromString(s: string): string;
  plainDateTimeFromComponents(
    year: number,
    month: number,
    day: number,
    hour: number,
    minute: number,
    second: number,
    millisecond: number,
    microsecond: number,
    nanosecond: number,
    calendarId: string | null
  ): string;
  plainDateTimeGetAllComponents(s: string): number[];
  plainDateTimeGetMonthCode(s: string): string;
  plainDateTimeGetCalendar(s: string): string;
  plainDateTimeAdd(dt: string, duration: string): string;
  plainDateTimeSubtract(dt: string, duration: string): string;
  plainDateTimeCompare(a: string, b: string): number;
  plainDateTimeWith(
    dt: string,
    year: number,
    month: number,
    day: number,
    hour: number,
    minute: number,
    second: number,
    millisecond: number,
    microsecond: number,
    nanosecond: number,
    calendarId: string | null
  ): string;
  plainDateTimeUntil(one: string, two: string): string;
  plainDateTimeSince(one: string, two: string): string;

  // PlainYearMonth methods
  plainYearMonthFromString(s: string): string;
  plainYearMonthFromComponents(
    year: number,
    month: number,
    calendarId: string | null,
    referenceDay: number
  ): string;
  plainYearMonthGetAllComponents(s: string): number[];
  plainYearMonthGetMonthCode(s: string): string;
  plainYearMonthGetCalendar(s: string): string;
  plainYearMonthAdd(ym: string, duration: string): string;
  plainYearMonthSubtract(ym: string, duration: string): string;
  plainYearMonthCompare(a: string, b: string): number;
  plainYearMonthWith(
    ym: string,
    year: number,
    month: number,
    calendarId: string | null
  ): string;
  plainYearMonthUntil(one: string, two: string): string;
  plainYearMonthSince(one: string, two: string): string;
  plainYearMonthToPlainDate(ym: string, day: number): string;

  // PlainMonthDay methods
  plainMonthDayFromString(s: string): string;
  plainMonthDayFromComponents(
    month: number,
    day: number,
    calendarId: string | null,
    referenceYear: number
  ): string;
  plainMonthDayGetAllComponents(s: string): number[];
  plainMonthDayGetMonthCode(s: string): string;
  plainMonthDayGetCalendar(s: string): string;
  plainMonthDayToPlainDate(md: string, year: number): string;

  // TimeZone methods
  timeZoneFromString(s: string): string;
  timeZoneGetId(s: string): string;
  timeZoneGetOffsetNanosecondsFor(tzId: string, instantStr: string): number;
  timeZoneGetOffsetStringFor(tzId: string, instantStr: string): string;
  timeZoneGetPlainDateTimeFor(
    tzId: string,
    instantStr: string,
    calendarId: string | null
  ): string;
  timeZoneGetInstantFor(
    tzId: string,
    dtStr: string,
    disambiguation: string | null
  ): string;
  timeZoneGetNextTransition(tzId: string, instantStr: string): string | null;
  timeZoneGetPreviousTransition(
    tzId: string,
    instantStr: string
  ): string | null;

  // ZonedDateTime methods
  zonedDateTimeFromString(s: string): string;
  zonedDateTimeFromComponents(
    year: number,
    month: number,
    day: number,
    hour: number,
    minute: number,
    second: number,
    millisecond: number,
    microsecond: number,
    nanosecond: number,
    calendarId: string | null,
    timeZoneId: string,
    offsetNanoseconds: number
  ): string;
  zonedDateTimeGetAllComponents(s: string): number[];
  zonedDateTimeEpochMilliseconds(s: string): number;
  zonedDateTimeEpochNanoseconds(s: string): string;
  zonedDateTimeGetCalendar(s: string): string;
  zonedDateTimeGetTimeZone(s: string): string;
  zonedDateTimeGetOffset(s: string): string;
  zonedDateTimeAdd(zdt: string, duration: string): string;
  zonedDateTimeSubtract(zdt: string, duration: string): string;
  zonedDateTimeCompare(a: string, b: string): number;
  zonedDateTimeWith(
    zdt: string,
    year: number,
    month: number,
    day: number,
    hour: number,
    minute: number,
    second: number,
    millisecond: number,
    microsecond: number,
    nanosecond: number,
    offsetNs: number,
    calendarId: string | null,
    timeZoneId: string | null
  ): string;
  zonedDateTimeUntil(one: string, two: string): string;
  zonedDateTimeSince(one: string, two: string): string;
  zonedDateTimeRound(
    zdt: string,
    smallestUnit: string,
    roundingIncrement: number,
    roundingMode: string | null
  ): string;
  zonedDateTimeToInstant(s: string): string;
  zonedDateTimeToPlainDate(s: string): string;
  zonedDateTimeToPlainTime(s: string): string;
  zonedDateTimeToPlainDateTime(s: string): string;

  // Calendar methods
  calendarFrom(id: string): string;
  calendarId(id: string): string;

  // Duration methods - minimal bridge, all logic in native

  /**
   * Parses an ISO 8601 duration string and returns the normalized ISO string.
   * Throws RangeError for invalid format, TypeError for null input.
   */
  durationFromString(input: string): string;

  /**
   * Creates a duration from individual component values.
   * Returns the ISO 8601 string representation.
   * Throws RangeError if values have mixed signs or are invalid.
   */
  durationFromComponents(
    years: number,
    months: number,
    weeks: number,
    days: number,
    hours: number,
    minutes: number,
    seconds: number,
    milliseconds: number,
    microseconds: number,
    nanoseconds: number
  ): string;

  /**
   * Gets all component values from a duration string in a single call.
   * Returns array: [years, months, weeks, days, hours, minutes, seconds,
   *                 milliseconds, microseconds, nanoseconds, sign, blank]
   * Throws RangeError for invalid duration, TypeError for null input.
   */
  durationGetAllComponents(durationStr: string): number[];

  /**
   * Adds two durations and returns the result as an ISO string.
   */
  durationAdd(a: string, b: string): string;

  /**
   * Subtracts duration b from a and returns the result as an ISO string.
   */
  durationSubtract(a: string, b: string): string;

  /**
   * Negates a duration and returns the result as an ISO string.
   */
  durationNegated(input: string): string;

  /**
   * Gets the absolute value of a duration and returns the result as an ISO string.
   */
  durationAbs(input: string): string;

  /**
   * Compares two durations. Returns -1, 0, or 1.
   * Note: Durations with years, months, or weeks require relativeTo (not yet supported).
   */
  durationCompare(a: string, b: string): number;

  /**
   * Creates a new duration by replacing specified components.
   * Pass Number.MIN_SAFE_INTEGER for components that should not be changed.
   */
  durationWith(
    original: string,
    years: number,
    months: number,
    weeks: number,
    days: number,
    hours: number,
    minutes: number,
    seconds: number,
    milliseconds: number,
    microseconds: number,
    nanoseconds: number
  ): string;
}

export default TurboModuleRegistry.getEnforcing<Spec>('Temporal');

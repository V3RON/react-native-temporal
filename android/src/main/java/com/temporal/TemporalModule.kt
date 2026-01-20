package com.temporal

import com.facebook.react.bridge.ReactApplicationContext
import com.facebook.react.bridge.WritableArray
import com.facebook.react.bridge.WritableNativeArray
import com.facebook.react.module.annotations.ReactModule

/**
 * Custom exception for Temporal RangeError (matches TC39 Temporal spec)
 */
class TemporalRangeError(message: String) : Exception(message)

/**
 * Custom exception for Temporal TypeError (matches TC39 Temporal spec)
 */
class TemporalTypeError(message: String) : Exception(message)

@ReactModule(name = TemporalModule.NAME)
class TemporalModule(reactContext: ReactApplicationContext) :
  NativeTemporalSpec(reactContext) {

  override fun getName(): String {
    return NAME
  }

  override fun multiply(a: Double, b: Double): Double {
    return a * b
  }

  override fun instantNow(): String {
    return TemporalNative.instantNow()
  }

  override fun instantFromString(s: String): String {
    return TemporalNative.instantFromString(s)
  }

  override fun instantFromEpochMilliseconds(ms: Double): String {
    return TemporalNative.instantFromEpochMilliseconds(ms.toLong())
  }

  override fun instantFromEpochNanoseconds(nsStr: String): String {
    return TemporalNative.instantFromEpochNanoseconds(nsStr)
  }

  override fun instantEpochMilliseconds(s: String): Double {
    // Return as double (React Native doesn't support Long well)
    val msStr = TemporalNative.instantEpochMilliseconds(s)
    return msStr.toDouble()
  }

  override fun instantEpochNanoseconds(s: String): String {
    return TemporalNative.instantEpochNanoseconds(s)
  }

  override fun instantAdd(instant: String, duration: String): String {
    return TemporalNative.instantAdd(instant, duration)
  }

  override fun instantSubtract(instant: String, duration: String): String {
    return TemporalNative.instantSubtract(instant, duration)
  }

  override fun instantCompare(one: String, two: String): Double {
    return TemporalNative.instantCompare(one, two).toDouble()
  }

  override fun instantUntil(one: String, two: String, largestUnit: String?, smallestUnit: String?, roundingIncrement: Double, roundingMode: String?): String {
    return TemporalNative.instantUntil(one, two, largestUnit, smallestUnit, roundingIncrement.toLong(), roundingMode)
  }

  override fun instantSince(one: String, two: String, largestUnit: String?, smallestUnit: String?, roundingIncrement: Double, roundingMode: String?): String {
    return TemporalNative.instantSince(one, two, largestUnit, smallestUnit, roundingIncrement.toLong(), roundingMode)
  }

  override fun instantRound(instant: String, smallestUnit: String, roundingIncrement: Double, roundingMode: String?): String {
    return TemporalNative.instantRound(instant, smallestUnit, roundingIncrement.toLong(), roundingMode)
  }

  override fun instantToZonedDateTime(instant: String, calendarId: String?, timeZoneId: String): String {
    return TemporalNative.instantToZonedDateTime(instant, calendarId, timeZoneId)
  }

  // Now methods

  override fun nowTimeZoneId(): String {
    return java.util.TimeZone.getDefault().id
  }

  override fun nowPlainDateTimeISO(tz: String?): String {
    val tzId = tz ?: java.util.TimeZone.getDefault().id
    return TemporalNative.nowPlainDateTimeISO(tzId)
  }

  override fun nowPlainDateISO(tz: String?): String {
    val tzId = tz ?: java.util.TimeZone.getDefault().id
    return TemporalNative.nowPlainDateISO(tzId)
  }

  override fun nowPlainTimeISO(tz: String?): String {
    val tzId = tz ?: java.util.TimeZone.getDefault().id
    return TemporalNative.nowPlainTimeISO(tzId)
  }

  override fun nowZonedDateTimeISO(tz: String?): String {
    val tzId = tz ?: java.util.TimeZone.getDefault().id
    return TemporalNative.nowZonedDateTimeISO(tzId)
  }

  // PlainTime methods

  override fun plainTimeFromString(s: String): String {
    return TemporalNative.plainTimeFromString(s)
  }

  override fun plainTimeFromComponents(
    hour: Double,
    minute: Double,
    second: Double,
    millisecond: Double,
    microsecond: Double,
    nanosecond: Double
  ): String {
    return TemporalNative.plainTimeFromComponents(
      hour.toInt(),
      minute.toInt(),
      second.toInt(),
      millisecond.toInt(),
      microsecond.toInt(),
      nanosecond.toInt()
    )
  }

  override fun plainTimeGetAllComponents(s: String): WritableArray {
    val components = TemporalNative.plainTimeGetAllComponents(s)
    val result = WritableNativeArray()
    for (value in components) {
      result.pushDouble(value.toDouble())
    }
    return result
  }

  override fun plainTimeAdd(time: String, duration: String): String {
    return TemporalNative.plainTimeAdd(time, duration)
  }

  override fun plainTimeSubtract(time: String, duration: String): String {
    return TemporalNative.plainTimeSubtract(time, duration)
  }

  override fun plainTimeCompare(one: String, two: String): Double {
    return TemporalNative.plainTimeCompare(one, two).toDouble()
  }

  override fun plainTimeUntil(one: String, two: String, largestUnit: String?, smallestUnit: String?, roundingIncrement: Double, roundingMode: String?): String {
    return TemporalNative.plainTimeUntil(one, two, largestUnit, smallestUnit, roundingIncrement.toLong(), roundingMode)
  }

  override fun plainTimeSince(one: String, two: String, largestUnit: String?, smallestUnit: String?, roundingIncrement: Double, roundingMode: String?): String {
    return TemporalNative.plainTimeSince(one, two, largestUnit, smallestUnit, roundingIncrement.toLong(), roundingMode)
  }

  override fun plainTimeRound(time: String, smallestUnit: String, roundingIncrement: Double, roundingMode: String?): String {
    return TemporalNative.plainTimeRound(time, smallestUnit, roundingIncrement.toLong(), roundingMode)
  }

  // PlainDate methods

  override fun plainDateFromString(s: String): String {
    return TemporalNative.plainDateFromString(s)
  }

  override fun plainDateFromComponents(
    year: Double,
    month: Double,
    day: Double,
    calendarId: String?
  ): String {
    return TemporalNative.plainDateFromComponents(year.toInt(), month.toInt(), day.toInt(), calendarId)
  }

  override fun plainDateGetAllComponents(s: String): WritableArray {
    val components = TemporalNative.plainDateGetAllComponents(s)
    val result = WritableNativeArray()
    for (value in components) {
      result.pushDouble(value.toDouble())
    }
    return result
  }

  override fun plainDateGetMonthCode(s: String): String {
    return TemporalNative.plainDateGetMonthCode(s)
  }

  override fun plainDateGetCalendar(s: String): String {
    return TemporalNative.plainDateGetCalendar(s)
  }

  override fun plainDateAdd(date: String, duration: String): String {
    return TemporalNative.plainDateAdd(date, duration)
  }

  override fun plainDateSubtract(date: String, duration: String): String {
    return TemporalNative.plainDateSubtract(date, duration)
  }

  override fun plainDateCompare(a: String, b: String): Double {
    return TemporalNative.plainDateCompare(a, b).toDouble()
  }

  override fun plainDateWith(
    date: String,
    year: Double,
    month: Double,
    day: Double,
    calendarId: String?
  ): String {
    return TemporalNative.plainDateWith(date, year.toInt(), month.toInt(), day.toInt(), calendarId)
  }

  override fun plainDateUntil(one: String, two: String): String {
    return TemporalNative.plainDateUntil(one, two)
  }

  override fun plainDateSince(one: String, two: String): String {
    return TemporalNative.plainDateSince(one, two)
  }

  // PlainDateTime methods

  override fun plainDateTimeFromString(s: String): String {
    return TemporalNative.plainDateTimeFromString(s)
  }

  override fun plainDateTimeFromComponents(
    year: Double, month: Double, day: Double,
    hour: Double, minute: Double, second: Double,
    millisecond: Double, microsecond: Double, nanosecond: Double,
    calendarId: String?
  ): String {
    return TemporalNative.plainDateTimeFromComponents(
      year.toInt(), month.toInt(), day.toInt(),
      hour.toInt(), minute.toInt(), second.toInt(),
      millisecond.toInt(), microsecond.toInt(), nanosecond.toInt(),
      calendarId
    )
  }

  override fun plainDateTimeGetAllComponents(s: String): WritableArray {
    val components = TemporalNative.plainDateTimeGetAllComponents(s)
    val result = WritableNativeArray()
    for (value in components) {
      result.pushDouble(value.toDouble())
    }
    return result
  }

  override fun plainDateTimeGetMonthCode(s: String): String {
    return TemporalNative.plainDateTimeGetMonthCode(s)
  }

  override fun plainDateTimeGetCalendar(s: String): String {
    return TemporalNative.plainDateTimeGetCalendar(s)
  }

  override fun plainDateTimeAdd(dt: String, duration: String): String {
    return TemporalNative.plainDateTimeAdd(dt, duration)
  }

  override fun plainDateTimeSubtract(dt: String, duration: String): String {
    return TemporalNative.plainDateTimeSubtract(dt, duration)
  }

  override fun plainDateTimeCompare(a: String, b: String): Double {
    return TemporalNative.plainDateTimeCompare(a, b).toDouble()
  }

  override fun plainDateTimeWith(
    dt: String,
    year: Double, month: Double, day: Double,
    hour: Double, minute: Double, second: Double,
    millisecond: Double, microsecond: Double, nanosecond: Double,
    calendarId: String?
  ): String {
    return TemporalNative.plainDateTimeWith(
      dt,
      year.toInt(), month.toInt(), day.toInt(),
      hour.toInt(), minute.toInt(), second.toInt(),
      millisecond.toInt(), microsecond.toInt(), nanosecond.toInt(),
      calendarId
    )
  }

  override fun plainDateTimeUntil(one: String, two: String): String {
    return TemporalNative.plainDateTimeUntil(one, two)
  }

  override fun plainDateTimeSince(one: String, two: String): String {
    return TemporalNative.plainDateTimeSince(one, two)
  }

  // PlainYearMonth methods

  override fun plainYearMonthFromString(s: String): String {
    return TemporalNative.plainYearMonthFromString(s)
  }

  override fun plainYearMonthFromComponents(year: Double, month: Double, calendarId: String?, referenceDay: Double): String {
    return TemporalNative.plainYearMonthFromComponents(year.toInt(), month.toInt(), calendarId, referenceDay.toInt())
  }

  override fun plainYearMonthGetAllComponents(s: String): WritableArray {
    val components = TemporalNative.plainYearMonthGetAllComponents(s)
    val result = WritableNativeArray()
    for (value in components) {
      result.pushDouble(value.toDouble())
    }
    return result
  }

  override fun plainYearMonthGetMonthCode(s: String): String {
    return TemporalNative.plainYearMonthGetMonthCode(s)
  }

  override fun plainYearMonthGetCalendar(s: String): String {
    return TemporalNative.plainYearMonthGetCalendar(s)
  }

  override fun plainYearMonthAdd(ym: String, duration: String): String {
    return TemporalNative.plainYearMonthAdd(ym, duration)
  }

  override fun plainYearMonthSubtract(ym: String, duration: String): String {
    return TemporalNative.plainYearMonthSubtract(ym, duration)
  }

  override fun plainYearMonthCompare(a: String, b: String): Double {
    return TemporalNative.plainYearMonthCompare(a, b).toDouble()
  }

  override fun plainYearMonthWith(ym: String, year: Double, month: Double, calendarId: String?): String {
    return TemporalNative.plainYearMonthWith(ym, year.toInt(), month.toInt(), calendarId)
  }

  override fun plainYearMonthUntil(one: String, two: String): String {
    return TemporalNative.plainYearMonthUntil(one, two)
  }

  override fun plainYearMonthSince(one: String, two: String): String {
    return TemporalNative.plainYearMonthSince(one, two)
  }

  override fun plainYearMonthToPlainDate(ym: String, day: Double): String {
    return TemporalNative.plainYearMonthToPlainDate(ym, day.toInt())
  }

  // PlainMonthDay methods

  override fun plainMonthDayFromString(s: String): String {
    return TemporalNative.plainMonthDayFromString(s)
  }

  override fun plainMonthDayFromComponents(month: Double, day: Double, calendarId: String?, referenceYear: Double): String {
    return TemporalNative.plainMonthDayFromComponents(month.toInt(), day.toInt(), calendarId, referenceYear.toInt())
  }

  override fun plainMonthDayGetAllComponents(s: String): WritableArray {
    val components = TemporalNative.plainMonthDayGetAllComponents(s)
    val result = WritableNativeArray()
    for (value in components) {
      result.pushDouble(value.toDouble())
    }
    return result
  }

  override fun plainMonthDayGetMonthCode(s: String): String {
    return TemporalNative.plainMonthDayGetMonthCode(s)
  }

  override fun plainMonthDayGetCalendar(s: String): String {
    return TemporalNative.plainMonthDayGetCalendar(s)
  }

  override fun plainMonthDayToPlainDate(md: String, year: Double): String {
    return TemporalNative.plainMonthDayToPlainDate(md, year.toInt())
  }

  // TimeZone methods

  override fun timeZoneFromString(s: String): String {
    return TemporalNative.timeZoneFromString(s)
  }

  override fun timeZoneGetId(s: String): String {
    return TemporalNative.timeZoneGetId(s)
  }

  override fun timeZoneGetOffsetNanosecondsFor(tzId: String, instantStr: String): Double {
    return TemporalNative.timeZoneGetOffsetNanosecondsFor(tzId, instantStr).toDouble()
  }

  override fun timeZoneGetOffsetStringFor(tzId: String, instantStr: String): String {
    return TemporalNative.timeZoneGetOffsetStringFor(tzId, instantStr)
  }

  override fun timeZoneGetPlainDateTimeFor(tzId: String, instantStr: String, calendarId: String?): String {
    return TemporalNative.timeZoneGetPlainDateTimeFor(tzId, instantStr, calendarId)
  }

  override fun timeZoneGetInstantFor(tzId: String, dtStr: String, disambiguation: String?): String {
    return TemporalNative.timeZoneGetInstantFor(tzId, dtStr, disambiguation)
  }

  override fun timeZoneGetNextTransition(tzId: String, instantStr: String): String? {
    return TemporalNative.timeZoneGetNextTransition(tzId, instantStr)
  }

  override fun timeZoneGetPreviousTransition(tzId: String, instantStr: String): String? {
    return TemporalNative.timeZoneGetPreviousTransition(tzId, instantStr)
  }

  // ZonedDateTime methods

  override fun zonedDateTimeFromString(s: String): String {
    return TemporalNative.zonedDateTimeFromString(s)
  }

  override fun zonedDateTimeFromComponents(
    year: Double, month: Double, day: Double,
    hour: Double, minute: Double, second: Double,
    millisecond: Double, microsecond: Double, nanosecond: Double,
    calendarId: String?, timeZoneId: String, offsetNanoseconds: Double
  ): String {
    return TemporalNative.zonedDateTimeFromComponents(
      year.toInt(), month.toInt(), day.toInt(),
      hour.toInt(), minute.toInt(), second.toInt(),
      millisecond.toInt(), microsecond.toInt(), nanosecond.toInt(),
      calendarId, timeZoneId, offsetNanoseconds.toLong()
    )
  }

  override fun zonedDateTimeGetAllComponents(s: String): WritableArray {
    val components = TemporalNative.zonedDateTimeGetAllComponents(s)
    val result = WritableNativeArray()
    for (value in components) {
      result.pushDouble(value.toDouble())
    }
    return result
  }

  override fun zonedDateTimeEpochMilliseconds(s: String): Double {
    val msStr = TemporalNative.zonedDateTimeEpochMilliseconds(s)
    return msStr.toDouble()
  }

  override fun zonedDateTimeEpochNanoseconds(s: String): String {
    return TemporalNative.zonedDateTimeEpochNanoseconds(s)
  }

  override fun zonedDateTimeGetCalendar(s: String): String {
    return TemporalNative.zonedDateTimeGetCalendar(s)
  }

  override fun zonedDateTimeGetTimeZone(s: String): String {
    return TemporalNative.zonedDateTimeGetTimeZone(s)
  }

  override fun zonedDateTimeGetOffset(s: String): String {
    return TemporalNative.zonedDateTimeGetOffset(s)
  }

  override fun zonedDateTimeAdd(zdt: String, duration: String): String {
    return TemporalNative.zonedDateTimeAdd(zdt, duration)
  }

  override fun zonedDateTimeSubtract(zdt: String, duration: String): String {
    return TemporalNative.zonedDateTimeSubtract(zdt, duration)
  }

  override fun zonedDateTimeCompare(a: String, b: String): Double {
    return TemporalNative.zonedDateTimeCompare(a, b).toDouble()
  }

  override fun zonedDateTimeWith(
    zdt: String,
    year: Double, month: Double, day: Double,
    hour: Double, minute: Double, second: Double,
    millisecond: Double, microsecond: Double, nanosecond: Double,
    offsetNs: Double,
    calendarId: String?, timeZoneId: String?
  ): String {
    return TemporalNative.zonedDateTimeWith(
      zdt,
      year.toInt(), month.toInt(), day.toInt(),
      hour.toInt(), minute.toInt(), second.toInt(),
      millisecond.toInt(), microsecond.toInt(), nanosecond.toInt(),
      offsetNs.toLong(),
      calendarId, timeZoneId
    )
  }

  override fun zonedDateTimeUntil(one: String, two: String): String {
    return TemporalNative.zonedDateTimeUntil(one, two)
  }

  override fun zonedDateTimeSince(one: String, two: String): String {
    return TemporalNative.zonedDateTimeSince(one, two)
  }

  override fun zonedDateTimeRound(zdt: String, smallestUnit: String, roundingIncrement: Double, roundingMode: String?): String {
    return TemporalNative.zonedDateTimeRound(zdt, smallestUnit, roundingIncrement.toLong(), roundingMode)
  }

  override fun zonedDateTimeToInstant(s: String): String {
    return TemporalNative.zonedDateTimeToInstant(s)
  }

  override fun zonedDateTimeToPlainDate(s: String): String {
    return TemporalNative.zonedDateTimeToPlainDate(s)
  }

  override fun zonedDateTimeToPlainTime(s: String): String {
    return TemporalNative.zonedDateTimeToPlainTime(s)
  }

  override fun zonedDateTimeToPlainDateTime(s: String): String {
    return TemporalNative.zonedDateTimeToPlainDateTime(s)
  }

  // Calendar methods

  override fun calendarFrom(id: String): String {
    return TemporalNative.calendarFrom(id)
  }

  override fun calendarId(id: String): String {
    return TemporalNative.calendarId(id)
  }

  // Duration methods

  override fun durationFromString(input: String): String {
    return TemporalNative.durationFromString(input)
  }

  override fun durationFromComponents(
    years: Double,
    months: Double,
    weeks: Double,
    days: Double,
    hours: Double,
    minutes: Double,
    seconds: Double,
    milliseconds: Double,
    microseconds: Double,
    nanoseconds: Double
  ): String {
    return TemporalNative.durationFromComponents(
      years.toLong(),
      months.toLong(),
      weeks.toLong(),
      days.toLong(),
      hours.toLong(),
      minutes.toLong(),
      seconds.toLong(),
      milliseconds.toLong(),
      microseconds.toLong(),
      nanoseconds.toLong()
    )
  }

  override fun durationGetAllComponents(durationStr: String): WritableArray {
    val components = TemporalNative.durationGetAllComponents(durationStr)
    val result = WritableNativeArray()
    for (value in components) {
      result.pushDouble(value.toDouble())
    }
    return result
  }

  override fun durationAdd(a: String, b: String): String {
    return TemporalNative.durationAdd(a, b)
  }

  override fun durationSubtract(a: String, b: String): String {
    return TemporalNative.durationSubtract(a, b)
  }

  override fun durationNegated(input: String): String {
    return TemporalNative.durationNegated(input)
  }

  override fun durationAbs(input: String): String {
    return TemporalNative.durationAbs(input)
  }

  override fun durationCompare(a: String, b: String): Double {
    return TemporalNative.durationCompare(a, b).toDouble()
  }

  override fun durationWith(
    original: String,
    years: Double,
    months: Double,
    weeks: Double,
    days: Double,
    hours: Double,
    minutes: Double,
    seconds: Double,
    milliseconds: Double,
    microseconds: Double,
    nanoseconds: Double
  ): String {
    return TemporalNative.durationWith(
      original,
      years.toLong(),
      months.toLong(),
      weeks.toLong(),
      days.toLong(),
      hours.toLong(),
      minutes.toLong(),
      seconds.toLong(),
      milliseconds.toLong(),
      microseconds.toLong(),
      nanoseconds.toLong()
    )
  }

  companion object {
    const val NAME = "Temporal"
  }
}

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

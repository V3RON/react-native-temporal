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

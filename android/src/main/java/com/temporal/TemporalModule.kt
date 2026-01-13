package com.temporal

import com.facebook.react.bridge.ReactApplicationContext
import com.facebook.react.module.annotations.ReactModule

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
    return TemporalNative.instantNow() ?: ""
  }

  companion object {
    const val NAME = "Temporal"
  }
}

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
     */
    external fun instantNow(): String?
}

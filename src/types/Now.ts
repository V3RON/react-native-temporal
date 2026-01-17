import { Instant } from './Instant';
import NativeTemporal from '../NativeTemporal';
import { wrapNativeCall } from '../utils';

export const Now = {
  /**
   * Returns the current instant in the system time zone.
   *
   * @returns {Instant} The current instant.
   */
  instant: (): Instant => {
    return Instant.now();
  },

  /**
   * Returns the system time zone identifier.
   */
  timeZoneId: (): string => {
    return wrapNativeCall(
      () => NativeTemporal.nowTimeZoneId(),
      'Failed to get time zone ID'
    );
  },

  /**
   * Returns the current date and time as an ISO 8601 string.
   */
  plainDateTimeISO: (temporalTimeZoneLike?: string): string => {
    return wrapNativeCall(
      () => NativeTemporal.nowPlainDateTimeISO(temporalTimeZoneLike),
      'Failed to get plain date time'
    );
  },

  /**
   * Returns the current date as an ISO 8601 string.
   */
  plainDateISO: (temporalTimeZoneLike?: string): string => {
    return wrapNativeCall(
      () => NativeTemporal.nowPlainDateISO(temporalTimeZoneLike),
      'Failed to get plain date'
    );
  },

  /**
   * Returns the current time as an ISO 8601 string.
   */
  plainTimeISO: (temporalTimeZoneLike?: string): string => {
    return wrapNativeCall(
      () => NativeTemporal.nowPlainTimeISO(temporalTimeZoneLike),
      'Failed to get plain time'
    );
  },
};

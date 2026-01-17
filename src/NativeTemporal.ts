import { TurboModuleRegistry, type TurboModule } from 'react-native';

export interface Spec extends TurboModule {
  multiply(a: number, b: number): number;
  instantNow(): string;

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

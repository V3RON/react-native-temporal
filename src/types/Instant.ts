import NativeTemporal from '../NativeTemporal';
import { wrapNativeCall } from '../utils';
import { Duration, type DurationLike } from './Duration';
import { ZonedDateTime } from './ZonedDateTime';
import { Calendar } from './Calendar';
import { TimeZone } from './TimeZone';

/**
 * A Temporal.Instant represents a fixed point in time, without regard to calendar or time zone,
 * e.g. July 20, 1969, at 20:17 UTC.
 *
 * This implementation follows the TC39 Temporal proposal.
 * @see https://tc39.es/proposal-temporal/#sec-temporal-instant-objects
 */
export class Instant {
  readonly #isoString: string;

  private constructor(isoString: string) {
    this.#isoString = isoString;
  }

  /**
   * Creates an Instant from an ISO 8601 string or another Instant.
   *
   * @example
   * Instant.from("2020-01-01T00:00:00Z")
   */
  static from(item: string | Instant): Instant {
    if (item instanceof Instant) {
      return item;
    }
    if (typeof item === 'string') {
      const iso = wrapNativeCall(
        () => NativeTemporal.instantFromString(item),
        `Invalid instant string: ${item}`
      );
      return new Instant(iso);
    }
    throw new TypeError('Instant.from requires a string or Instant');
  }

  /**
   * Creates an Instant from the number of milliseconds since the Unix epoch.
   */
  static fromEpochMilliseconds(epochMilliseconds: number): Instant {
    const iso = wrapNativeCall(
      () => NativeTemporal.instantFromEpochMilliseconds(epochMilliseconds),
      'Invalid epoch milliseconds'
    );
    return new Instant(iso);
  }

  /**
   * Creates an Instant from the number of nanoseconds since the Unix epoch.
   */
  static fromEpochNanoseconds(epochNanoseconds: bigint): Instant {
    const iso = wrapNativeCall(
      () =>
        NativeTemporal.instantFromEpochNanoseconds(epochNanoseconds.toString()),
      'Invalid epoch nanoseconds'
    );
    return new Instant(iso);
  }

  /**
   * Returns the current instant.
   */
  static now(): Instant {
    const iso = wrapNativeCall(
      () => NativeTemporal.instantNow(),
      'Failed to get current instant'
    );
    return new Instant(iso);
  }

  /**
   * Compares two Instant objects.
   */
  static compare(one: Instant | string, two: Instant | string): -1 | 0 | 1 {
    const instOne = one instanceof Instant ? one : Instant.from(one);
    const instTwo = two instanceof Instant ? two : Instant.from(two);

    const result = wrapNativeCall(
      () =>
        NativeTemporal.instantCompare(instOne.#isoString, instTwo.#isoString),
      'Failed to compare instants'
    );
    return result as -1 | 0 | 1;
  }

  /**
   * Returns the number of milliseconds since the Unix epoch.
   */
  get epochMilliseconds(): number {
    return wrapNativeCall(
      () => NativeTemporal.instantEpochMilliseconds(this.#isoString),
      'Failed to get epoch milliseconds'
    );
  }

  /**
   * Returns the number of nanoseconds since the Unix epoch.
   */
  get epochNanoseconds(): bigint {
    const nsStr = wrapNativeCall(
      () => NativeTemporal.instantEpochNanoseconds(this.#isoString),
      'Failed to get epoch nanoseconds'
    );
    return BigInt(nsStr);
  }

  /**
   * Adds a duration to this instant.
   */
  add(duration: Duration | DurationLike | string): Instant {
    const durationObj =
      duration instanceof Duration ? duration : Duration.from(duration);
    const iso = wrapNativeCall(
      () => NativeTemporal.instantAdd(this.#isoString, durationObj.toString()),
      'Failed to add duration'
    );
    return new Instant(iso);
  }

  /**
   * Subtracts a duration from this instant.
   */
  subtract(duration: Duration | DurationLike | string): Instant {
    const durationObj =
      duration instanceof Duration ? duration : Duration.from(duration);
    const iso = wrapNativeCall(
      () =>
        NativeTemporal.instantSubtract(this.#isoString, durationObj.toString()),
      'Failed to subtract duration'
    );
    return new Instant(iso);
  }

  /**
   * Checks if this Instant is equal to another.
   */
  equals(other: Instant | string): boolean {
    const otherInst = other instanceof Instant ? other : Instant.from(other);
    return Instant.compare(this, otherInst) === 0;
  }

  /**
   * Computes the difference between this Instant and another.
   */
  until(
    other: Instant | string,
    options?: {
      largestUnit?: string;
      smallestUnit?: string;
      roundingIncrement?: number;
      roundingMode?: string;
    }
  ): Duration {
    const otherInst = other instanceof Instant ? other : Instant.from(other);
    const durStr = wrapNativeCall(
      () =>
        NativeTemporal.instantUntil(
          this.#isoString,
          otherInst.#isoString,
          options?.largestUnit ?? null,
          options?.smallestUnit ?? null,
          options?.roundingIncrement ?? 1,
          options?.roundingMode ?? null
        ),
      'Until failed'
    );
    return Duration.from(durStr);
  }

  /**
   * Computes the difference between another Instant and this one.
   */
  since(
    other: Instant | string,
    options?: {
      largestUnit?: string;
      smallestUnit?: string;
      roundingIncrement?: number;
      roundingMode?: string;
    }
  ): Duration {
    const otherInst = other instanceof Instant ? other : Instant.from(other);
    const durStr = wrapNativeCall(
      () =>
        NativeTemporal.instantSince(
          this.#isoString,
          otherInst.#isoString,
          options?.largestUnit ?? null,
          options?.smallestUnit ?? null,
          options?.roundingIncrement ?? 1,
          options?.roundingMode ?? null
        ),
      'Since failed'
    );
    return Duration.from(durStr);
  }

  /**
   * Rounds the Instant to the given smallest unit.
   */
  round(options: {
    smallestUnit: string;
    roundingIncrement?: number;
    roundingMode?: string;
  }): Instant {
    const iso = wrapNativeCall(
      () =>
        NativeTemporal.instantRound(
          this.#isoString,
          options.smallestUnit,
          options.roundingIncrement ?? 1,
          options.roundingMode ?? null
        ),
      'Round failed'
    );
    return new Instant(iso);
  }

  /**
   * Converts this Instant to a ZonedDateTime in the ISO 8601 calendar.
   */
  toZonedDateTimeISO(timeZone: string | TimeZone): ZonedDateTime {
    const tz = TimeZone.from(timeZone);
    const iso = wrapNativeCall(
      () =>
        NativeTemporal.instantToZonedDateTime(
          this.#isoString,
          null, // Default to ISO8601 implied by null calendar logic in native or we can pass 'iso8601'
          tz.id
        ),
      'To ZonedDateTime ISO failed'
    );
    return ZonedDateTime.from(iso);
  }

  /**
   * Converts this Instant to a ZonedDateTime with the specified calendar and time zone.
   */
  toZonedDateTime(options: {
    timeZone: string | TimeZone;
    calendar: string | Calendar;
  }): ZonedDateTime {
    const tz = TimeZone.from(options.timeZone);
    const cal = Calendar.from(options.calendar);
    const iso = wrapNativeCall(
      () =>
        NativeTemporal.instantToZonedDateTime(this.#isoString, cal.id, tz.id),
      'To ZonedDateTime failed'
    );
    return ZonedDateTime.from(iso);
  }

  /**
   * Returns the ISO 8601 string representation.
   */
  toString(options?: { timeZone?: string | TimeZone }): string {
    if (options?.timeZone) {
      return this.toZonedDateTimeISO(options.timeZone).toString();
    }
    return this.#isoString;
  }

  toJSON(): string {
    return this.toString();
  }

  valueOf(): never {
    throw new TypeError('Cannot convert a Temporal.Instant to a primitive');
  }

  toLocaleString(
    locales?: string | string[],
    options?: Intl.DateTimeFormatOptions
  ): string {
    // Fallback to Date for formatting for now
    return new Date(this.epochMilliseconds).toLocaleString(locales, options);
  }
}

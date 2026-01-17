import NativeTemporal from '../NativeTemporal';

/**
 * Sentinel value for "unchanged" in durationWith.
 * Uses a value that maps to i64::MIN in native code.
 */
const UNCHANGED = Number.MIN_SAFE_INTEGER;

/**
 * Component indices in the array returned by durationGetAllComponents.
 */
const enum ComponentIndex {
  Years = 0,
  Months = 1,
  Weeks = 2,
  Days = 3,
  Hours = 4,
  Minutes = 5,
  Seconds = 6,
  Milliseconds = 7,
  Microseconds = 8,
  Nanoseconds = 9,
  Sign = 10,
  Blank = 11,
}

export type DurationLike = {
  years?: number;
  months?: number;
  weeks?: number;
  days?: number;
  hours?: number;
  minutes?: number;
  seconds?: number;
  milliseconds?: number;
  microseconds?: number;
  nanoseconds?: number;
};

/**
 * Wraps a native call and ensures proper TC39 Temporal error types are thrown.
 * Native exceptions come through as generic Errors, so we re-throw
 * them as RangeError or TypeError to match the Temporal specification.
 */
const wrapNativeCall = <T>(fn: () => T, context: string): T => {
  try {
    return fn();
  } catch (error) {
    if (error instanceof Error) {
      let message = error.message || '';

      // Strip React Native's "Exception in HostFunction: " prefix
      message = message.replace(/^Exception in HostFunction:\s*/i, '');

      // Check for error type markers
      const isTypeError =
        error.name === 'TypeError' ||
        message.startsWith('[TypeError]') ||
        message.toLowerCase().includes('cannot be null') ||
        message.toLowerCase().includes('type error');

      // Clean up any [ErrorType] prefix from the message
      message = message.replace(/^\[(RangeError|TypeError)\]\s*/i, '');

      if (isTypeError) {
        throw new TypeError(message || context);
      }
      throw new RangeError(message || context);
    }
    throw new RangeError(context);
  }
};

/**
 * A Temporal.Duration represents a duration of time, such as "2 hours and 30 minutes"
 * or "3 years, 2 months". Unlike Instant which represents a specific moment in time,
 * Duration represents the amount of time between two moments.
 *
 * This implementation follows the TC39 Temporal proposal.
 * @see https://tc39.es/proposal-temporal/#sec-temporal-duration-objects
 */
export class Duration {
  /** Internal ISO 8601 string representation */
  readonly #isoString: string;

  /** Cached components array from native */
  readonly #components: number[];

  private constructor(isoString: string, components: number[]) {
    this.#isoString = isoString;
    this.#components = components;
  }

  /**
   * Creates a Duration from an ISO 8601 duration string or a DurationLike object.
   *
   * @example
   * Duration.from("P1Y2M3DT4H5M6S")
   * Duration.from({ hours: 2, minutes: 30 })
   *
   * @throws {RangeError} If the string is not a valid ISO 8601 duration or values are invalid
   * @throws {TypeError} If the argument is not a string or object
   */
  static from(item: string | DurationLike | Duration): Duration {
    if (item instanceof Duration) {
      return item;
    }

    if (typeof item === 'string') {
      const isoString = wrapNativeCall(
        () => NativeTemporal.durationFromString(item),
        `Invalid duration string: ${item}`
      );
      const components = wrapNativeCall(
        () => NativeTemporal.durationGetAllComponents(isoString),
        'Failed to get duration components'
      );
      return new Duration(isoString, components);
    }

    if (typeof item !== 'object' || item === null) {
      throw new TypeError(
        'Duration.from requires a string, object, or Duration'
      );
    }

    // Create duration from components - all validation happens in native
    const isoString = wrapNativeCall(
      () =>
        NativeTemporal.durationFromComponents(
          item.years ?? 0,
          item.months ?? 0,
          item.weeks ?? 0,
          item.days ?? 0,
          item.hours ?? 0,
          item.minutes ?? 0,
          item.seconds ?? 0,
          item.milliseconds ?? 0,
          item.microseconds ?? 0,
          item.nanoseconds ?? 0
        ),
      'Invalid duration values'
    );
    const components = wrapNativeCall(
      () => NativeTemporal.durationGetAllComponents(isoString),
      'Failed to get duration components'
    );
    return new Duration(isoString, components);
  }

  /**
   * Compares two Duration values.
   *
   * @returns -1 if one < two, 0 if equal, 1 if one > two
   * @throws {RangeError} If durations contain years, months, or weeks (requires relativeTo)
   *
   * @example
   * Duration.compare(Duration.from("PT1H"), Duration.from("PT30M")) // 1
   */
  static compare(
    one: Duration | DurationLike | string,
    two: Duration | DurationLike | string
  ): -1 | 0 | 1 {
    const durationOne = one instanceof Duration ? one : Duration.from(one);
    const durationTwo = two instanceof Duration ? two : Duration.from(two);

    const result = wrapNativeCall(
      () =>
        NativeTemporal.durationCompare(
          durationOne.#isoString,
          durationTwo.#isoString
        ),
      'Failed to compare durations'
    );

    return result as -1 | 0 | 1;
  }

  /**
   * Returns the ISO 8601 string representation of this duration.
   */
  toString(): string {
    return this.#isoString;
  }

  /**
   * Returns a primitive value for comparison operations.
   * @throws {TypeError} Always throws - Duration cannot be converted to a primitive value.
   */
  valueOf(): never {
    throw new TypeError(
      'Cannot convert a Temporal.Duration to a primitive. Use Duration.compare() for comparison.'
    );
  }

  /**
   * Converts this Duration to a JSON representation.
   */
  toJSON(): string {
    return this.#isoString;
  }

  /**
   * Returns the string tag for this object.
   */
  get [Symbol.toStringTag](): string {
    return 'Temporal.Duration';
  }

  // Component getters - read from cached components array

  get years(): number {
    return this.#components[ComponentIndex.Years]!;
  }

  get months(): number {
    return this.#components[ComponentIndex.Months]!;
  }

  get weeks(): number {
    return this.#components[ComponentIndex.Weeks]!;
  }

  get days(): number {
    return this.#components[ComponentIndex.Days]!;
  }

  get hours(): number {
    return this.#components[ComponentIndex.Hours]!;
  }

  get minutes(): number {
    return this.#components[ComponentIndex.Minutes]!;
  }

  get seconds(): number {
    return this.#components[ComponentIndex.Seconds]!;
  }

  get milliseconds(): number {
    return this.#components[ComponentIndex.Milliseconds]!;
  }

  get microseconds(): number {
    return this.#components[ComponentIndex.Microseconds]!;
  }

  get nanoseconds(): number {
    return this.#components[ComponentIndex.Nanoseconds]!;
  }

  /**
   * Returns the sign of the duration: -1 for negative, 0 for zero, 1 for positive.
   */
  get sign(): -1 | 0 | 1 {
    return this.#components[ComponentIndex.Sign] as -1 | 0 | 1;
  }

  /**
   * Returns true if this duration is zero (all components are zero).
   * This is the TC39 Temporal property name.
   */
  get blank(): boolean {
    return this.#components[ComponentIndex.Blank] === 1;
  }

  /**
   * Adds another duration to this one and returns the result.
   *
   * @example
   * Duration.from("PT1H").add({ minutes: 30 }) // PT1H30M
   */
  add(other: Duration | DurationLike | string): Duration {
    const otherDuration =
      other instanceof Duration ? other : Duration.from(other);
    const isoString = wrapNativeCall(
      () =>
        NativeTemporal.durationAdd(this.#isoString, otherDuration.#isoString),
      'Failed to add durations'
    );
    const components = wrapNativeCall(
      () => NativeTemporal.durationGetAllComponents(isoString),
      'Failed to get duration components'
    );
    return new Duration(isoString, components);
  }

  /**
   * Subtracts another duration from this one and returns the result.
   *
   * @example
   * Duration.from("PT2H").subtract({ minutes: 30 }) // PT1H30M
   */
  subtract(other: Duration | DurationLike | string): Duration {
    const otherDuration =
      other instanceof Duration ? other : Duration.from(other);
    const isoString = wrapNativeCall(
      () =>
        NativeTemporal.durationSubtract(
          this.#isoString,
          otherDuration.#isoString
        ),
      'Failed to subtract durations'
    );
    const components = wrapNativeCall(
      () => NativeTemporal.durationGetAllComponents(isoString),
      'Failed to get duration components'
    );
    return new Duration(isoString, components);
  }

  /**
   * Returns a new Duration with the opposite sign.
   *
   * @example
   * Duration.from("PT1H").negated() // -PT1H
   */
  negated(): Duration {
    const isoString = wrapNativeCall(
      () => NativeTemporal.durationNegated(this.#isoString),
      'Failed to negate duration'
    );
    const components = wrapNativeCall(
      () => NativeTemporal.durationGetAllComponents(isoString),
      'Failed to get duration components'
    );
    return new Duration(isoString, components);
  }

  /**
   * Returns a new Duration with all positive components (absolute value).
   *
   * @example
   * Duration.from("-PT1H").abs() // PT1H
   */
  abs(): Duration {
    const isoString = wrapNativeCall(
      () => NativeTemporal.durationAbs(this.#isoString),
      'Failed to get absolute duration'
    );
    const components = wrapNativeCall(
      () => NativeTemporal.durationGetAllComponents(isoString),
      'Failed to get duration components'
    );
    return new Duration(isoString, components);
  }

  /**
   * Returns a new Duration with some components replaced.
   *
   * @example
   * Duration.from("P1Y2M").with({ months: 6 }) // P1Y6M
   *
   * @throws {RangeError} If new values result in mixed signs
   */
  with(durationLike: DurationLike): Duration {
    const isoString = wrapNativeCall(
      () =>
        NativeTemporal.durationWith(
          this.#isoString,
          durationLike.years ?? UNCHANGED,
          durationLike.months ?? UNCHANGED,
          durationLike.weeks ?? UNCHANGED,
          durationLike.days ?? UNCHANGED,
          durationLike.hours ?? UNCHANGED,
          durationLike.minutes ?? UNCHANGED,
          durationLike.seconds ?? UNCHANGED,
          durationLike.milliseconds ?? UNCHANGED,
          durationLike.microseconds ?? UNCHANGED,
          durationLike.nanoseconds ?? UNCHANGED
        ),
      'Failed to modify duration'
    );
    const components = wrapNativeCall(
      () => NativeTemporal.durationGetAllComponents(isoString),
      'Failed to get duration components'
    );
    return new Duration(isoString, components);
  }

  /**
   * Returns a locale-sensitive string representation of this duration.
   *
   * Note: This is a simplified implementation. Full Intl.DurationFormat
   * support is not yet available in all environments.
   */
  toLocaleString(
    _locales?: string | string[],
    _options?: Intl.DateTimeFormatOptions
  ): string {
    // TODO: Use Intl.DurationFormat when available
    // For now, return a simple human-readable format
    const parts: string[] = [];

    if (this.years !== 0)
      parts.push(`${this.years} year${this.years !== 1 ? 's' : ''}`);
    if (this.months !== 0)
      parts.push(`${this.months} month${this.months !== 1 ? 's' : ''}`);
    if (this.weeks !== 0)
      parts.push(`${this.weeks} week${this.weeks !== 1 ? 's' : ''}`);
    if (this.days !== 0)
      parts.push(`${this.days} day${this.days !== 1 ? 's' : ''}`);
    if (this.hours !== 0)
      parts.push(`${this.hours} hour${this.hours !== 1 ? 's' : ''}`);
    if (this.minutes !== 0)
      parts.push(`${this.minutes} minute${this.minutes !== 1 ? 's' : ''}`);
    if (this.seconds !== 0)
      parts.push(`${this.seconds} second${this.seconds !== 1 ? 's' : ''}`);
    if (this.milliseconds !== 0)
      parts.push(
        `${this.milliseconds} millisecond${this.milliseconds !== 1 ? 's' : ''}`
      );
    if (this.microseconds !== 0)
      parts.push(
        `${this.microseconds} microsecond${this.microseconds !== 1 ? 's' : ''}`
      );
    if (this.nanoseconds !== 0)
      parts.push(
        `${this.nanoseconds} nanosecond${this.nanoseconds !== 1 ? 's' : ''}`
      );

    return parts.length > 0 ? parts.join(', ') : '0 seconds';
  }
}

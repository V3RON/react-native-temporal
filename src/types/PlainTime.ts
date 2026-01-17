import NativeTemporal from '../NativeTemporal';
import { wrapNativeCall } from '../utils';
import { Duration, type DurationLike } from './Duration';

/**
 * Component indices in the array returned by plainTimeGetAllComponents.
 */
const enum ComponentIndex {
  Hour = 0,
  Minute = 1,
  Second = 2,
  Millisecond = 3,
  Microsecond = 4,
  Nanosecond = 5,
}

export type PlainTimeLike = {
  hour?: number;
  minute?: number;
  second?: number;
  millisecond?: number;
  microsecond?: number;
  nanosecond?: number;
};

/**
 * A Temporal.PlainTime represents a wall-clock time, with a precision in nanoseconds,
 * and without any time zone. "Wall-clock time" means that it is not a specific instant
 * in time, but rather a time that appears on a clock.
 *
 * This implementation follows the TC39 Temporal proposal.
 * @see https://tc39.es/proposal-temporal/#sec-temporal-plaintime-objects
 */
export class PlainTime {
  readonly #isoString: string;
  readonly #components: number[];

  private constructor(isoString: string, components: number[]) {
    this.#isoString = isoString;
    this.#components = components;
  }

  /**
   * Creates a PlainTime from an ISO 8601 string or a PlainTimeLike object.
   *
   * @example
   * PlainTime.from("12:30:00")
   * PlainTime.from({ hour: 12, minute: 30 })
   */
  static from(item: string | PlainTimeLike | PlainTime): PlainTime {
    if (item instanceof PlainTime) {
      return item;
    }

    if (typeof item === 'string') {
      const isoString = wrapNativeCall(
        () => NativeTemporal.plainTimeFromString(item),
        `Invalid plain time string: ${item}`
      );
      const components = wrapNativeCall(
        () => NativeTemporal.plainTimeGetAllComponents(isoString),
        'Failed to get plain time components'
      );
      return new PlainTime(isoString, components);
    }

    if (typeof item === 'object' && item !== null) {
      const isoString = wrapNativeCall(
        () =>
          NativeTemporal.plainTimeFromComponents(
            item.hour ?? 0,
            item.minute ?? 0,
            item.second ?? 0,
            item.millisecond ?? 0,
            item.microsecond ?? 0,
            item.nanosecond ?? 0
          ),
        'Invalid plain time components'
      );
      const components = wrapNativeCall(
        () => NativeTemporal.plainTimeGetAllComponents(isoString),
        'Failed to get plain time components'
      );
      return new PlainTime(isoString, components);
    }

    throw new TypeError(
      'PlainTime.from requires a string, object, or PlainTime'
    );
  }

  /**
   * Compares two PlainTime objects.
   */
  static compare(
    one: PlainTime | string | PlainTimeLike,
    two: PlainTime | string | PlainTimeLike
  ): -1 | 0 | 1 {
    const t1 = one instanceof PlainTime ? one : PlainTime.from(one);
    const t2 = two instanceof PlainTime ? two : PlainTime.from(two);

    const result = wrapNativeCall(
      () => NativeTemporal.plainTimeCompare(t1.#isoString, t2.#isoString),
      'Failed to compare plain times'
    );
    return result as -1 | 0 | 1;
  }

  get hour(): number {
    return this.#components[ComponentIndex.Hour]!;
  }

  get minute(): number {
    return this.#components[ComponentIndex.Minute]!;
  }

  get second(): number {
    return this.#components[ComponentIndex.Second]!;
  }

  get millisecond(): number {
    return this.#components[ComponentIndex.Millisecond]!;
  }

  get microsecond(): number {
    return this.#components[ComponentIndex.Microsecond]!;
  }

  get nanosecond(): number {
    return this.#components[ComponentIndex.Nanosecond]!;
  }

  /**
   * Adds a duration to this time.
   */
  add(duration: Duration | DurationLike | string): PlainTime {
    const d = duration instanceof Duration ? duration : Duration.from(duration);
    const isoString = wrapNativeCall(
      () => NativeTemporal.plainTimeAdd(this.#isoString, d.toString()),
      'Failed to add duration'
    );
    const components = wrapNativeCall(
      () => NativeTemporal.plainTimeGetAllComponents(isoString),
      'Failed to get plain time components'
    );
    return new PlainTime(isoString, components);
  }

  /**
   * Subtracts a duration from this time.
   */
  subtract(duration: Duration | DurationLike | string): PlainTime {
    const d = duration instanceof Duration ? duration : Duration.from(duration);
    const isoString = wrapNativeCall(
      () => NativeTemporal.plainTimeSubtract(this.#isoString, d.toString()),
      'Failed to subtract duration'
    );
    const components = wrapNativeCall(
      () => NativeTemporal.plainTimeGetAllComponents(isoString),
      'Failed to get plain time components'
    );
    return new PlainTime(isoString, components);
  }

  /**
   * Returns a new PlainTime with some fields replaced.
   */
  with(timeLike: PlainTimeLike): PlainTime {
    return PlainTime.from({
      hour: timeLike.hour ?? this.hour,
      minute: timeLike.minute ?? this.minute,
      second: timeLike.second ?? this.second,
      millisecond: timeLike.millisecond ?? this.millisecond,
      microsecond: timeLike.microsecond ?? this.microsecond,
      nanosecond: timeLike.nanosecond ?? this.nanosecond,
    });
  }

  equals(other: PlainTime | string | PlainTimeLike): boolean {
    return PlainTime.compare(this, other) === 0;
  }

  toString(_options?: object): string {
    return this.#isoString;
  }

  toJSON(): string {
    return this.toString();
  }

  valueOf(): never {
    throw new TypeError('Cannot convert a Temporal.PlainTime to a primitive');
  }

  toLocaleString(
    locales?: string | string[],
    options?: Intl.DateTimeFormatOptions
  ): string {
    // Basic fallback using Date (assumes dummy date)
    const date = new Date();
    date.setHours(this.hour, this.minute, this.second, this.millisecond);
    return date.toLocaleTimeString(locales, options);
  }

  /**
   * Computes the difference between this time and another.
   */
  until(
    _other: PlainTime | string | PlainTimeLike,
    _options?: object
  ): Duration {
    // TODO: Implement via FFI
    throw new Error('PlainTime.until is not yet implemented');
  }

  /**
   * Computes the difference between another time and this one.
   */
  since(
    _other: PlainTime | string | PlainTimeLike,
    _options?: object
  ): Duration {
    // TODO: Implement via FFI
    throw new Error('PlainTime.since is not yet implemented');
  }

  /**
   * Rounds the time to the given smallest unit.
   */
  round(_options: object): PlainTime {
    // TODO: Implement via FFI
    throw new Error('PlainTime.round is not yet implemented');
  }
}

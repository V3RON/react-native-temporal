import NativeTemporal from '../NativeTemporal';
import { wrapNativeCall } from '../utils';
import { Duration, type DurationLike } from './Duration';

export type PlainDateLike = {
  year?: number;
  month?: number;
  day?: number;
  calendar?: string;
  monthCode?: string;
};

const enum ComponentIndex {
  Year = 0,
  Month = 1,
  Day = 2,
  DayOfWeek = 3,
  DayOfYear = 4,
  WeekOfYear = 5,
  YearOfWeek = 6,
  DaysInWeek = 7,
  DaysInMonth = 8,
  DaysInYear = 9,
  MonthsInYear = 10,
  InLeapYear = 11,
}

export class PlainDate {
  readonly #isoString: string;
  readonly #components: number[];
  #monthCode: string | undefined;
  #calendarId: string | undefined;

  private constructor(isoString: string, components: number[]) {
    this.#isoString = isoString;
    this.#components = components;
  }

  static from(item: string | PlainDateLike | PlainDate): PlainDate {
    if (item instanceof PlainDate) return item;

    if (typeof item === 'string') {
      const isoString = wrapNativeCall(
        () => NativeTemporal.plainDateFromString(item),
        `Invalid plain date string: ${item}`
      );
      if (!isoString) {
        throw new RangeError(`Invalid plain date string: ${item}`);
      }
      const components = wrapNativeCall(
        () => NativeTemporal.plainDateGetAllComponents(isoString),
        'Failed to get plain date components'
      );
      return new PlainDate(isoString, components);
    }

    if (typeof item === 'object' && item !== null) {
      const isoString = wrapNativeCall(
        () =>
          NativeTemporal.plainDateFromComponents(
            item.year ?? 0,
            item.month ?? 0,
            item.day ?? 0,
            item.calendar ?? null
          ),
        'Invalid plain date components'
      );
      if (!isoString) {
        throw new RangeError('Invalid plain date components');
      }
      const components = wrapNativeCall(
        () => NativeTemporal.plainDateGetAllComponents(isoString),
        'Failed to get plain date components'
      );
      return new PlainDate(isoString, components);
    }

    throw new TypeError(
      'PlainDate.from requires a string, object, or PlainDate'
    );
  }

  static compare(
    one: PlainDate | string | PlainDateLike,
    two: PlainDate | string | PlainDateLike
  ): number {
    const d1 = one instanceof PlainDate ? one : PlainDate.from(one);
    const d2 = two instanceof PlainDate ? two : PlainDate.from(two);
    return wrapNativeCall(
      () => NativeTemporal.plainDateCompare(d1.#isoString, d2.#isoString),
      'Failed to compare plain dates'
    );
  }

  get year(): number {
    return this.#components[ComponentIndex.Year]!;
  }
  get month(): number {
    return this.#components[ComponentIndex.Month]!;
  }
  get day(): number {
    return this.#components[ComponentIndex.Day]!;
  }

  get monthCode(): string {
    if (this.#monthCode === undefined) {
      this.#monthCode = NativeTemporal.plainDateGetMonthCode(this.#isoString);
    }
    return this.#monthCode!;
  }

  get calendarId(): string {
    if (this.#calendarId === undefined) {
      this.#calendarId = NativeTemporal.plainDateGetCalendar(this.#isoString);
    }
    return this.#calendarId!;
  }

  get dayOfWeek(): number {
    return this.#components[ComponentIndex.DayOfWeek]!;
  }
  get dayOfYear(): number {
    return this.#components[ComponentIndex.DayOfYear]!;
  }
  get weekOfYear(): number {
    return this.#components[ComponentIndex.WeekOfYear]!;
  }
  get yearOfWeek(): number {
    return this.#components[ComponentIndex.YearOfWeek]!;
  }
  get daysInWeek(): number {
    return this.#components[ComponentIndex.DaysInWeek]!;
  }
  get daysInMonth(): number {
    return this.#components[ComponentIndex.DaysInMonth]!;
  }
  get daysInYear(): number {
    return this.#components[ComponentIndex.DaysInYear]!;
  }
  get monthsInYear(): number {
    return this.#components[ComponentIndex.MonthsInYear]!;
  }
  get inLeapYear(): boolean {
    return this.#components[ComponentIndex.InLeapYear]! === 1;
  }

  add(duration: Duration | DurationLike | string): PlainDate {
    const d = duration instanceof Duration ? duration : Duration.from(duration);
    const isoString = wrapNativeCall(
      () => NativeTemporal.plainDateAdd(this.#isoString, d.toString()),
      'Failed to add duration'
    );
    const components = wrapNativeCall(
      () => NativeTemporal.plainDateGetAllComponents(isoString),
      'Failed to get components'
    );
    return new PlainDate(isoString, components);
  }

  subtract(duration: Duration | DurationLike | string): PlainDate {
    const d = duration instanceof Duration ? duration : Duration.from(duration);
    const isoString = wrapNativeCall(
      () => NativeTemporal.plainDateSubtract(this.#isoString, d.toString()),
      'Failed to subtract duration'
    );
    const components = wrapNativeCall(
      () => NativeTemporal.plainDateGetAllComponents(isoString),
      'Failed to get components'
    );
    return new PlainDate(isoString, components);
  }

  with(dateLike: PlainDateLike): PlainDate {
    const isoString = wrapNativeCall(
      () =>
        NativeTemporal.plainDateWith(
          this.#isoString,
          dateLike.year ?? Number.MIN_SAFE_INTEGER,
          dateLike.month ?? Number.MIN_SAFE_INTEGER,
          dateLike.day ?? Number.MIN_SAFE_INTEGER,
          dateLike.calendar ?? null
        ),
      'Failed to update plain date'
    );
    const components = wrapNativeCall(
      () => NativeTemporal.plainDateGetAllComponents(isoString),
      'Failed to get components'
    );
    return new PlainDate(isoString, components);
  }

  until(other: PlainDate | string | PlainDateLike): Duration {
    const o = other instanceof PlainDate ? other : PlainDate.from(other);
    const durationIso = wrapNativeCall(
      () => NativeTemporal.plainDateUntil(this.#isoString, o.#isoString),
      'Failed to compute until'
    );
    return Duration.from(durationIso);
  }

  since(other: PlainDate | string | PlainDateLike): Duration {
    const o = other instanceof PlainDate ? other : PlainDate.from(other);
    const durationIso = wrapNativeCall(
      () => NativeTemporal.plainDateSince(this.#isoString, o.#isoString),
      'Failed to compute since'
    );
    return Duration.from(durationIso);
  }

  equals(other: PlainDate | string | PlainDateLike): boolean {
    return PlainDate.compare(this, other) === 0;
  }

  toString(_options?: object): string {
    return this.#isoString;
  }

  toJSON(): string {
    return this.toString();
  }

  toLocaleString(
    locales?: string | string[],
    options?: Intl.DateTimeFormatOptions
  ): string {
    const date = new Date(this.year, this.month - 1, this.day);
    return date.toLocaleDateString(locales, options);
  }

  valueOf(): never {
    throw new TypeError('Cannot convert a Temporal.PlainDate to a primitive');
  }
}

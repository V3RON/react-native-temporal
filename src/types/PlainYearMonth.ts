import NativeTemporal from '../NativeTemporal';
import { wrapNativeCall } from '../utils';
import { Duration, type DurationLike } from './Duration';
import { PlainDate } from './PlainDate';

export type PlainYearMonthLike = {
  year?: number;
  month?: number;
  monthCode?: string;
  calendar?: string;
};

const enum ComponentIndex {
  Year = 0,
  Month = 1,
  Day = 2,
  DaysInMonth = 3,
  DaysInYear = 4,
  MonthsInYear = 5,
  InLeapYear = 6,
  EraYear = 7,
}

export class PlainYearMonth {
  readonly #isoString: string;
  readonly #components: number[];
  #monthCode: string | undefined;
  #calendarId: string | undefined;

  private constructor(isoString: string, components: number[]) {
    this.#isoString = isoString;
    this.#components = components;
  }

  static from(
    item: string | PlainYearMonthLike | PlainYearMonth
  ): PlainYearMonth {
    if (item instanceof PlainYearMonth) return item;

    if (typeof item === 'string') {
      const isoString = wrapNativeCall(
        () => NativeTemporal.plainYearMonthFromString(item),
        `Invalid plain year month string: ${item}`
      );
      if (!isoString) {
        throw new RangeError(`Invalid plain year month string: ${item}`);
      }
      const components = wrapNativeCall(
        () => NativeTemporal.plainYearMonthGetAllComponents(isoString),
        'Failed to get plain year month components'
      );
      return new PlainYearMonth(isoString, components);
    }

    if (typeof item === 'object' && item !== null) {
      const isoString = wrapNativeCall(
        () =>
          NativeTemporal.plainYearMonthFromComponents(
            item.year ?? 0,
            item.month ?? 0,
            item.calendar ?? null,
            0 // referenceDay (default)
          ),
        'Invalid plain year month components'
      );
      if (!isoString) {
        throw new RangeError('Invalid plain year month components');
      }
      const components = wrapNativeCall(
        () => NativeTemporal.plainYearMonthGetAllComponents(isoString),
        'Failed to get plain year month components'
      );
      return new PlainYearMonth(isoString, components);
    }

    throw new TypeError(
      'PlainYearMonth.from requires a string, object, or PlainYearMonth'
    );
  }

  static compare(
    one: PlainYearMonth | string | PlainYearMonthLike,
    two: PlainYearMonth | string | PlainYearMonthLike
  ): number {
    const ym1 = one instanceof PlainYearMonth ? one : PlainYearMonth.from(one);
    const ym2 = two instanceof PlainYearMonth ? two : PlainYearMonth.from(two);
    return wrapNativeCall(
      () =>
        NativeTemporal.plainYearMonthCompare(ym1.#isoString, ym2.#isoString),
      'Failed to compare plain year months'
    );
  }

  get year(): number {
    return this.#components[ComponentIndex.Year]!;
  }
  get month(): number {
    return this.#components[ComponentIndex.Month]!;
  }
  get monthCode(): string {
    if (this.#monthCode === undefined) {
      this.#monthCode = NativeTemporal.plainYearMonthGetMonthCode(
        this.#isoString
      );
    }
    return this.#monthCode!;
  }
  get calendarId(): string {
    if (this.#calendarId === undefined) {
      this.#calendarId = NativeTemporal.plainYearMonthGetCalendar(
        this.#isoString
      );
    }
    return this.#calendarId!;
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
    return this.#components[ComponentIndex.InLeapYear] === 1;
  }
  get eraYear(): number | undefined {
    // Only return eraYear if it's relevant (non-ISO usually, but implementation details vary)
    // For now we just return what we got if calendar uses it.
    // The FFI returns 0 if not present/applicable, but we might want to refine this logic
    // based on calendar. For now, basic return.
    const val = this.#components[ComponentIndex.EraYear];
    return val === 0 ? undefined : val;
  }

  add(duration: Duration | DurationLike | string): PlainYearMonth {
    const d = duration instanceof Duration ? duration : Duration.from(duration);
    const isoString = wrapNativeCall(
      () => NativeTemporal.plainYearMonthAdd(this.#isoString, d.toString()),
      'Failed to add duration'
    );
    const components = wrapNativeCall(
      () => NativeTemporal.plainYearMonthGetAllComponents(isoString),
      'Failed to get components'
    );
    return new PlainYearMonth(isoString, components);
  }

  subtract(duration: Duration | DurationLike | string): PlainYearMonth {
    const d = duration instanceof Duration ? duration : Duration.from(duration);
    const isoString = wrapNativeCall(
      () =>
        NativeTemporal.plainYearMonthSubtract(this.#isoString, d.toString()),
      'Failed to subtract duration'
    );
    const components = wrapNativeCall(
      () => NativeTemporal.plainYearMonthGetAllComponents(isoString),
      'Failed to get components'
    );
    return new PlainYearMonth(isoString, components);
  }

  with(like: PlainYearMonthLike): PlainYearMonth {
    const isoString = wrapNativeCall(
      () =>
        NativeTemporal.plainYearMonthWith(
          this.#isoString,
          like.year ?? Number.MIN_SAFE_INTEGER,
          like.month ?? Number.MIN_SAFE_INTEGER,
          like.calendar ?? null
        ),
      'Failed to update plain year month'
    );
    const components = wrapNativeCall(
      () => NativeTemporal.plainYearMonthGetAllComponents(isoString),
      'Failed to get components'
    );
    return new PlainYearMonth(isoString, components);
  }

  until(other: PlainYearMonth | string | PlainYearMonthLike): Duration {
    const o =
      other instanceof PlainYearMonth ? other : PlainYearMonth.from(other);
    const durationIso = wrapNativeCall(
      () => NativeTemporal.plainYearMonthUntil(this.#isoString, o.#isoString),
      'Failed to compute until'
    );
    return Duration.from(durationIso);
  }

  since(other: PlainYearMonth | string | PlainYearMonthLike): Duration {
    const o =
      other instanceof PlainYearMonth ? other : PlainYearMonth.from(other);
    const durationIso = wrapNativeCall(
      () => NativeTemporal.plainYearMonthSince(this.#isoString, o.#isoString),
      'Failed to compute since'
    );
    return Duration.from(durationIso);
  }

  equals(other: PlainYearMonth | string | PlainYearMonthLike): boolean {
    return PlainYearMonth.compare(this, other) === 0;
  }

  toPlainDate(item: { day: number }): PlainDate {
    if (
      typeof item !== 'object' ||
      item === null ||
      typeof item.day !== 'number'
    ) {
      throw new TypeError(
        'toPlainDate requires an object with a "day" property'
      );
    }
    const isoString = wrapNativeCall(
      () => NativeTemporal.plainYearMonthToPlainDate(this.#isoString, item.day),
      'Failed to convert to plain date'
    );
    return PlainDate.from(isoString);
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
    // Fallback using Date
    const date = new Date(this.year, this.month - 1, 1);
    return date.toLocaleString(locales, options);
  }

  valueOf(): never {
    throw new TypeError(
      'Cannot convert a Temporal.PlainYearMonth to a primitive'
    );
  }
}

import NativeTemporal from '../NativeTemporal';
import { wrapNativeCall } from '../utils';
import { Duration, type DurationLike } from './Duration';
import { PlainDate, type PlainDateLike } from './PlainDate';
import { PlainTime, type PlainTimeLike } from './PlainTime';

export type PlainDateTimeLike = {
  year?: number;
  month?: number;
  day?: number;
  hour?: number;
  minute?: number;
  second?: number;
  millisecond?: number;
  microsecond?: number;
  nanosecond?: number;
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
  Hour = 12,
  Minute = 13,
  Second = 14,
  Millisecond = 15,
  Microsecond = 16,
  Nanosecond = 17,
}

export class PlainDateTime {
  readonly #isoString: string;
  readonly #components: number[];
  #monthCode: string | undefined;
  #calendarId: string | undefined;

  private constructor(isoString: string, components: number[]) {
    this.#isoString = isoString;
    this.#components = components;
  }

  static from(item: string | PlainDateTimeLike | PlainDateTime): PlainDateTime {
    if (item instanceof PlainDateTime) return item;

    if (typeof item === 'string') {
      const isoString = wrapNativeCall(
        () => NativeTemporal.plainDateTimeFromString(item),
        `Invalid plain date time string: ${item}`
      );
      if (!isoString) {
        throw new RangeError(`Invalid plain date time string: ${item}`);
      }
      const components = wrapNativeCall(
        () => NativeTemporal.plainDateTimeGetAllComponents(isoString),
        'Failed to get plain date time components'
      );
      return new PlainDateTime(isoString, components);
    }

    if (typeof item === 'object' && item !== null) {
      const isoString = wrapNativeCall(
        () =>
          NativeTemporal.plainDateTimeFromComponents(
            item.year ?? 0,
            item.month ?? 0,
            item.day ?? 0,
            item.hour ?? 0,
            item.minute ?? 0,
            item.second ?? 0,
            item.millisecond ?? 0,
            item.microsecond ?? 0,
            item.nanosecond ?? 0,
            item.calendar ?? null
          ),
        'Invalid plain date time components'
      );
      if (!isoString) {
        throw new RangeError('Invalid plain date time components');
      }
      const components = wrapNativeCall(
        () => NativeTemporal.plainDateTimeGetAllComponents(isoString),
        'Failed to get plain date time components'
      );
      return new PlainDateTime(isoString, components);
    }

    throw new TypeError(
      'PlainDateTime.from requires a string, object, or PlainDateTime'
    );
  }

  static compare(
    one: PlainDateTime | string | PlainDateTimeLike,
    two: PlainDateTime | string | PlainDateTimeLike
  ): number {
    const dt1 = one instanceof PlainDateTime ? one : PlainDateTime.from(one);
    const dt2 = two instanceof PlainDateTime ? two : PlainDateTime.from(two);
    return wrapNativeCall(
      () => NativeTemporal.plainDateTimeCompare(dt1.#isoString, dt2.#isoString),
      'Failed to compare plain date times'
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

  get monthCode(): string {
    if (this.#monthCode === undefined) {
      this.#monthCode = NativeTemporal.plainDateTimeGetMonthCode(
        this.#isoString
      );
    }
    return this.#monthCode!;
  }

  get calendarId(): string {
    if (this.#calendarId === undefined) {
      this.#calendarId = NativeTemporal.plainDateTimeGetCalendar(
        this.#isoString
      );
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

  add(duration: Duration | DurationLike | string): PlainDateTime {
    const d = duration instanceof Duration ? duration : Duration.from(duration);
    const isoString = wrapNativeCall(
      () => NativeTemporal.plainDateTimeAdd(this.#isoString, d.toString()),
      'Failed to add duration'
    );
    const components = wrapNativeCall(
      () => NativeTemporal.plainDateTimeGetAllComponents(isoString),
      'Failed to get components'
    );
    return new PlainDateTime(isoString, components);
  }

  subtract(duration: Duration | DurationLike | string): PlainDateTime {
    const d = duration instanceof Duration ? duration : Duration.from(duration);
    const isoString = wrapNativeCall(
      () => NativeTemporal.plainDateTimeSubtract(this.#isoString, d.toString()),
      'Failed to subtract duration'
    );
    const components = wrapNativeCall(
      () => NativeTemporal.plainDateTimeGetAllComponents(isoString),
      'Failed to get components'
    );
    return new PlainDateTime(isoString, components);
  }

  with(like: PlainDateTimeLike): PlainDateTime {
    const isoString = wrapNativeCall(
      () =>
        NativeTemporal.plainDateTimeWith(
          this.#isoString,
          like.year ?? Number.MIN_SAFE_INTEGER,
          like.month ?? Number.MIN_SAFE_INTEGER,
          like.day ?? Number.MIN_SAFE_INTEGER,
          like.hour ?? Number.MIN_SAFE_INTEGER,
          like.minute ?? Number.MIN_SAFE_INTEGER,
          like.second ?? Number.MIN_SAFE_INTEGER,
          like.millisecond ?? Number.MIN_SAFE_INTEGER,
          like.microsecond ?? Number.MIN_SAFE_INTEGER,
          like.nanosecond ?? Number.MIN_SAFE_INTEGER,
          like.calendar ?? null
        ),
      'Failed to update plain date time'
    );
    const components = wrapNativeCall(
      () => NativeTemporal.plainDateTimeGetAllComponents(isoString),
      'Failed to get components'
    );
    return new PlainDateTime(isoString, components);
  }

  withPlainDate(date: PlainDate | PlainDateLike | string): PlainDateTime {
    const d = PlainDate.from(date);
    return this.with({
      year: d.year,
      month: d.month,
      day: d.day,
      calendar: d.calendarId,
    });
  }

  withPlainTime(time: PlainTime | PlainTimeLike | string): PlainDateTime {
    const t = PlainTime.from(time);
    return this.with({
      hour: t.hour,
      minute: t.minute,
      second: t.second,
      millisecond: t.millisecond,
      microsecond: t.microsecond,
      nanosecond: t.nanosecond,
    });
  }

  until(other: PlainDateTime | string | PlainDateTimeLike): Duration {
    const o =
      other instanceof PlainDateTime ? other : PlainDateTime.from(other);
    const durationIso = wrapNativeCall(
      () => NativeTemporal.plainDateTimeUntil(this.#isoString, o.#isoString),
      'Failed to compute until'
    );
    return Duration.from(durationIso);
  }

  since(other: PlainDateTime | string | PlainDateTimeLike): Duration {
    const o =
      other instanceof PlainDateTime ? other : PlainDateTime.from(other);
    const durationIso = wrapNativeCall(
      () => NativeTemporal.plainDateTimeSince(this.#isoString, o.#isoString),
      'Failed to compute since'
    );
    return Duration.from(durationIso);
  }

  equals(other: PlainDateTime | string | PlainDateTimeLike): boolean {
    return PlainDateTime.compare(this, other) === 0;
  }

  toPlainDate(): PlainDate {
    return PlainDate.from({
      year: this.year,
      month: this.month,
      day: this.day,
      calendar: this.calendarId,
    });
  }

  toPlainTime(): PlainTime {
    return PlainTime.from({
      hour: this.hour,
      minute: this.minute,
      second: this.second,
      millisecond: this.millisecond,
      microsecond: this.microsecond,
      nanosecond: this.nanosecond,
    });
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
    const date = new Date(
      this.year,
      this.month - 1,
      this.day,
      this.hour,
      this.minute,
      this.second,
      this.millisecond
    );
    return date.toLocaleString(locales, options);
  }

  valueOf(): never {
    throw new TypeError(
      'Cannot convert a Temporal.PlainDateTime to a primitive'
    );
  }
}

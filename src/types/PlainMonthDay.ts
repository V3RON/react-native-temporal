import NativeTemporal from '../NativeTemporal';
import { wrapNativeCall } from '../utils';
import { PlainDate } from './PlainDate';

export type PlainMonthDayLike = {
  monthCode?: string;
  day?: number;
  month?: number;
  calendar?: string;
};

const enum ComponentIndex {
  Month = 0,
  Day = 1,
}

export class PlainMonthDay {
  readonly #isoString: string;
  readonly #components: number[];
  #monthCode: string | undefined;
  #calendarId: string | undefined;

  private constructor(isoString: string, components: number[]) {
    this.#isoString = isoString;
    this.#components = components;
  }

  static from(item: string | PlainMonthDayLike | PlainMonthDay): PlainMonthDay {
    if (item instanceof PlainMonthDay) return item;

    if (typeof item === 'string') {
      const isoString = wrapNativeCall(
        () => NativeTemporal.plainMonthDayFromString(item),
        `Invalid plain month day string: ${item}`
      );
      if (!isoString) {
        throw new RangeError(`Invalid plain month day string: ${item}`);
      }
      const components = wrapNativeCall(
        () => NativeTemporal.plainMonthDayGetAllComponents(isoString),
        'Failed to get plain month day components'
      );
      return new PlainMonthDay(isoString, components);
    }

    if (typeof item === 'object' && item !== null) {
      const isoString = wrapNativeCall(
        () =>
          NativeTemporal.plainMonthDayFromComponents(
            item.month ?? 0,
            item.day ?? 0,
            item.calendar ?? null,
            0 // referenceYear
          ),
        'Invalid plain month day components'
      );
      if (!isoString) {
        throw new RangeError('Invalid plain month day components');
      }
      const components = wrapNativeCall(
        () => NativeTemporal.plainMonthDayGetAllComponents(isoString),
        'Failed to get plain month day components'
      );
      return new PlainMonthDay(isoString, components);
    }

    throw new TypeError(
      'PlainMonthDay.from requires a string, object, or PlainMonthDay'
    );
  }

  get monthCode(): string {
    if (this.#monthCode === undefined) {
      this.#monthCode = NativeTemporal.plainMonthDayGetMonthCode(
        this.#isoString
      );
    }
    return this.#monthCode!;
  }

  get day(): number {
    return this.#components[ComponentIndex.Day]!;
  }

  get calendarId(): string {
    if (this.#calendarId === undefined) {
      this.#calendarId = NativeTemporal.plainMonthDayGetCalendar(
        this.#isoString
      );
    }
    return this.#calendarId!;
  }

  toPlainDate(item: { year: number }): PlainDate {
    if (
      typeof item !== 'object' ||
      item === null ||
      typeof item.year !== 'number'
    ) {
      throw new TypeError(
        'toPlainDate requires an object with a "year" property'
      );
    }
    const isoString = wrapNativeCall(
      () => NativeTemporal.plainMonthDayToPlainDate(this.#isoString, item.year),
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
    // Fallback using Date with a reference year
    // Note: ISO 8601 reference year is 1972 (leap year) to support Feb 29
    const date = new Date(
      1972,
      this.#components[ComponentIndex.Month]! - 1,
      this.day
    );
    return date.toLocaleString(locales, options);
  }

  valueOf(): never {
    throw new TypeError(
      'Cannot convert a Temporal.PlainMonthDay to a primitive'
    );
  }
}

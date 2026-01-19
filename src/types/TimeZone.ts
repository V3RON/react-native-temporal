import NativeTemporal from '../NativeTemporal';
import { wrapNativeCall } from '../utils';
import { Instant } from './Instant';
import { PlainDateTime } from './PlainDateTime';
import { Calendar } from './Calendar';

export class TimeZone {
  readonly #id: string;

  constructor(id: string) {
    this.#id = id;
  }

  static from(item: string | TimeZone): TimeZone {
    if (item instanceof TimeZone) {
      return item;
    }
    if (typeof item === 'string') {
      const id = wrapNativeCall(
        () => NativeTemporal.timeZoneFromString(item),
        `Invalid timezone: ${item}`
      );
      return new TimeZone(id);
    }
    // TODO: Support parsing from ZonedDateTime or other objects
    throw new TypeError('TimeZone.from requires a string or TimeZone');
  }

  get id(): string {
    return this.#id;
  }

  getOffsetNanosecondsFor(instant: Instant): number {
    return wrapNativeCall(
      () =>
        NativeTemporal.timeZoneGetOffsetNanosecondsFor(
          this.#id,
          instant.toString()
        ),
      'Failed to get offset nanoseconds'
    );
  }

  getOffsetStringFor(instant: Instant): string {
    return wrapNativeCall(
      () =>
        NativeTemporal.timeZoneGetOffsetStringFor(this.#id, instant.toString()),
      'Failed to get offset string'
    );
  }

  getPlainDateTimeFor(
    instant: Instant,
    calendarLike: string | Calendar = 'iso8601'
  ): PlainDateTime {
    const calendar = Calendar.from(calendarLike);
    const dtStr = wrapNativeCall(
      () =>
        NativeTemporal.timeZoneGetPlainDateTimeFor(
          this.#id,
          instant.toString(),
          calendar.id
        ),
      'Failed to get plain date time'
    );
    return PlainDateTime.from(dtStr);
  }

  getInstantFor(
    dateTime: PlainDateTime,
    options?: { disambiguation?: 'compatible' | 'earlier' | 'later' | 'reject' }
  ): Instant {
    const disambiguation = options?.disambiguation ?? 'compatible';
    const instantStr = wrapNativeCall(
      () =>
        NativeTemporal.timeZoneGetInstantFor(
          this.#id,
          dateTime.toString(),
          disambiguation
        ),
      'Failed to get instant'
    );
    return Instant.from(instantStr);
  }

  getNextTransition(instant: Instant): Instant | null {
    const nextStr = wrapNativeCall(
      () =>
        NativeTemporal.timeZoneGetNextTransition(this.#id, instant.toString()),
      'Failed to get next transition'
    );
    return nextStr ? Instant.from(nextStr) : null;
  }

  getPreviousTransition(instant: Instant): Instant | null {
    const prevStr = wrapNativeCall(
      () =>
        NativeTemporal.timeZoneGetPreviousTransition(
          this.#id,
          instant.toString()
        ),
      'Failed to get previous transition'
    );
    return prevStr ? Instant.from(prevStr) : null;
  }

  toString(): string {
    return this.#id;
  }

  toJSON(): string {
    return this.toString();
  }
}

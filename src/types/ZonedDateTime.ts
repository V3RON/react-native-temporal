import NativeTemporal from '../NativeTemporal';
import { wrapNativeCall } from '../utils';
import { Calendar } from './Calendar';
import { TimeZone } from './TimeZone';
import { Duration } from './Duration';
import { Instant } from './Instant';
import { PlainDate } from './PlainDate';
import { PlainTime } from './PlainTime';
import { PlainDateTime } from './PlainDateTime';

export class ZonedDateTime {
  readonly #iso: string;
  readonly #calendar: Calendar;
  readonly #timeZone: TimeZone;

  private constructor(iso: string, calendar: Calendar, timeZone: TimeZone) {
    this.#iso = iso;
    this.#calendar = calendar;
    this.#timeZone = timeZone;
  }

  static from(item: string | ZonedDateTime): ZonedDateTime {
    if (item instanceof ZonedDateTime) {
      return item;
    }

    if (typeof item === 'string') {
      const iso = wrapNativeCall(
        () => NativeTemporal.zonedDateTimeFromString(item),
        'Invalid ZonedDateTime string'
      );
      const calendarId = NativeTemporal.zonedDateTimeGetCalendar(iso);
      const timeZoneId = NativeTemporal.zonedDateTimeGetTimeZone(iso);
      return new ZonedDateTime(
        iso,
        Calendar.from(calendarId),
        TimeZone.from(timeZoneId)
      );
    }

    throw new TypeError(
      'ZonedDateTime.from only supports strings and ZonedDateTime instances currently'
    );
  }

  get year(): number {
    return this.#getComponent(0);
  }
  get month(): number {
    return this.#getComponent(1);
  }
  get day(): number {
    return this.#getComponent(2);
  }
  get dayOfWeek(): number {
    return this.#getComponent(3);
  }
  get dayOfYear(): number {
    return this.#getComponent(4);
  }
  get weekOfYear(): number {
    return this.#getComponent(5);
  }
  get yearOfWeek(): number {
    return this.#getComponent(6);
  }
  get daysInWeek(): number {
    return this.#getComponent(7);
  }
  get daysInMonth(): number {
    return this.#getComponent(8);
  }
  get daysInYear(): number {
    return this.#getComponent(9);
  }
  get monthsInYear(): number {
    return this.#getComponent(10);
  }
  get inLeapYear(): boolean {
    return this.#getComponent(11) === 1;
  }
  get hoursInDay(): number {
    const currentStart = this.startOfDay();
    const pd = currentStart.toPlainDate();
    const nextPd = pd.add({ days: 1 });
    const nextPdt = PlainDateTime.from({
      year: nextPd.year,
      month: nextPd.month,
      day: nextPd.day,
      calendar: this.calendarId,
    });
    const nextInstant = this.timeZone.getInstantFor(nextPdt);
    const startInstant = currentStart.toInstant();
    const diffNs = nextInstant.epochNanoseconds - startInstant.epochNanoseconds;
    return Number(diffNs / 3600000000000n);
  }
  get hour(): number {
    return this.#getComponent(12);
  }
  get minute(): number {
    return this.#getComponent(13);
  }
  get second(): number {
    return this.#getComponent(14);
  }
  get millisecond(): number {
    return this.#getComponent(15);
  }
  get microsecond(): number {
    return this.#getComponent(16);
  }
  get nanosecond(): number {
    return this.#getComponent(17);
  }
  get offsetNanoseconds(): number {
    return this.#getComponent(18);
  }

  get offset(): string {
    return NativeTemporal.zonedDateTimeGetOffset(this.#iso);
  }

  get epochMilliseconds(): number {
    return NativeTemporal.zonedDateTimeEpochMilliseconds(this.#iso);
  }

  // Note: returns string to preserve precision if needed, but TS types might want BigInt?
  // Using string for now or BigInt if environment supports it (RN usually does now)
  get epochNanoseconds(): bigint {
    return BigInt(NativeTemporal.zonedDateTimeEpochNanoseconds(this.#iso));
  }

  get calendar(): Calendar {
    return this.#calendar;
  }
  get timeZone(): TimeZone {
    return this.#timeZone;
  }
  get calendarId(): string {
    return this.#calendar.id;
  }
  get timeZoneId(): string {
    return this.#timeZone.id;
  }

  #components: number[] | null = null;
  #getComponent(index: number): number {
    if (!this.#components) {
      this.#components = NativeTemporal.zonedDateTimeGetAllComponents(
        this.#iso
      );
    }
    return this.#components[index]!;
  }

  add(duration: Duration | string | object): ZonedDateTime {
    const d = Duration.from(duration);
    const newIso = wrapNativeCall(
      () => NativeTemporal.zonedDateTimeAdd(this.#iso, d.toString()),
      'Add failed'
    );
    return this.#clone(newIso);
  }

  subtract(duration: Duration | string | object): ZonedDateTime {
    const d = Duration.from(duration);
    const newIso = wrapNativeCall(
      () => NativeTemporal.zonedDateTimeSubtract(this.#iso, d.toString()),
      'Subtract failed'
    );
    return this.#clone(newIso);
  }

  until(other: ZonedDateTime, _options?: any): Duration {
    const durStr = wrapNativeCall(
      () => NativeTemporal.zonedDateTimeUntil(this.#iso, other.toString()),
      'Until failed'
    );
    return Duration.from(durStr);
  }

  since(other: ZonedDateTime, _options?: any): Duration {
    const durStr = wrapNativeCall(
      () => NativeTemporal.zonedDateTimeSince(this.#iso, other.toString()),
      'Since failed'
    );
    return Duration.from(durStr);
  }

  round(options: {
    smallestUnit: string;
    roundingIncrement?: number;
    roundingMode?: string;
  }): ZonedDateTime {
    const newIso = wrapNativeCall(
      () =>
        NativeTemporal.zonedDateTimeRound(
          this.#iso,
          options.smallestUnit,
          options.roundingIncrement ?? 1,
          options.roundingMode ?? null
        ),
      'Round failed'
    );
    return this.#clone(newIso);
  }

  with(
    fields: {
      year?: number;
      month?: number;
      day?: number;
      hour?: number;
      minute?: number;
      second?: number;
      millisecond?: number;
      microsecond?: number;
      nanosecond?: number;
      offset?: string;
      calendar?: string | Calendar;
      timeZone?: string | TimeZone;
    },
    _options?: any
  ): ZonedDateTime {
    const MIN = Number.MIN_SAFE_INTEGER;

    const newIso = wrapNativeCall(
      () =>
        NativeTemporal.zonedDateTimeWith(
          this.#iso,
          fields.year ?? MIN,
          fields.month ?? MIN,
          fields.day ?? MIN,
          fields.hour ?? MIN,
          fields.minute ?? MIN,
          fields.second ?? MIN,
          fields.millisecond ?? MIN,
          fields.microsecond ?? MIN,
          fields.nanosecond ?? MIN,
          MIN, // offset_ns todo
          fields.calendar ? fields.calendar.toString() : null,
          fields.timeZone ? fields.timeZone.toString() : null
        ),
      'With failed'
    );

    return this.#clone(newIso);
  }

  startOfDay(): ZonedDateTime {
    const pd = this.toPlainDate();
    const pdt = PlainDateTime.from({
      year: pd.year,
      month: pd.month,
      day: pd.day,
      calendar: this.calendarId,
    });
    const instant = this.timeZone.getInstantFor(pdt);
    return instant.toZonedDateTime({
      timeZone: this.timeZone,
      calendar: this.calendar,
    });
  }

  toInstant(): Instant {
    const s = NativeTemporal.zonedDateTimeToInstant(this.#iso);
    return Instant.from(s);
  }

  toPlainDate(): PlainDate {
    const s = NativeTemporal.zonedDateTimeToPlainDate(this.#iso);
    return PlainDate.from(s);
  }

  toPlainTime(): PlainTime {
    const s = NativeTemporal.zonedDateTimeToPlainTime(this.#iso);
    return PlainTime.from(s);
  }

  toPlainDateTime(): PlainDateTime {
    const s = NativeTemporal.zonedDateTimeToPlainDateTime(this.#iso);
    return PlainDateTime.from(s);
  }

  toString(_options?: any): string {
    return this.#iso;
  }

  equals(other: ZonedDateTime | string): boolean {
    const otherZdt = ZonedDateTime.from(other);
    return ZonedDateTime.compare(this, otherZdt) === 0;
  }

  toJSON(): string {
    return this.toString();
  }

  static compare(one: ZonedDateTime, two: ZonedDateTime): -1 | 0 | 1 {
    return NativeTemporal.zonedDateTimeCompare(
      one.toString(),
      two.toString()
    ) as -1 | 0 | 1;
  }

  #clone(iso: string): ZonedDateTime {
    const calendarId = NativeTemporal.zonedDateTimeGetCalendar(iso);
    const timeZoneId = NativeTemporal.zonedDateTimeGetTimeZone(iso);
    return new ZonedDateTime(
      iso,
      Calendar.from(calendarId),
      TimeZone.from(timeZoneId)
    );
  }
}

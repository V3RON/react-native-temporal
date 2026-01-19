import { Duration } from './types/Duration';
import { Instant } from './types/Instant';
import { Now } from './types/Now';
import { PlainTime } from './types/PlainTime';
import { PlainDate } from './types/PlainDate';
import { PlainDateTime } from './types/PlainDateTime';
import { PlainMonthDay } from './types/PlainMonthDay';
import { PlainYearMonth } from './types/PlainYearMonth';
import { Calendar } from './types/Calendar';
import { TimeZone } from './types/TimeZone';
import { ZonedDateTime } from './types/ZonedDateTime';

declare global {
  var Temporal: {
    Duration: typeof Duration;
    Instant: typeof Instant;
    Now: typeof Now;
    PlainTime: typeof PlainTime;
    PlainDate: typeof PlainDate;
    PlainDateTime: typeof PlainDateTime;
    PlainMonthDay: typeof PlainMonthDay;
    PlainYearMonth: typeof PlainYearMonth;
    Calendar: typeof Calendar;
    TimeZone: typeof TimeZone;
    ZonedDateTime: typeof ZonedDateTime;
  };
}

globalThis.Temporal = globalThis.Temporal || {
  Duration,
  Instant,
  Now,
  PlainTime,
  PlainDate,
  PlainDateTime,
  PlainMonthDay,
  PlainYearMonth,
  Calendar,
  TimeZone,
  ZonedDateTime,
};

import { Duration } from './types/Duration';
import { Instant } from './types/Instant';
import { Now } from './types/Now';
import { PlainTime } from './types/PlainTime';
import { PlainDate } from './types/PlainDate';
import { Calendar } from './types/Calendar';

declare global {
  var Temporal: {
    Duration: typeof Duration;
    Instant: typeof Instant;
    Now: typeof Now;
    PlainTime: typeof PlainTime;
    PlainDate: typeof PlainDate;
    Calendar: typeof Calendar;
  };
}

globalThis.Temporal = globalThis.Temporal || {
  Duration,
  Instant,
  Now,
  PlainTime,
  PlainDate,
  Calendar,
};

import { Duration } from './types/Duration';
import { Instant } from './types/Instant';
import { Now } from './types/Now';
import { PlainTime } from './types/PlainTime';

declare global {
  var Temporal: {
    Duration: typeof Duration;
    Instant: typeof Instant;
    Now: typeof Now;
    PlainTime: typeof PlainTime;
  };
}

globalThis.Temporal = globalThis.Temporal || {
  Duration,
  Instant,
  Now,
  PlainTime,
};

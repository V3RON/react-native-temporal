import { Duration } from './types/Duration';
import { Instant } from './types/Instant';
import { Now } from './types/Now';

declare global {
  var Temporal: {
    Duration: typeof Duration;
    Instant: typeof Instant;
    Now: typeof Now;
  };
}

globalThis.Temporal = globalThis.Temporal || {
  Duration,
  Instant,
  Now,
};

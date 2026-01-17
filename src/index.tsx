import Temporal from './NativeTemporal';

export function multiply(a: number, b: number): number {
  return Temporal.multiply(a, b);
}

// Export Temporal types
export { Instant } from './types/Instant';
export { Duration } from './types/Duration';
export { Now } from './types/Now';
export { PlainTime } from './types/PlainTime';
export { Calendar } from './types/Calendar';
export type { DurationLike } from './types/Duration';
export type { PlainTimeLike } from './types/PlainTime';

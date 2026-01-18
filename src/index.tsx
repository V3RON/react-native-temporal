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
export { PlainDate } from './types/PlainDate';
export { PlainDateTime } from './types/PlainDateTime';
export { PlainYearMonth } from './types/PlainYearMonth';
export { PlainMonthDay } from './types/PlainMonthDay';
export type { DurationLike } from './types/Duration';
export type { PlainTimeLike } from './types/PlainTime';
export type { PlainDateLike } from './types/PlainDate';
export type { PlainDateTimeLike } from './types/PlainDateTime';
export type { PlainYearMonthLike } from './types/PlainYearMonth';
export type { PlainMonthDayLike } from './types/PlainMonthDay';

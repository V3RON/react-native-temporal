import Temporal from './NativeTemporal';

export function multiply(a: number, b: number): number {
  return Temporal.multiply(a, b);
}

export function instantNow(): string {
  return Temporal.instantNow();
}

export const Instant = {
  now: instantNow,
};

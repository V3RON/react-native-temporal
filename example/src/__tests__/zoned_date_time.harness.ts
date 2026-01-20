import { describe, it, expect } from 'react-native-harness';
import { ZonedDateTime } from 'react-native-temporal';

describe('ZonedDateTime', () => {
  describe('Creation', () => {
    it('should create from ISO string', () => {
      const zdt = ZonedDateTime.from('2020-01-01T00:00:00+01:00[Europe/Paris]');
      expect(zdt.toString()).toBe('2020-01-01T00:00:00+01:00[Europe/Paris]');
      expect(zdt.year).toBe(2020);
      expect(zdt.month).toBe(1);
      expect(zdt.day).toBe(1);
      expect(zdt.hour).toBe(0);
      expect(zdt.offset).toBe('+01:00');
      expect(zdt.timeZoneId).toBe('Europe/Paris');
    });

    it('should create from object (not fully implemented in JS yet but via from())', () => {
      // ZonedDateTime.from(obj) delegates to native
      // NOTE: Current TS implementation only supports string or ZonedDateTime instance in from()
      // If we implemented object parsing in native, we should update TS.
      // Checking TS implementation:
      // if (typeof item === 'string') { ... }
      // throw new TypeError('ZonedDateTime.from only supports strings and ZonedDateTime instances currently');
      // So skipping object test for now until TS wrapper is updated.
    });
  });

  describe('Arithmetic', () => {
    it('should add duration', () => {
      const zdt = ZonedDateTime.from('2020-01-01T00:00:00+01:00[Europe/Paris]');
      const result = zdt.add('PT1H');
      expect(result.toString()).toBe('2020-01-01T01:00:00+01:00[Europe/Paris]');
    });

    it('should subtract duration', () => {
      const zdt = ZonedDateTime.from('2020-01-01T01:00:00+01:00[Europe/Paris]');
      const result = zdt.subtract('PT1H');
      expect(result.toString()).toBe('2020-01-01T00:00:00+01:00[Europe/Paris]');
    });

    it('should handle DST transitions (adding hours)', () => {
      // Europe/Paris DST start 2020: March 29, 02:00 -> 03:00
      const before = ZonedDateTime.from(
        '2020-03-29T01:00:00+01:00[Europe/Paris]'
      );
      const after = before.add('PT2H'); // 1:00 + 2h = 3:00 which is 4:00 DST? No, 1:00 + 2h = 3:00 local time?
      // 1:00 UTC+1 is 0:00 UTC.
      // + 2 hours = 2:00 UTC.
      // 2:00 UTC in Paris is 4:00 UTC+2 (DST).
      expect(after.toString()).toBe('2020-03-29T04:00:00+02:00[Europe/Paris]');
    });
  });

  describe('Comparison', () => {
    it('should compare correctly', () => {
      const z1 = ZonedDateTime.from('2020-01-01T00:00:00+00:00[UTC]');
      const z2 = ZonedDateTime.from('2020-01-01T01:00:00+00:00[UTC]');
      expect(ZonedDateTime.compare(z1, z2)).toBe(-1);
      expect(ZonedDateTime.compare(z2, z1)).toBe(1);
      expect(ZonedDateTime.compare(z1, z1)).toBe(0);
    });

    it('should check equality', () => {
      const z1 = ZonedDateTime.from('2020-01-01T00:00:00+00:00[UTC]');
      const z2 = ZonedDateTime.from('2020-01-01T00:00:00+00:00[UTC]');
      const z3 = ZonedDateTime.from('2020-01-01T01:00:00+00:00[UTC]');
      expect(z1.equals(z2)).toBe(true);
      expect(z1.equals(z3)).toBe(false);
      expect(z1.equals('2020-01-01T00:00:00+00:00[UTC]')).toBe(true);
    });
  });

  describe('Start of Day & Hours in Day', () => {
    it('should get start of day', () => {
      const zdt = ZonedDateTime.from('2020-01-15T15:30:00+01:00[Europe/Paris]');
      const start = zdt.startOfDay();
      expect(start.toString()).toBe('2020-01-15T00:00:00+01:00[Europe/Paris]');
    });

    it('should get hours in day (regular day)', () => {
      const zdt = ZonedDateTime.from('2020-01-15T15:30:00+01:00[Europe/Paris]');
      expect(zdt.hoursInDay).toBe(24);
    });

    it('should get hours in day (DST start - 23h)', () => {
      // Europe/Paris DST start 2020-03-29
      const zdt = ZonedDateTime.from('2020-03-29T12:00:00+02:00[Europe/Paris]');
      expect(zdt.hoursInDay).toBe(23);
    });

    it('should get hours in day (DST end - 25h)', () => {
      // Europe/Paris DST end 2020-10-25
      const zdt = ZonedDateTime.from('2020-10-25T12:00:00+01:00[Europe/Paris]');
      expect(zdt.hoursInDay).toBe(25);
    });
  });

  describe('Difference', () => {
    it('should calculate until', () => {
      const start = ZonedDateTime.from('2020-01-01T00:00:00+00:00[UTC]');
      const end = ZonedDateTime.from('2020-01-01T01:00:00+00:00[UTC]');
      const diff = start.until(end);
      expect(diff.toString()).toBe('PT1H');
    });

    it('should calculate since', () => {
      const start = ZonedDateTime.from('2020-01-01T01:00:00+00:00[UTC]');
      const end = ZonedDateTime.from('2020-01-01T00:00:00+00:00[UTC]');
      const diff = start.since(end);
      expect(diff.toString()).toBe('PT1H');
    });
  });

  describe('Rounding', () => {
    it('should round to nearest hour', () => {
      const zdt = ZonedDateTime.from('2020-01-01T00:30:00+00:00[UTC]');
      const rounded = zdt.round({ smallestUnit: 'hour' });
      expect(rounded.toString()).toBe('2020-01-01T01:00:00+00:00[UTC]');
    });
  });

  describe('Conversions', () => {
    it('should convert to Instant', () => {
      const zdt = ZonedDateTime.from('2020-01-01T00:00:00+00:00[UTC]');
      const instant = zdt.toInstant();
      expect(instant.toString()).toBe('2020-01-01T00:00:00Z');
    });

    it('should convert to PlainDate', () => {
      const zdt = ZonedDateTime.from('2020-01-01T15:00:00+00:00[UTC]');
      const date = zdt.toPlainDate();
      expect(date.toString()).toBe('2020-01-01');
    });

    it('should convert to PlainTime', () => {
      const zdt = ZonedDateTime.from('2020-01-01T15:30:00+00:00[UTC]');
      const time = zdt.toPlainTime();
      expect(time.toString()).toBe('15:30:00');
    });

    it('should convert to PlainDateTime', () => {
      const zdt = ZonedDateTime.from('2020-01-01T15:30:00+00:00[UTC]');
      const dt = zdt.toPlainDateTime();
      expect(dt.toString()).toBe('2020-01-01T15:30:00');
    });
  });
});

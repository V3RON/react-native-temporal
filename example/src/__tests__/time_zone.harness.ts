import { describe, it, expect } from 'react-native-harness';
import { TimeZone, Instant, PlainDateTime } from 'react-native-temporal';

describe('TimeZone', () => {
  describe('Creation', () => {
    it('should create from ID', () => {
      const tz = TimeZone.from('Europe/London');
      expect(tz.id).toBe('Europe/London');
      expect(tz.toString()).toBe('Europe/London');
    });

    it('should create from fixed offset', () => {
      const tz = TimeZone.from('+05:30');
      expect(tz.id).toBe('+05:30');
    });

    it('should throw for invalid ID', () => {
      expect(() => TimeZone.from('Invalid/TimeZone')).toThrow();
    });
  });

  describe('Offsets', () => {
    it('should get offset string for instant', () => {
      const tz = TimeZone.from('Europe/London');
      const instant = Instant.from('2020-01-01T00:00:00Z');
      expect(tz.getOffsetStringFor(instant)).toBe('+00:00');

      const summerInstant = Instant.from('2020-07-01T00:00:00Z');
      expect(tz.getOffsetStringFor(summerInstant)).toBe('+01:00');
    });

    it('should get offset nanoseconds for instant', () => {
      const tz = TimeZone.from('Europe/London');
      const instant = Instant.from('2020-01-01T00:00:00Z');
      expect(tz.getOffsetNanosecondsFor(instant)).toBe(0);

      const summerInstant = Instant.from('2020-07-01T00:00:00Z');
      expect(tz.getOffsetNanosecondsFor(summerInstant)).toBe(3600000000000); // 1 hour in ns
    });
  });

  describe('Instant / DateTime conversions', () => {
    it('should get PlainDateTime for Instant', () => {
      const tz = TimeZone.from('Europe/London');
      const instant = Instant.from('2020-01-01T00:00:00Z');
      const dt = tz.getPlainDateTimeFor(instant);
      expect(dt.toString()).toBe('2020-01-01T00:00:00');
    });

    it('should get Instant for PlainDateTime', () => {
      const tz = TimeZone.from('Europe/London');
      const dt = PlainDateTime.from('2020-01-01T00:00:00');
      const instant = tz.getInstantFor(dt);
      expect(instant.toString()).toBe('2020-01-01T00:00:00Z');
    });

    it('should handle disambiguation (skipped time)', () => {
      const tz = TimeZone.from('Europe/Berlin');
      // 2020-03-29 02:00 -> 03:00 (spring forward)
      // 02:30 doesn't exist
      const dt = PlainDateTime.from('2020-03-29T02:30:00');

      // Default compatible = later (03:30)
      const instant = tz.getInstantFor(dt);
      // 3:30 local is 1:30 UTC. Wait.
      // 2:00 -> 3:00 jump. 2:30 is in the gap.
      // 'compatible' modes usually picks the time after the gap (adjusted by gap duration).
      // gap is 1 hour. 2:30 -> 3:30.
      // 3:30 Europe/Berlin (UTC+2) is 1:30 UTC.
      expect(instant.toString()).toBe('2020-03-29T01:30:00Z');
    });
  });

  describe('Transitions', () => {
    it('should get next transition', () => {
      const tz = TimeZone.from('Europe/London');
      // Before March 2020 transition (29th March 01:00 UTC)
      const instant = Instant.from('2020-03-28T00:00:00Z');
      const next = tz.getNextTransition(instant);
      expect(next?.toString()).toBe('2020-03-29T01:00:00Z');
    });

    it('should get previous transition', () => {
      const tz = TimeZone.from('Europe/London');
      // After March 2020 transition
      const instant = Instant.from('2020-03-30T00:00:00Z');
      const prev = tz.getPreviousTransition(instant);
      expect(prev?.toString()).toBe('2020-03-29T01:00:00Z');
    });
  });
});

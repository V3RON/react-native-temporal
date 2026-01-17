import { describe, it, expect } from 'react-native-harness';
import { Now, Instant } from 'react-native-temporal';

describe('Now', () => {
  describe('Now.instant', () => {
    it('should return a valid Instant', () => {
      const instant = Now.instant();
      expect(instant).toBeInstanceOf(Instant);
      // Basic sanity check: year should be > 2020
      const currentYear = parseInt(instant.toString().substring(0, 4), 10);
      expect(currentYear).toBeGreaterThan(2020);
    });
  });

  describe('Now.timeZoneId', () => {
    it('should return a string timezone ID', () => {
      const tz = Now.timeZoneId();
      expect(typeof tz).toBe('string');
      expect(tz.length).toBeGreaterThan(0);
    });
  });

  describe('Now.plainDateTimeISO', () => {
    it('should return a valid ISO string', () => {
      const dt = Now.plainDateTimeISO();
      // Regex for roughly ISO 8601 DateTime
      expect(dt).toMatch(/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}/);
    });
  });

  describe('Now.plainDateISO', () => {
    it('should return a valid ISO date string', () => {
      const d = Now.plainDateISO();
      expect(d).toMatch(/^\d{4}-\d{2}-\d{2}$/);
    });
  });

  describe('Now.plainTimeISO', () => {
    it('should return a valid ISO time string', () => {
      const t = Now.plainTimeISO();
      expect(t).toMatch(/^\d{2}:\d{2}:\d{2}/);
    });
  });
});

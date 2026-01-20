import { describe, it, expect } from 'react-native-harness';
import { PlainDateTime } from 'react-native-temporal';

describe('PlainDateTime', () => {
  describe('Creation', () => {
    it('should create from ISO string', () => {
      const dt = PlainDateTime.from('2020-01-01T12:30:45');
      expect(dt.toString()).toBe('2020-01-01T12:30:45');
      expect(dt.year).toBe(2020);
      expect(dt.month).toBe(1);
      expect(dt.day).toBe(1);
      expect(dt.hour).toBe(12);
      expect(dt.minute).toBe(30);
      expect(dt.second).toBe(45);
    });

    it('should create from object', () => {
      const dt = PlainDateTime.from({
        year: 2020,
        month: 1,
        day: 1,
        hour: 12,
        minute: 30,
      });
      expect(dt.toString()).toBe('2020-01-01T12:30:00');
    });
  });

  describe('Arithmetic', () => {
    it('should add duration', () => {
      const dt = PlainDateTime.from('2020-01-01T12:00:00');
      const result = dt.add('P1DT1H');
      expect(result.toString()).toBe('2020-01-02T13:00:00');
    });

    it('should subtract duration', () => {
      const dt = PlainDateTime.from('2020-01-02T13:00:00');
      const result = dt.subtract('P1DT1H');
      expect(result.toString()).toBe('2020-01-01T12:00:00');
    });
  });

  describe('Comparison', () => {
    it('should compare correctly', () => {
      const dt1 = PlainDateTime.from('2020-01-01T12:00:00');
      const dt2 = PlainDateTime.from('2020-01-01T13:00:00');
      expect(PlainDateTime.compare(dt1, dt2)).toBe(-1);
      expect(PlainDateTime.compare(dt2, dt1)).toBe(1);
      expect(PlainDateTime.compare(dt1, dt1)).toBe(0);
    });
  });

  describe('Difference', () => {
    it('should calculate until', () => {
      const start = PlainDateTime.from('2020-01-01T12:00:00');
      const end = PlainDateTime.from('2020-01-02T13:00:00');
      const diff = start.until(end);
      expect(diff.toString()).toBe('P1DT1H');
    });

    it('should calculate since', () => {
      const start = PlainDateTime.from('2020-01-02T13:00:00');
      const end = PlainDateTime.from('2020-01-01T12:00:00');
      const diff = start.since(end);
      expect(diff.toString()).toBe('P1DT1H');
    });
  });

  /*
  describe('Rounding', () => {
    it('should round to nearest hour', () => {
      const dt = PlainDateTime.from('2020-01-01T12:30:00');
      const rounded = dt.round({ smallestUnit: 'hour' });
      expect(rounded.toString()).toBe('2020-01-01T13:00:00');
    });
  });
*/

  describe('Conversions', () => {
    it('should convert to PlainDate', () => {
      const dt = PlainDateTime.from('2020-01-01T12:30:00');
      expect(dt.toPlainDate().toString()).toBe('2020-01-01');
    });

    it('should convert to PlainTime', () => {
      const dt = PlainDateTime.from('2020-01-01T12:30:00');
      expect(dt.toPlainTime().toString()).toBe('12:30:00');
    });
  });
});

import { describe, it, expect } from 'react-native-harness';
import { PlainDate } from 'react-native-temporal';

describe('PlainDate', () => {
  describe('PlainDate.from', () => {
    it('should create from ISO string', () => {
      const date = PlainDate.from('2024-01-17');
      expect(date.year).toBe(2024);
      expect(date.month).toBe(1);
      expect(date.day).toBe(17);
    });

    it('should create from object', () => {
      const date = PlainDate.from({ year: 2024, month: 1, day: 17 });
      expect(date.toString()).toBe('2024-01-17');
    });

    it('should throw for invalid inputs', () => {
      expect(() => PlainDate.from('invalid')).toThrow();
    });
  });

  describe('Components', () => {
    it('should have correct getters', () => {
      const date = PlainDate.from('2024-01-17');
      expect(date.year).toBe(2024);
      expect(date.month).toBe(1);
      expect(date.day).toBe(17);
      expect(date.dayOfWeek).toBe(3); // Wednesday
      expect(date.daysInWeek).toBe(7);
      expect(date.monthsInYear).toBe(12);
      expect(date.inLeapYear).toBe(true);
      expect(date.monthCode).toBe('M01');
    });
  });

  describe('Comparison', () => {
    it('should compare correctly', () => {
      const d1 = PlainDate.from('2024-01-17');
      const d2 = PlainDate.from('2024-01-18');
      expect(PlainDate.compare(d1, d2)).toBe(-1);
      expect(PlainDate.compare(d2, d1)).toBe(1);
      expect(d1.equals('2024-01-17')).toBe(true);
    });
  });

  describe('Arithmetic', () => {
    it('should add duration', () => {
      const date = PlainDate.from('2024-01-17');
      const result = date.add('P1D');
      expect(result.toString()).toBe('2024-01-18');
    });

    it('should subtract duration', () => {
      const date = PlainDate.from('2024-01-17');
      const result = date.subtract('P1M');
      expect(result.toString()).toBe('2023-12-17');
    });

    it('should handle complex arithmetic', () => {
      const date = PlainDate.from('2024-01-31');
      const result = date.add({ months: 1 });
      // 2024 is leap year, Feb has 29 days.
      // Adding 1 month to Jan 31 -> Feb 29 (constrained)
      expect(result.toString()).toBe('2024-02-29');
    });
  });

  describe('PlainDate.prototype.with', () => {
    it('should replace fields', () => {
      const date = PlainDate.from('2024-01-17');
      const result = date.with({ day: 20 });
      expect(result.toString()).toBe('2024-01-20');
    });
  });

  describe('Difference', () => {
    it('should compute until', () => {
      const d1 = PlainDate.from('2024-01-01');
      const d2 = PlainDate.from('2024-02-01');
      const diff = d1.until(d2);
      expect(diff.days).toBe(31);
    });

    it('should compute since', () => {
      const d1 = PlainDate.from('2024-02-01');
      const d2 = PlainDate.from('2024-01-01');
      const diff = d1.since(d2);
      expect(diff.days).toBe(31);
    });
  });

  describe('PlainDate.prototype.valueOf', () => {
    it('should throw TypeError on valueOf', () => {
      const date = PlainDate.from('2024-01-17');
      expect(() => date.valueOf()).toThrow();
    });
  });
});

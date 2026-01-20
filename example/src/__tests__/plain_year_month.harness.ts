import { describe, it, expect } from 'react-native-harness';
import { PlainYearMonth } from 'react-native-temporal';

describe('PlainYearMonth', () => {
  describe('Creation', () => {
    it('should create from ISO string', () => {
      const ym = PlainYearMonth.from('2020-01');
      expect(ym.toString()).toBe('2020-01');
      expect(ym.year).toBe(2020);
      expect(ym.month).toBe(1);
    });

    it('should create from object', () => {
      const ym = PlainYearMonth.from({ year: 2020, month: 1 });
      expect(ym.toString()).toBe('2020-01');
    });
  });

  describe('Arithmetic', () => {
    it('should add duration', () => {
      const ym = PlainYearMonth.from('2020-01');
      const result = ym.add('P1M');
      expect(result.toString()).toBe('2020-02');
    });

    it('should subtract duration', () => {
      const ym = PlainYearMonth.from('2020-02');
      const result = ym.subtract('P1M');
      expect(result.toString()).toBe('2020-01');
    });
  });

  describe('Comparison', () => {
    it('should compare correctly', () => {
      const ym1 = PlainYearMonth.from('2020-01');
      const ym2 = PlainYearMonth.from('2020-02');
      expect(PlainYearMonth.compare(ym1, ym2)).toBe(-1);
      expect(PlainYearMonth.compare(ym2, ym1)).toBe(1);
      expect(PlainYearMonth.compare(ym1, ym1)).toBe(0);
    });
  });

  describe('Difference', () => {
    it('should calculate until', () => {
      const start = PlainYearMonth.from('2020-01');
      const end = PlainYearMonth.from('2021-01');
      const diff = start.until(end);
      expect(diff.toString()).toBe('P1Y'); // or P12M depending on implementation default
    });

    it('should calculate since', () => {
      const start = PlainYearMonth.from('2021-01');
      const end = PlainYearMonth.from('2020-01');
      const diff = start.since(end);
      expect(diff.toString()).toBe('P1Y');
    });
  });

  describe('Conversions', () => {
    it('should convert to PlainDate', () => {
      const ym = PlainYearMonth.from('2020-01');
      const date = ym.toPlainDate({ day: 15 });
      expect(date.toString()).toBe('2020-01-15');
    });
  });
});

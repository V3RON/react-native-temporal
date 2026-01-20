import { describe, it, expect } from 'react-native-harness';
import { PlainMonthDay } from 'react-native-temporal';

describe('PlainMonthDay', () => {
  describe('Creation', () => {
    it('should create from ISO string', () => {
      const md = PlainMonthDay.from('01-01');
      const str = md.toString();
      // Implementation might return '1972-01-01' (reference year) or 'M01-01' or similar.
      // We check that it contains the month and day.
      expect(str.includes('01-01')).toBe(true);
      expect(md.monthCode).toBe('M01');
      expect(md.day).toBe(1);
    });

    it('should create from object', () => {
      const md = PlainMonthDay.from({ month: 1, day: 1 });
      expect(md.toString().includes('01-01')).toBe(true);
    });
  });

  describe('Conversions', () => {
    it('should convert to PlainDate', () => {
      const md = PlainMonthDay.from('01-01');
      const date = md.toPlainDate({ year: 2020 });
      expect(date.toString()).toBe('2020-01-01');
    });
  });
});

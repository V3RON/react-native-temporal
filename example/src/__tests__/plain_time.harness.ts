import { describe, it, expect } from 'react-native-harness';
import { PlainTime } from 'react-native-temporal';

describe('PlainTime', () => {
  describe('PlainTime.from', () => {
    it('should create from ISO string', () => {
      const time = PlainTime.from('12:30:45');
      expect(time.hour).toBe(12);
      expect(time.minute).toBe(30);
      expect(time.second).toBe(45);
    });

    it('should create from object', () => {
      const time = PlainTime.from({ hour: 12, minute: 30 });
      expect(time.toString()).toBe('12:30:00');
    });

    it('should throw for invalid inputs', () => {
      expect(() => PlainTime.from('invalid')).toThrow();
      expect(() => PlainTime.from({ hour: 25 })).toThrow(); // Invalid hour
    });
  });

  describe('Components', () => {
    it('should have correct getters', () => {
      const time = PlainTime.from('12:30:45.123456789');
      expect(time.hour).toBe(12);
      expect(time.minute).toBe(30);
      expect(time.second).toBe(45);
      expect(time.millisecond).toBe(123);
      expect(time.microsecond).toBe(456);
      expect(time.nanosecond).toBe(789);
    });
  });

  describe('Comparison', () => {
    it('should compare correctly', () => {
      const t1 = PlainTime.from('12:00');
      const t2 = PlainTime.from('13:00');
      expect(PlainTime.compare(t1, t2)).toBe(-1);
      expect(PlainTime.compare(t2, t1)).toBe(1);
      expect(t1.equals('12:00')).toBe(true);
    });
  });

  describe('Arithmetic', () => {
    it('should add duration', () => {
      const time = PlainTime.from('12:00');
      const result = time.add('PT1H30M');
      expect(result.toString()).toBe('13:30:00');
    });

    it('should subtract duration', () => {
      const time = PlainTime.from('12:00');
      const result = time.subtract('PT1H');
      expect(result.toString()).toBe('11:00:00');
    });

    it('should rollover correctly', () => {
      const time = PlainTime.from('23:00');
      const result = time.add('PT2H');
      expect(result.toString()).toBe('01:00:00');
    });
  });

  describe('PlainTime.prototype.with', () => {
    it('should replace fields', () => {
      const time = PlainTime.from('12:30:00');
      const result = time.with({ minute: 45 });
      expect(result.toString()).toBe('12:45:00');
    });
  });

  describe('PlainTime.prototype.valueOf', () => {
    it('should throw TypeError on valueOf', () => {
      const time = PlainTime.from('12:00');
      expect(() => time.valueOf()).toThrow();
    });
  });
});

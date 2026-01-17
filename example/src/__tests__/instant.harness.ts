import { describe, it, expect } from 'react-native-harness';
import { Instant, Duration } from 'react-native-temporal';

describe('Instant', () => {
  describe('Instant.from', () => {
    it('should create an Instant from a valid ISO string', () => {
      const instant = Instant.from('2020-01-01T00:00:00Z');
      expect(instant.toString()).toBe('2020-01-01T00:00:00Z');
    });

    it('should create an Instant from another Instant', () => {
      const instant1 = Instant.from('2020-01-01T00:00:00Z');
      const instant2 = Instant.from(instant1);
      expect(instant2.toString()).toBe('2020-01-01T00:00:00Z');
      expect(instant1.equals(instant2)).toBe(true);
    });

    it('should throw for invalid ISO strings', () => {
      expect(() => Instant.from('invalid-string')).toThrow();
      expect(() => Instant.from('2020-01-01')).toThrow(); // Missing time/offset
    });

    it('should throw for non-string/non-Instant arguments', () => {
      // @ts-ignore
      expect(() => Instant.from(123)).toThrow();
      // @ts-ignore
      expect(() => Instant.from({})).toThrow();
    });
  });

  describe('Instant.fromEpochMilliseconds', () => {
    it('should create an Instant from epoch milliseconds', () => {
      const instant = Instant.fromEpochMilliseconds(1577836800000); // 2020-01-01T00:00:00Z
      expect(instant.toString()).toBe('2020-01-01T00:00:00Z');
    });

    it('should handle negative epoch milliseconds', () => {
      const instant = Instant.fromEpochMilliseconds(-1000);
      expect(instant.toString()).toBe('1969-12-31T23:59:59Z');
    });
  });

  describe('Instant.fromEpochNanoseconds', () => {
    it('should create an Instant from epoch nanoseconds (BigInt)', () => {
      const ns = 1577836800000000000n;
      const instant = Instant.fromEpochNanoseconds(ns);
      expect(instant.toString()).toBe('2020-01-01T00:00:00Z');
    });
  });

  describe('Instant.compare', () => {
    it('should compare Instants correctly', () => {
      const one = Instant.from('2020-01-01T00:00:00Z');
      const two = Instant.from('2020-01-02T00:00:00Z');

      expect(Instant.compare(one, two)).toBe(-1);
      expect(Instant.compare(two, one)).toBe(1);
      expect(Instant.compare(one, one)).toBe(0);
    });

    it('should compare Instant and string correctly', () => {
      const one = Instant.from('2020-01-01T00:00:00Z');
      const stringTwo = '2020-01-02T00:00:00Z';

      expect(Instant.compare(one, stringTwo)).toBe(-1);
    });
  });

  describe('Instant.prototype.add', () => {
    it('should add a duration string', () => {
      const instant = Instant.from('2020-01-01T00:00:00Z');
      const result = instant.add('PT1H');
      expect(result.toString()).toBe('2020-01-01T01:00:00Z');
    });

    it('should add a Duration object', () => {
      const instant = Instant.from('2020-01-01T00:00:00Z');
      const duration = Duration.from({ hours: 1 });
      const result = instant.add(duration);
      expect(result.toString()).toBe('2020-01-01T01:00:00Z');
    });
  });

  describe('Instant.prototype.subtract', () => {
    it('should subtract a duration string', () => {
      const instant = Instant.from('2020-01-01T01:00:00Z');
      const result = instant.subtract('PT1H');
      expect(result.toString()).toBe('2020-01-01T00:00:00Z');
    });
  });

  describe('Instant.prototype.equals', () => {
    it('should return true for equal instants', () => {
      const i1 = Instant.from('2020-01-01T00:00:00Z');
      const i2 = Instant.from('2020-01-01T00:00:00Z');
      expect(i1.equals(i2)).toBe(true);
    });

    it('should return false for different instants', () => {
      const i1 = Instant.from('2020-01-01T00:00:00Z');
      const i2 = Instant.from('2020-01-01T00:00:01Z');
      expect(i1.equals(i2)).toBe(false);
    });
  });

  describe('Instant.prototype.valueOf', () => {
    it('should throw TypeError on valueOf', () => {
      const instant = Instant.from('2020-01-01T00:00:00Z');
      expect(() => instant.valueOf()).toThrow();
    });
  });
});

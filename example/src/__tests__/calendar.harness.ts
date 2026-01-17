import { describe, it, expect } from 'react-native-harness';
import { Calendar } from 'react-native-temporal';

describe('Calendar', () => {
  describe('Calendar.from', () => {
    it('should create from string identifier', () => {
      const cal = Calendar.from('iso8601');
      expect(cal.id).toBe('iso8601');
      expect(cal.toString()).toBe('iso8601');
    });

    it('should throw for invalid identifier', () => {
      // Assuming strict validation
      expect(() => Calendar.from('invalid-calendar')).toThrow();
    });
  });
});

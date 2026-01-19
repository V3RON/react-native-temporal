import { TimeZone } from '../types/TimeZone';
import { Instant } from '../types/Instant';

describe('TimeZone', () => {
  it('should create from string', () => {
    const tz = TimeZone.from('UTC');
    expect(tz.id).toBe('UTC');
  });

  it('should get offset', () => {
    const tz = TimeZone.from('UTC');
    const instant = Instant.from('2024-01-01T00:00:00Z');
    expect(tz.getOffsetStringFor(instant)).toBe('+00:00');
    expect(tz.getOffsetNanosecondsFor(instant)).toBe(0);
  });
});

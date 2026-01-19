import { ZonedDateTime } from '../types/ZonedDateTime';

describe('ZonedDateTime', () => {
  it('should parse ISO string', () => {
    // Note: Depends on native environment supporting Europe/Paris, usually standard in TZDB
    const zdt = ZonedDateTime.from('2024-01-15T10:30:00+01:00[Europe/Paris]');
    expect(zdt.year).toBe(2024);
    expect(zdt.timeZone.id).toBe('Europe/Paris');
    expect(zdt.offset).toBe('+01:00');
  });

  it('should add duration', () => {
    const zdt = ZonedDateTime.from('2024-01-15T10:00:00Z[UTC]');
    const added = zdt.add('PT1H');
    expect(added.hour).toBe(11);
  });

  it('should convert to PlainDateTime', () => {
    const zdt = ZonedDateTime.from('2024-01-15T10:30:00+01:00[Europe/Paris]');
    const pdt = zdt.toPlainDateTime();
    expect(pdt.toString()).toContain('2024-01-15T10:30:00');
  });
});

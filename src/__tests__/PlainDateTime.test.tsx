import { PlainDateTime } from '../types/PlainDateTime';

describe('PlainDateTime', () => {
  it('should parse ISO string', () => {
    const dt = PlainDateTime.from('2024-01-15T10:30:00');
    expect(dt.year).toBe(2024);
    expect(dt.month).toBe(1);
    expect(dt.day).toBe(15);
    expect(dt.hour).toBe(10);
    expect(dt.minute).toBe(30);
    expect(dt.second).toBe(0);
    expect(dt.toString()).toBe('2024-01-15T10:30:00');
  });

  it('should create from components', () => {
    const dt = PlainDateTime.from({
      year: 2024,
      month: 1,
      day: 15,
      hour: 10,
      minute: 30,
    });
    expect(dt.toString()).toBe('2024-01-15T10:30:00');
  });

  it('should add duration', () => {
    const dt = PlainDateTime.from('2024-01-15T10:00:00');
    const added = dt.add('P1DT1H'); // +1 day, +1 hour
    expect(added.toString()).toBe('2024-01-16T11:00:00');
  });

  it('should subtract duration', () => {
    const dt = PlainDateTime.from('2024-01-15T10:00:00');
    const subtracted = dt.subtract('P1DT1H');
    expect(subtracted.toString()).toBe('2024-01-14T09:00:00');
  });

  it('should compare plain date times', () => {
    const dt1 = PlainDateTime.from('2024-01-15T10:00:00');
    const dt2 = PlainDateTime.from('2024-01-15T11:00:00');
    expect(PlainDateTime.compare(dt1, dt2)).toBe(-1);
    expect(PlainDateTime.compare(dt2, dt1)).toBe(1);
    expect(PlainDateTime.compare(dt1, dt1)).toBe(0);
  });

  it('should update with with()', () => {
    const dt = PlainDateTime.from('2024-01-15T10:00:00');
    const updated = dt.with({ hour: 12, month: 2 });
    expect(updated.toString()).toBe('2024-02-15T12:00:00');
  });

  it('should convert to PlainDate and PlainTime', () => {
    const dt = PlainDateTime.from('2024-01-15T10:30:45');
    const date = dt.toPlainDate();
    const time = dt.toPlainTime();
    expect(date.toString()).toBe('2024-01-15');
    expect(time.toString()).toBe('10:30:45');
  });

  it('should compute difference using until()', () => {
    const dt1 = PlainDateTime.from('2024-01-15T10:00:00');
    const dt2 = PlainDateTime.from('2024-01-15T12:30:00');
    const diff = dt1.until(dt2);
    expect(diff.toString()).toBe('PT2H30M');
  });

  it('should compute difference using since()', () => {
    const dt1 = PlainDateTime.from('2024-01-15T12:30:00');
    const dt2 = PlainDateTime.from('2024-01-15T10:00:00');
    const diff = dt1.since(dt2);
    expect(diff.toString()).toBe('PT2H30M');
  });
});

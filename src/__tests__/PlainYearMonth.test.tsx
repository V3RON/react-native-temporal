import { PlainYearMonth } from '../types/PlainYearMonth';

describe('PlainYearMonth', () => {
  it('should parse ISO string', () => {
    const ym = PlainYearMonth.from('2024-03');
    expect(ym.year).toBe(2024);
    expect(ym.month).toBe(3);
    expect(ym.toString()).toBe('2024-03');
  });

  it('should create from components', () => {
    const ym = PlainYearMonth.from({ year: 2024, month: 3 });
    expect(ym.toString()).toBe('2024-03');
  });

  it('should add duration', () => {
    const ym = PlainYearMonth.from('2024-03');
    const added = ym.add('P1M');
    expect(added.toString()).toBe('2024-04');
  });

  it('should subtract duration', () => {
    const ym = PlainYearMonth.from('2024-03');
    const subtracted = ym.subtract('P1M');
    expect(subtracted.toString()).toBe('2024-02');
  });

  it('should compare', () => {
    const ym1 = PlainYearMonth.from('2024-03');
    const ym2 = PlainYearMonth.from('2024-04');
    expect(PlainYearMonth.compare(ym1, ym2)).toBe(-1);
    expect(PlainYearMonth.compare(ym2, ym1)).toBe(1);
    expect(PlainYearMonth.compare(ym1, ym1)).toBe(0);
  });

  it('should convert to PlainDate', () => {
    const ym = PlainYearMonth.from('2024-03');
    const date = ym.toPlainDate({ day: 14 });
    expect(date.toString()).toBe('2024-03-14');
  });
});

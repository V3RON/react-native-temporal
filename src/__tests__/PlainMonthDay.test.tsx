import { PlainMonthDay } from '../types/PlainMonthDay';

describe('PlainMonthDay', () => {
  it('should parse ISO string', () => {
    const md = PlainMonthDay.from('03-14');
    expect(md.monthCode).toBe('M03');
    expect(md.day).toBe(14);
    expect(md.toString()).toBe('03-14');
  });

  it('should create from components', () => {
    const md = PlainMonthDay.from({ month: 3, day: 14 });
    expect(md.toString()).toBe('03-14');
  });

  it('should convert to PlainDate', () => {
    const md = PlainMonthDay.from('03-14');
    const date = md.toPlainDate({ year: 2024 });
    expect(date.toString()).toBe('2024-03-14');
  });
});

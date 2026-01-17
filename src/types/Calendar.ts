import NativeTemporal from '../NativeTemporal';
import { wrapNativeCall } from '../utils';

/**
 * A Temporal.Calendar represents a calendar system.
 *
 * This implementation follows the TC39 Temporal proposal.
 * @see https://tc39.es/proposal-temporal/#sec-temporal-calendar-objects
 */
export class Calendar {
  readonly #id: string;

  private constructor(id: string) {
    this.#id = id;
  }

  /**
   * Creates a Calendar from a string identifier.
   */
  static from(item: string | Calendar): Calendar {
    if (item instanceof Calendar) {
      return item;
    }

    if (typeof item === 'string') {
      const id = wrapNativeCall(
        () => NativeTemporal.calendarFrom(item),
        `Invalid calendar identifier: ${item}`
      );
      return new Calendar(id);
    }

    throw new TypeError('Calendar.from requires a string or Calendar');
  }

  /**
   * Returns the calendar identifier.
   */
  get id(): string {
    return this.#id;
  }

  toString(): string {
    return this.#id;
  }

  toJSON(): string {
    return this.toString();
  }
}

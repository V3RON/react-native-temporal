import { Instant } from './Instant';

export const Now = {
  /**
   * Returns the current instant in the system time zone.
   *
   * @returns {Instant} The current instant.
   */
  instant: (): Instant => {
    return Instant.now();
  },
};

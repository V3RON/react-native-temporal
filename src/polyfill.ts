import { Duration } from './types/Duration';

declare global {
  var Temporal: {
    Duration: typeof Duration;
  };
}

globalThis.Temporal = globalThis.Temporal || {
  Duration,
};

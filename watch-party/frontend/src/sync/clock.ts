export interface ClockSample {
  offset: number;
  rtt: number;
}

export const SAMPLE_WINDOW = 16;

export function sampleFrom(t0: number, t1: number, t2: number, t3: number): ClockSample {
  return {
    offset: (t1 - t0 + (t2 - t3)) / 2,
    rtt: t3 - t0 - (t2 - t1),
  };
}

export function pushSample(samples: ClockSample[], sample: ClockSample, window = SAMPLE_WINDOW): ClockSample[] {
  return [...samples, sample].slice(-window);
}

export function bestSample(samples: ClockSample[]): ClockSample | null {
  return samples.reduce<ClockSample | null>((best, s) => (best === null || s.rtt < best.rtt ? s : best), null);
}

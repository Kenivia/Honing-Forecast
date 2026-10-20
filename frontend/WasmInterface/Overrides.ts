// Kept apart from PayloadBuilder so the profile can import them without an import cycle.
export enum NormalOverride {
  Full,
  Empty,
  Optimizer,
}
export enum AdvOverride {
  Full,
  Grace,
  Empty,
  Optimizer,
}
export interface StateOverride {
  juice: NormalOverride;
  book: NormalOverride;
}

// this kinda of have to work differently from stateoverride because weapon and highsest aren't independent
export interface SpecialOverride {
  optimizer: boolean;
  weapon_first: boolean;
  highest_first: boolean;
}

export interface AdvStateOverride {
  juice: AdvOverride;
  scroll: AdvOverride;
}
export interface OptimizerOverride {
  normal: StateOverride;
  special: SpecialOverride;
  advanced: AdvStateOverride;
}

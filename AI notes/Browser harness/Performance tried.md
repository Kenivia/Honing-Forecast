# Performance: tried

- Don't move `MaterialGraph`'s width read from `onMounted` into the `ResizeObserver`'s first observation to avoid the forced layout (`getBoundingClientRect`, about 87 ms sampled on roster→calc at 4x): the layout still happens, the observation then forces a second render and layout in the same frame, and native time rose by more than was saved (2026-11-17).

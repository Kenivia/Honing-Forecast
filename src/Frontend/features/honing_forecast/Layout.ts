export function recomputeLayout(mainRef, setMainScale, setControlsLeft) {
    const mainEl = mainRef.current
    if (!mainEl) return
    // Compute natural width using scrollWidth (not affected by CSS transform)
    const naturalWidth = mainEl.scrollWidth || 1268
    const vw = window.innerWidth
    const scale = Math.min(1, vw / naturalWidth)
    setMainScale(scale)

    // After scaling, use the element's bounding rect to position controls 30px to the right, clamped to viewport
    const rect = mainEl.getBoundingClientRect()
    // const controlsWidth = 200 // fixed width since we'll use transform scale
    const desiredLeft = rect.right + 30
    // const maxLeft = vw - controlsWidth + 100 // keep a small margin from the right edge
    setControlsLeft(Math.min(desiredLeft, Math.max(0, desiredLeft)))
}

export function createMouseEventFromTouch(touchEvent: React.TouchEvent, type: "mousedown" | "mousemove" | "mouseup"): React.MouseEvent {
    const touch = touchEvent.touches[0] || touchEvent.changedTouches[0]
    return {
        ...touchEvent,
        type,
        clientX: touch.clientX,
        clientY: touch.clientY,
        button: 0, // Left mouse button
        buttons: type === "mousedown" ? 1 : type === "mouseup" ? 0 : 1,
        preventDefault: touchEvent.preventDefault.bind(touchEvent),
        stopPropagation: touchEvent.stopPropagation.bind(touchEvent),
    } as unknown as React.MouseEvent
}

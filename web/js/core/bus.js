// One bus for the whole app. Components announce on it and never learn who listens.
//
// Not tree propagation: where a panel sits in the DOM is a layout decision, and moving one
// from the header into the dock must not change the information path.
//
// Names are a flat namespace, so anything that is not global carries its module: source:grow,
// model:grow. Singleton state sends no payload, listeners read the module and get the current
// value even when events coalesce. Data that exists more than once sends itself.

const bus = new EventTarget();

export const emit = (name, detail) => bus.dispatchEvent(new CustomEvent(name, { detail }));
export const on = (name, fn, signal) => bus.addEventListener(name, fn, { signal });

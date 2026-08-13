let transitionId = 0;

/** @param {number} ms */
function wait(ms) {
  return new Promise((resolve) => {
    setTimeout(resolve, ms);
  });
}

/** @param {string} name */
function readCssDuration(name) {
  const value = getComputedStyle(document.documentElement)
    .getPropertyValue(name)
    .trim();

  return Number.parseFloat(value) * (value.endsWith("ms") ? 1 : 1000);
}

export function waitForTransition() {
  return wait(readCssDuration("--transition-duration"));
}

function waitForReveal() {
  return wait(readCssDuration("--reveal-duration"));
}

/** @param {() => void | Promise<void>} render */
export async function transitionPage(render) {
  const id = ++transitionId;
  document.documentElement.dataset.transition = "";
  await waitForTransition();
  if (id !== transitionId) return;

  await render();
  if (id !== transitionId) return;

  requestAnimationFrame(() => {
    if (id === transitionId) delete document.documentElement.dataset.transition;
  });
}

export async function revealPage() {
  await waitForTransition();
  delete document.documentElement.dataset.loading;
  document.documentElement.dataset.revealing = "";
  await waitForReveal();
  delete document.documentElement.dataset.revealing;
}

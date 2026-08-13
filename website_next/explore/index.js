import { createChain } from "./chain/index.js";
import { transitionMs } from "./chain/transition.js";

export function createExplorePage() {
  const main = document.createElement("main");
  const chain = createChain({ onOpen: (block) => void showBlock(block) });
  let showingDetails = false;
  let transitioning = false;
  let blockDetails = /** @type {Awaited<ReturnType<typeof loadBlockDetails>> | null} */ (
    null
  );
  let blockDetailsPromise = /** @type {ReturnType<typeof loadBlockDetails> | null} */ (
    null
  );

  main.dataset.page = "explore";
  main.append(chain.element);

  async function loadBlockDetails() {
    const { createBlockDetails } = await import("./block/index.js");
    const details = createBlockDetails({ onBack: () => void showChain() });

    details.element.inert = true;
    main.append(details.element);

    return details;
  }

  async function getBlockDetails() {
    blockDetailsPromise ??= loadBlockDetails();
    blockDetails ??= await blockDetailsPromise;

    return blockDetails;
  }

  function syncChain() {
    chain.setActive(
      !main.hidden && !document.hidden && !showingDetails && !transitioning,
    );
  }

  /** @param {import("../modules/brk-client/index.js").BlockInfoV1} block */
  async function showBlock(block) {
    if (showingDetails || transitioning) return;

    transitioning = true;
    syncChain();

    try {
      const details = await getBlockDetails();

      details.update(block);
      details.element.scrollTop = 0;
      details.element.inert = false;
      chain.element.inert = true;
      void details.element.offsetWidth;
      showingDetails = true;
      main.dataset.details = "";
      await waitForSlide();
      details.activate();
    } catch (error) {
      console.error("explore block open:", error);
    } finally {
      transitioning = false;
      syncChain();
    }
  }

  async function showChain() {
    if (!blockDetails || !showingDetails || transitioning) return;

    transitioning = true;
    showingDetails = false;
    blockDetails.element.inert = true;
    delete main.dataset.details;
    syncChain();

    await waitForSlide();

    chain.element.inert = false;
    transitioning = false;
    syncChain();
  }

  function waitForSlide() {
    const milliseconds = transitionMs(chain.element, "transform");

    return new Promise((resolve) => window.setTimeout(resolve, milliseconds));
  }

  main.addEventListener("pageactive", syncChain);
  main.addEventListener("pageinactive", syncChain);
  document.addEventListener("visibilitychange", syncChain);
  requestIdleCallback(() => void getBlockDetails());

  return main;
}

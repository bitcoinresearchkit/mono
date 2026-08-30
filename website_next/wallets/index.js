import { bitview } from "../utils/client.js";
import { setStatus } from "./dom.js";
import { createEmpty } from "./empty/index.js";
import { getErrorMessage } from "./errors.js";
import { createAddForm } from "./add/index.js";
import { createAddSubmit } from "./add/submit.js";
import { createLayout } from "./layout/index.js";
import { redaction } from "./redaction/index.js";
import { scanStatus } from "./wallet/status.js";
import { createSelector } from "./selector/index.js";
import { createWalletSession } from "./start/session.js";
import {
  createWalletPanel,
  renderWalletPanel,
} from "./wallet/index.js";
import { createVault } from "./vault/index.js";

/**
 * @typedef {import("./scan/index.js").WalletScan} WalletScan
 * @typedef {import("./vault/index.js").StoredWallet} StoredWallet
 * @typedef {import("./vault/index.js").WalletRuntime} WalletRuntime
 */

export function createWalletsPage() {
  const {
    main,
    utilities,
    privacyButton,
    sessionButton,
    selector: selectorElement,
    walletList,
    content,
    addDialog,
  } = createLayout();
  const vault = createVault();
  const session = createWalletSession({
    vault,
    content,
    onChange: render,
  });
  const submitAdd = createAddSubmit({
    vault,
    dialog: addDialog,
    onAdded: render,
  });
  const selector = createSelector(walletList, {
    getSelectedId() {
      return vault.selectedId;
    },
    onSelect: select,
    onAdd() {
      openAdd();
    },
    onDelete() {
      deleteWallet(vault.selectedId);
    },
  });

  redaction.syncButton(privacyButton);

  /**
   * @param {string} walletId
   */
  function select(walletId) {
    vault.select(walletId);
    render();
  }

  /**
   * @param {string} walletId
   */
  function deleteWallet(walletId) {
    void vault.deleteWallet(walletId).then(() => {
      render();
    }, (error) => {
      console.error(error);
    });
  }

  function openAdd() {
    addDialog.replaceChildren(createAddForm({
      onCancel() {
        addDialog.close();
      },
      onSubmit(submit) {
        return submitAdd(submit);
      },
    }));
    addDialog.showModal();
  }

  privacyButton.addEventListener("click", () => {
    redaction.toggle(privacyButton);
  });

  sessionButton.addEventListener("click", () => {
    if (vault.isEphemeral()) {
      session.clearEphemeral();
      return;
    }

    session.lock();
  });

  /**
   * @param {StoredWallet} wallet
   * @param {WalletRuntime} runtime
   */
  function renderUnlocked(wallet, runtime) {
    const panel = createWalletPanel();

    content.replaceChildren(...panel.nodes);

    if (runtime.scan) {
      renderWalletData(runtime.scan, panel);
      setStatus(panel.status, "Ready");
      return;
    }

    scanStatus.setPending(panel.status);
    void runtime.load({
      client: bitview,
      onProgress(progress) {
        scanStatus.setProgress(panel.status, progress);
      },
    }).then((scan) => {
      if (!isCurrentPanel(wallet, runtime, panel)) return;

      renderWalletData(scan, panel);
      setStatus(panel.status, "Ready");
    }, (error) => {
      if (isCurrentPanel(wallet, runtime, panel)) {
        setStatus(panel.status, getErrorMessage(error));
      }
    });
  }

  /**
   * @param {StoredWallet} wallet
   * @param {WalletRuntime} runtime
   * @param {ReturnType<typeof createWalletPanel>} panel
   */
  function isCurrentPanel(wallet, runtime, panel) {
    return (
      vault.isCurrent(wallet, runtime) &&
      !vault.isLocked() &&
      vault.selectedId === wallet.id &&
      panel.results.isConnected
    );
  }

  /**
   * @param {WalletScan} scan
   * @param {ReturnType<typeof createWalletPanel>} panel
   */
  function renderWalletData(scan, panel) {
    renderWalletPanel(scan, panel, bitview);
  }

  function renderContent() {
    const needsSetup = vault.needsSetup();
    const locked = vault.isLocked();
    const ephemeral = vault.isEphemeral();
    const current = vault.current();
    const empty = !needsSetup && !locked && !current;

    utilities.hidden = locked || needsSetup || empty;
    selectorElement.hidden = locked || needsSetup || empty;
    sessionButton.hidden = locked || needsSetup || (!vault.hasPassword && !ephemeral);
    sessionButton.textContent = ephemeral ? "Clear" : "Lock";

    if (needsSetup) {
      session.renderStart("create");
      return;
    }

    if (locked) {
      session.renderStart("unlock");
      return;
    }

    if (!current) {
      content.replaceChildren(createEmpty({
        onAdd() {
          openAdd();
        },
        onClear: ephemeral ? session.clearEphemeral : undefined,
      }));
      return;
    }

    renderUnlocked(current.wallet, current.runtime);
  }

  function render() {
    if (vault.isLocked()) {
      selector.clear();
    } else {
      selector.render(vault.wallets);
    }
    renderContent();
  }

  render();

  return main;
}

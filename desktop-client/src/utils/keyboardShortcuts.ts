/**
 * Keyboard shortcuts for NozyWallet Desktop Client
 */

export const KeyboardShortcuts = {
  // Browser shortcuts
  NEW_TAB: 'Ctrl+T',
  CLOSE_TAB: 'Ctrl+W',
  REFRESH: 'Ctrl+R',
  FOCUS_ADDRESS_BAR: 'Ctrl+L',
  GO_BACK: 'Alt+Left',
  GO_FORWARD: 'Alt+Right',
  
  // General shortcuts
  CLOSE_DIALOG: 'Escape',
  SUBMIT_FORM: 'Enter',
  CANCEL: 'Escape',
} as const;

export type ShortcutAction = 
  | 'newTab'
  | 'closeTab'
  | 'refresh'
  | 'focusAddressBar'
  | 'goBack'
  | 'goForward'
  | 'closeDialog'
  | 'submitForm'
  | 'cancel';

export function useKeyboardShortcuts(
  actions: Partial<Record<ShortcutAction, () => void>>,
  enabled: boolean = true
) {
  const handleKeyDown = (e: KeyboardEvent) => {
    if (!enabled) return;

    const isCtrl = e.ctrlKey || e.metaKey; // Cmd on Mac
    const isAlt = e.altKey;
    const key = e.key.toLowerCase();

    // Browser shortcuts
    if (isCtrl && key === 't' && actions.newTab) {
      e.preventDefault();
      actions.newTab();
    }
    if (isCtrl && key === 'w' && actions.closeTab) {
      e.preventDefault();
      actions.closeTab();
    }
    if (isCtrl && key === 'r' && actions.refresh) {
      e.preventDefault();
      actions.refresh();
    }
    if (isCtrl && key === 'l' && actions.focusAddressBar) {
      e.preventDefault();
      actions.focusAddressBar();
    }
    if (isAlt && key === 'arrowleft' && actions.goBack) {
      e.preventDefault();
      actions.goBack();
    }
    if (isAlt && key === 'arrowright' && actions.goForward) {
      e.preventDefault();
      actions.goForward();
    }

    // General shortcuts
    if (key === 'escape' && actions.closeDialog) {
      e.preventDefault();
      actions.closeDialog();
    }
    if (key === 'enter' && e.target instanceof HTMLInputElement && actions.submitForm) {
      // Enter in input fields
      if (e.target.type !== 'textarea') {
        actions.submitForm();
      }
    }
  };

  return handleKeyDown;
}

const TOAST_EVENT = "pft:toast";

export function notify(message: string) {
  window.dispatchEvent(new CustomEvent(TOAST_EVENT, { detail: message }));
}

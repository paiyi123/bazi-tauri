import { onBackButtonPress } from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api/core";

function isTauriRuntime() {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

// Android／iOS 版才有相機掃碼。
export function canScanRecordQr() {
  return isTauriRuntime() && /Android|iPhone|iPad/i.test(navigator.userAgent);
}

export async function getRecordShareQrSvg(text: string): Promise<string> {
  if (!isTauriRuntime()) {
    throw new Error("瀏覽器除錯模式無法產生 QR Code，請直接複製分享碼");
  }
  return invoke<string>("record_share_qr_svg", { text });
}

type PermissionState = "granted" | "denied" | "prompt" | "prompt-with-rationale";

// 掃描時加在 <html> 上：網頁變透明，只留掃描遮罩，讓底下的相機畫面露出來。
const SCANNING_CLASS = "qr-scanning";

// 目前這次掃描的「結束掃描」函式；取消時呼叫。
let rejectPendingScan: (() => void) | undefined;

// 直接呼叫官方 barcode-scanner 外掛，免另外安裝 npm 套件。
// 用 windowed 模式：相機畫面在網頁「後面」，網頁上才能放自己的「取消」按鈕；
// 非 windowed 模式相機會蓋住整個畫面，沒有任何方法關掉。
export async function scanRecordQr(options: { onCameraOpen?: () => void } = {}): Promise<string> {
  let permission = await invoke<{ camera: PermissionState }>("plugin:barcode-scanner|check_permissions")
    .then((result) => result.camera)
    .catch(() => "prompt" as PermissionState);
  if (permission !== "granted") {
    permission = await invoke<{ camera: PermissionState }>("plugin:barcode-scanner|request_permissions").then(
      (result) => result.camera,
    );
  }
  if (permission !== "granted") {
    throw new Error("沒有相機權限，請到手機設定允許 BaZi 使用相機");
  }
  options.onCameraOpen?.();
  document.documentElement.classList.add(SCANNING_CLASS);
  // 掃描中按手機的「返回」鍵 = 取消掃描（有註冊監聽時，Tauri 不會再做預設的返回動作）。
  const backListener = await onBackButtonPress(() => {
    void cancelRecordQrScan();
  }).catch(() => undefined);
  try {
    // 外掛 2.4.x 的 cancel 會先清掉等待中的 scan 再 reject，scan 永遠不會結束，
    // 所以取消時由我們自己結束這次掃描（cancelRecordQrScan 會呼叫 rejectPendingScan）。
    const cancelled = new Promise<never>((_, reject) => {
      rejectPendingScan = () => reject(new Error("cancelled"));
    });
    const result = await Promise.race([
      invoke<{ content: string }>("plugin:barcode-scanner|scan", {
        windowed: true,
        formats: ["QR_CODE"],
      }),
      cancelled,
    ]);
    return result.content;
  } finally {
    rejectPendingScan = undefined;
    document.documentElement.classList.remove(SCANNING_CLASS);
    await backListener?.unregister().catch(() => undefined);
  }
}

export async function cancelRecordQrScan() {
  await invoke("plugin:barcode-scanner|cancel").catch(() => undefined);
  rejectPendingScan?.();
}

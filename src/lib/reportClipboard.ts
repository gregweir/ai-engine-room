export interface ReportClipboard {
  copyText(text: string): Promise<void>;
}

export async function createNativeReportClipboard(): Promise<ReportClipboard> {
  const { writeText } = await import("@tauri-apps/plugin-clipboard-manager");

  return {
    copyText(text: string) {
      return writeText(text);
    },
  };
}

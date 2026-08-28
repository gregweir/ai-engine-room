import { describe, expect, it, vi } from "vitest";
import { axe } from "vitest-axe";
import { render } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import type { ReportClipboard } from "../reportClipboard";
import ReportWorkspace from "./ReportWorkspace.svelte";

const preview =
  "AI Engine Room — Observation Report\n\nObservation\nMemory in use\nStatus: Not available\nValue: No value reported\n";

function fakeClipboard(copyText = vi.fn(async (_text: string) => {})) {
  return { clipboard: { copyText } satisfies ReportClipboard, copyText };
}

describe("Report clipboard workflow", () => {
  it("copies the exact visible preview once only after explicit activation", async () => {
    const { clipboard, copyText } = fakeClipboard();
    const view = render(ReportWorkspace, {
      props: { preview, clipboard, clipboardState: "native-ready" },
    });

    expect(copyText).not.toHaveBeenCalled();
    const button = view.getByRole("button", { name: "Copy report" });
    button.focus();
    await userEvent.setup().click(button);

    expect(copyText).toHaveBeenCalledTimes(1);
    expect(copyText).toHaveBeenCalledWith(preview);
    expect(copyText.mock.calls[0][0]).toBe(preview);
    expect(copyText.mock.calls[0][0]).toMatch(/\n$/);
    expect(view.getByRole("status")).toHaveTextContent(
      "Report copied to the system clipboard.",
    );
    expect(button).toHaveFocus();
  });

  it("disables duplicate activation while one write is pending", async () => {
    let resolveWrite!: () => void;
    const pendingWrite = new Promise<void>((resolve) => {
      resolveWrite = resolve;
    });
    const copyText = vi.fn(() => pendingWrite);
    const { clipboard } = fakeClipboard(copyText);
    const view = render(ReportWorkspace, {
      props: { preview, clipboard, clipboardState: "native-ready" },
    });
    const user = userEvent.setup();
    const button = view.getByRole("button", { name: "Copy report" });

    await user.click(button);
    expect(button).toBeDisabled();
    expect(button).toHaveAttribute("aria-busy", "true");
    await user.click(button);
    expect(copyText).toHaveBeenCalledTimes(1);

    resolveWrite();
    expect(await view.findByRole("status")).toBeVisible();
    expect(button).toBeEnabled();
  });

  it("uses a controlled failure and permits one explicit retry", async () => {
    const copyText = vi
      .fn<(text: string) => Promise<void>>()
      .mockRejectedValueOnce(new Error("private plugin stack /home/owner"))
      .mockResolvedValueOnce(undefined);
    const { clipboard } = fakeClipboard(copyText);
    const view = render(ReportWorkspace, {
      props: { preview, clipboard, clipboardState: "native-ready" },
    });
    const user = userEvent.setup();
    const button = view.getByRole("button", { name: "Copy report" });

    await user.click(button);
    expect(view.getByRole("alert")).toHaveTextContent(
      "Could not copy the report to the system clipboard.",
    );
    expect(view.container).not.toHaveTextContent("private plugin stack");
    expect(copyText).toHaveBeenCalledTimes(1);

    await user.click(button);
    expect(copyText).toHaveBeenCalledTimes(2);
    expect(await view.findByRole("status")).toHaveTextContent(
      "Report copied to the system clipboard.",
    );
  });

  it("performs no write for an empty preview", async () => {
    const { clipboard, copyText } = fakeClipboard();
    const view = render(ReportWorkspace, {
      props: {
        preview: "",
        clipboard,
        clipboardState: "native-ready",
      },
    });

    const button = view.getByRole("button", { name: "Copy report" });
    expect(button).toBeDisabled();
    await userEvent.setup().click(button);
    expect(copyText).not.toHaveBeenCalled();
    expect(view.getByText(/No report-safe preview is available/)).toBeVisible();
  });

  it("keeps browser/mock copy unavailable without an active clipboard control", () => {
    const { copyText } = fakeClipboard();
    const view = render(ReportWorkspace, {
      props: {
        preview,
        clipboard: null,
        clipboardState: "browser-unavailable",
      },
    });

    expect(view.queryByRole("button", { name: "Copy report" })).toBeNull();
    expect(
      view.getByText("Copy report is available in the native app."),
    ).toBeVisible();
    expect(copyText).not.toHaveBeenCalled();
  });

  it("distinguishes native initialization failure without a copy action", () => {
    const { copyText } = fakeClipboard();
    const view = render(ReportWorkspace, {
      props: {
        preview,
        clipboard: null,
        clipboardState: "native-init-failed",
      },
    });

    expect(
      view.getByText(
        "Clipboard export could not be initialized in this native session.",
      ),
    ).toBeVisible();
    expect(
      view.queryByText("Copy report is available in the native app."),
    ).toBeNull();
    expect(view.queryByRole("button", { name: "Copy report" })).toBeNull();
    expect(copyText).not.toHaveBeenCalled();
  });

  it("keeps the privacy disclosure visible and has no axe violations", async () => {
    const { clipboard } = fakeClipboard();
    const view = render(ReportWorkspace, {
      props: { preview, clipboard, clipboardState: "native-ready" },
    });

    expect(
      view.getByText(/Other applications.*may be able to read/s),
    ).toBeVisible();
    expect(view.getByText(/may remain.*until overwritten/s)).toBeVisible();
    expect(
      view.getByText(/does not automatically send or upload/s),
    ).toBeVisible();
    const results = await axe(view.container, {
      rules: { "color-contrast": { enabled: false } },
    });
    expect(results.violations).toEqual([]);
  });
});

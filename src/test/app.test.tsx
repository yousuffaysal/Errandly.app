import { describe, expect, it } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import Workspace from "../components/Workspace";
import { ErrorBoundary } from "../components/ErrorBoundary";
import { fakeBackend } from "./fakeBackend";

describe("Errandly smoke tests", () => {
  it("a new user is asked the onboarding questions, and answers are saved", async () => {
    const { calls } = fakeBackend({ profileCompleted: false });
    render(<Workspace />);
    expect(await screen.findByText("A little about you.")).toBeInTheDocument();

    const user = userEvent.setup();
    await user.type(screen.getByLabelText("Your answer"), "Yusuf{Enter}");
    expect(await screen.findByText(/Lovely to meet you, Yusuf/)).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Skip the introductions for now" }));
    await waitFor(() => expect(calls.some((c) => c.cmd === "save_profile")).toBe(true));
    const saved = calls.find((c) => c.cmd === "save_profile")!.args.profile as { name: string; completed: boolean };
    expect(saved).toMatchObject({ name: "Yusuf", completed: true });
  });

  it("sends a message and shows the reply", async () => {
    const { calls } = fakeBackend({ profileCompleted: true });
    render(<Workspace />);
    expect(await screen.findByText("What’s on your mind?")).toBeInTheDocument();

    const user = userEvent.setup();
    await user.type(await screen.findByLabelText("Message Errandly"), "hi{Enter}");
    expect(await screen.findByText("Hello! I’m Ario.")).toBeInTheDocument();
    expect(calls.find((c) => c.cmd === "send_message")?.args).toMatchObject({ conversationId: "c1", text: "hi" });
  });

  it("shows spreadsheet numbers exactly as calculated", async () => {
    fakeBackend({
      profileCompleted: true,
      reply: () => ({
        text: "Here’s sales.csv.",
        card: {
          type: "spreadsheet", file: "sales.csv", sheet: null, title: "Sales by product", insights: ["Coffee leads."], rejectedInsights: 0,
          analysis: {
            rows: 6, valueColumn: "Revenue", groupColumn: "Product", total: 7935.75, otherGroups: 0, skippedRows: 0, numeric: [],
            groups: [{ name: "Coffee", total: 4325.5, rows: 2, share: 54.51 }, { name: "Tea", total: 3300, rows: 3, share: 41.58 }],
          },
        },
      }),
    });
    render(<Workspace />);
    const user = userEvent.setup();
    await user.type(await screen.findByLabelText("Message Errandly"), "analyze sales{Enter}");
    expect(await screen.findByText("Sales by product")).toBeInTheDocument();
    expect(screen.getByText("7,935.75")).toBeInTheDocument();
    expect(screen.getByText("4,325.5")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Save as Excel report/ })).toBeInTheDocument();
  });

  it("opens Settings with ⌘, and shows storage", async () => {
    const { calls } = fakeBackend({ profileCompleted: true });
    render(<Workspace />);
    await screen.findByText("What’s on your mind?");
    const user = userEvent.setup();
    await user.keyboard("{Meta>},{/Meta}");
    expect(await screen.findByRole("dialog", { name: "Settings" })).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Storage" }));
    expect(await screen.findByText("2.5 GB")).toBeInTheDocument();
    expect(calls.some((c) => c.cmd === "storage_report")).toBe(true);
  });

  it("a crash shows the recovery screen instead of a blank window", async () => {
    fakeBackend();
    const Boom = () => {
      throw new Error("kaboom");
    };
    const quiet = console.error;
    console.error = () => {};
    render(
      <ErrorBoundary>
        <Boom />
      </ErrorBoundary>,
    );
    console.error = quiet;
    expect(screen.getByText("Something went wrong.")).toBeInTheDocument();
    expect(screen.getByText("kaboom")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Reload Errandly" })).toBeInTheDocument();
  });
});

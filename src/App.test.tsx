import { render, screen } from "@testing-library/react";
import App from "./App";

describe("App", () => {
  it("renders the main merge heading", () => {
    render(<App />);
    expect(
      screen.getByRole("heading", {
        name: /merge images and pdfs into one clean document/i
      })
    ).toBeInTheDocument();
  });

  it("starts with merge disabled until desktop files are selected", () => {
    render(<App />);
    expect(
      screen.getByRole("button", {
        name: /merge to pdf/i
      })
    ).toBeDisabled();
  });
});

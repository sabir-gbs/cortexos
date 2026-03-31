import { test, expect } from "@playwright/test";

test.describe("Frontend Dev Server", () => {
  test("serves the desktop shell index page", async ({ page }) => {
    const resp = await page.goto("/");
    expect(resp).toBeTruthy();
    expect(resp!.status()).toBe(200);

    // The page should load the React app and render the login screen
    const title = await page.title();
    expect(title).toBeTruthy();
  });
});

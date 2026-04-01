import { test, expect } from "@playwright/test";

test.describe("Desktop Shell", () => {
  test("shows login screen when not authenticated", async ({ page }) => {
    await page.goto("/");

    // The login card with aria-label "Login" should be visible
    const loginMain = page.getByRole("main", { name: "Login" });
    await expect(loginMain).toBeVisible({ timeout: 10_000 });

    // Title and subtitle
    await expect(page.getByText("CortexOS")).toBeVisible();
    await expect(page.getByText("Sign in to your account")).toBeVisible();

    // Form fields wrapped in <label> elements
    await expect(page.getByLabel("Username")).toBeVisible();
    await expect(page.getByLabel("Password")).toBeVisible();

    // Submit button
    await expect(page.getByRole("button", { name: "Sign in" })).toBeVisible();
  });

  test("login form submits and shows error when backend is unavailable", async ({
    page,
  }) => {
    await page.goto("/");

    // Fill in the login form
    await page.getByLabel("Username").fill("admin");
    await page.getByLabel("Password").fill("admin");

    // Submit
    await page.getByRole("button", { name: "Sign in" }).click();

    // Since the backend is not running, the form should show an error message
    // in the role="alert" region. If the backend were running, it would
    // transition to the desktop — but E2E without backend expects failure.
    const alert = page.getByRole("alert");
    await expect(alert).toBeVisible({ timeout: 10_000 });
    await expect(alert).toContainText(/failed|error|unauthorized|network|invalid credentials/i);
  });
});

// example.spec.ts
import { test, expect } from "@playwright/test";
require('dotenv').config()

const test_url = process.env.TEST_URL;

test("home page test", async ({ page }) => {
  await page.goto(test_url);

  // Expect a context "to contain" a substring.
  await expect(page).toHaveTitle(/React App/);

  await page.click("text=Todos");
  await expect(page.locator("h1").first()).toHaveText("Todos");
});

test.describe("to do page tests", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(test_url);
    await page.click("text=Todos");
  });

  test("add test", async ({ page }) => {
    await page.fill('[placeholder="New todo..."]', "todo test");

    await page.click("text=Add");
    // await expect(page.locator("text=Page 1 of 1").first()).toBeVisible();
    await expect(page.locator("[class='Form']").first()).toHaveText(
      /todo test/
    );
  });

  test("edit test", async ({ page }) => {
    let target = page
      .locator('[class="Form"]:has-text("todo test")')
      .locator("text=edit");

    await target.click();
    await expect(page.locator("input").first()).toHaveValue("todo test");

    await page.locator("input").fill("test todo");

    await page.click("text=Save");

    await expect(page.locator("[class='Form']").first()).toHaveText(
      /test todo/
    );
  });

  test("delete test", async ({ page }) => {
    // await page.waitForTimeout(100);

    let target = await page.locator('[class="Form"]:has-text("test todo")');

    expect(await target.count()).toBe(1);

    await target.locator("text=delete").click();

    await page.reload();

    target = await page.locator('[class="Form"]:has-text("test todo")');

    expect(await target.count()).toBe(0);
  });
});

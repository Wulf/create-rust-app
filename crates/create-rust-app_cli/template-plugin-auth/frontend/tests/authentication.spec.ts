import { test, expect } from "@playwright/test";
require("dotenv").config();

const test_url = process.env.TEST_URL;
const test_email = process.env.TEST_EMAIL;
const test_password = process.env.TEST_PASSWORD;

const get_ipts = async (page) => {
  let emailIpt = page
    .locator('[class="Form"]')
    .locator('div:has-text("Email")')
    .locator("input");
  let passwordIpt = page
    .locator('[class="Form"]')
    .locator('div:has-text("Password")')
    .locator("input");
  return [emailIpt, passwordIpt];
};

test("home page test", async ({ page }) => {
  await page.goto(test_url);

  // Expect a context "React App" a substring.
  await expect(page).toHaveTitle(/React App/);

  await page.click("text=Login/Register");
  await expect(page.locator("h1").first()).toHaveText("Login");
});

test.describe("login/register page tests", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(test_url);
    await page.click("text=Login/Register");
  });

  //register tests
  test("register test", async ({ page }) => {
    await page.click("text=/to register/");
    // await page.waitForTimeout(5000);
    await expect(page.locator("h1").first()).toHaveText("Registration");

    let [emailIpt, passwordIpt] = await get_ipts(page);

    await emailIpt.fill(test_email);
    await passwordIpt.fill(test_password);
    await page.click("button >> text=Register");
    await expect(page.locator("h1").first()).toHaveText("Activate");
  });

  //these tests require you had active your account
  //login tests
  test("login test", async ({ page }) => {
    await expect(page.locator("h1").first()).toHaveText("Login");

    let [emailIpt, passwordIpt] = await get_ipts(page);

    await emailIpt.fill(test_email);
    await passwordIpt.fill(test_password);
    await page.click("button >> text=Login");
    await expect(page.locator(".NavButton").last()).toHaveText("Logout");
    await page.click("text=Logout");
    await expect(page.locator(".NavButton").last()).toHaveText(
      "Login/Register"
    );
  });

  test.describe("tests after login", () => {
    test.beforeEach(async ({ page }) => {
      await expect(page.locator("h1").first()).toHaveText("Login");
      let [emailIpt, passwordIpt] = await get_ipts(page);
      await emailIpt.fill(test_email);
      await passwordIpt.fill(test_password);
      await page.click("button >> text=Login");
    });

    test.afterEach(async ({ page }) => {
      let target = await page.locator(".NavButton:has-text('Logout')");
      if (await target.count() !== 0) {
        await target.click();
      }
    });

    test("logout test", async ({ page }) => {
      await page.click("text=Logout");
      await expect(page.locator(".NavButton").last()).toHaveText(
        "Login/Register"
      );
    });

    test("refresh test", async ({ page }) => {
      await expect(page.locator(".NavButton").last()).toHaveText("Logout");
      await page.reload();
      await expect(page.locator(".NavButton").last()).toHaveText("Logout");
    });
  });
});

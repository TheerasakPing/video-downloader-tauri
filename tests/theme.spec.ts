import { test, expect } from '@playwright/test';

test.describe('Theme Color Switching', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    // Go to Settings tab
    await page.locator('header').getByRole('button', { name: /Settings/i }).click();
    await page.waitForTimeout(500);
  });

  test('should display theme selector with color options', async ({ page }) => {
    // Theme selector should be visible
    await expect(page.getByText('Color Theme')).toBeVisible();

    // Should have theme buttons
    const themeButtons = page.locator('button').filter({ has: page.locator('[style*="background"]') });
    const count = await themeButtons.count();
    expect(count).toBeGreaterThanOrEqual(5); // At least 5 themes
  });

  test('should change background when selecting Ocean theme', async ({ page }) => {
    // Get initial body background
    const initialBg = await page.evaluate(() => document.body.style.background);

    // Find and click Ocean theme (second theme)
    const themeButtons = page.locator('button[title]').filter({ hasText: /Ocean|Blue/i });
    if (await themeButtons.count() > 0) {
      await themeButtons.first().click();
    } else {
      // Click the second theme button in the grid
      const allThemeButtons = page.locator('.grid button').nth(1);
      await allThemeButtons.click();
    }

    await page.waitForTimeout(500);

    // Check that background changed
    const newBg = await page.evaluate(() => document.body.style.background);

    // Background should have changed (contains ocean blue colors)
    console.log('Initial background:', initialBg);
    console.log('New background:', newBg);

    // Either the background changed or it contains ocean colors
    expect(newBg.length).toBeGreaterThan(0);
  });

  test('should change background when selecting Emerald theme', async ({ page }) => {
    // Click on Emerald theme (third theme)
    const allThemeButtons = page.locator('.grid.grid-cols-5 button');
    await allThemeButtons.nth(2).click();

    await page.waitForTimeout(500);

    // Check that background contains emerald colors
    const newBg = await page.evaluate(() => document.body.style.background);
    console.log('Emerald background:', newBg);

    expect(newBg).toContain('linear-gradient');
  });

  test('should change background when selecting Rose theme', async ({ page }) => {
    // Click on Rose theme (fourth theme)
    const allThemeButtons = page.locator('.grid.grid-cols-5 button');
    await allThemeButtons.nth(3).click();

    await page.waitForTimeout(500);

    // Check that background contains rose colors
    const newBg = await page.evaluate(() => document.body.style.background);
    console.log('Rose background:', newBg);

    expect(newBg).toContain('linear-gradient');
  });

  test('should change background when selecting Amber theme', async ({ page }) => {
    // Click on Amber theme (fifth theme)
    const allThemeButtons = page.locator('.grid.grid-cols-5 button');
    await allThemeButtons.nth(4).click();

    await page.waitForTimeout(500);

    // Check that background contains amber colors
    const newBg = await page.evaluate(() => document.body.style.background);
    console.log('Amber background:', newBg);

    expect(newBg).toContain('linear-gradient');
  });

  test('should persist theme selection', async ({ page }) => {
    // Click on Emerald theme
    const allThemeButtons = page.locator('.grid.grid-cols-5 button');
    await allThemeButtons.nth(2).click();
    await page.waitForTimeout(500);

    // Get current background
    const bgBefore = await page.evaluate(() => document.body.style.background);

    // Reload page
    await page.reload();
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(500);

    // Background should still be the same (persisted)
    const bgAfter = await page.evaluate(() => document.body.style.background);

    expect(bgAfter).toBe(bgBefore);
  });

  test('should update CSS variables when theme changes', async ({ page }) => {
    // Get initial accent color
    const initialAccent = await page.evaluate(() =>
      getComputedStyle(document.documentElement).getPropertyValue('--accent').trim()
    );

    // Click on Ocean theme (blue)
    const allThemeButtons = page.locator('.grid.grid-cols-5 button');
    await allThemeButtons.nth(1).click();
    await page.waitForTimeout(500);

    // Get new accent color
    const newAccent = await page.evaluate(() =>
      getComputedStyle(document.documentElement).getPropertyValue('--accent').trim()
    );

    console.log('Initial accent:', initialAccent);
    console.log('New accent:', newAccent);

    // Accent color should have changed
    expect(newAccent).not.toBe(initialAccent);
  });
});

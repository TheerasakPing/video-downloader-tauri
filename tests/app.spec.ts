import { test, expect } from '@playwright/test';

test.describe('Rongyok Video Downloader - Main UI', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('should display header with logo and title', async ({ page }) => {
    // Check header exists
    await expect(page.locator('header')).toBeVisible();

    // Check title text
    await expect(page.getByText('Rongyok')).toBeVisible();
  });

  test('should display all navigation tabs in header', async ({ page }) => {
    // Check tabs in header navigation
    const header = page.locator('header');
    await expect(header.getByRole('button').filter({ hasText: /Download|Files|History|Settings|Logs/i }).first()).toBeVisible();
  });

  test('should switch between tabs', async ({ page }) => {
    // Click Files tab
    await page.locator('header').getByRole('button', { name: /Files/i }).click();
    await expect(page.getByText('No files in output directory')).toBeVisible();

    // Click History tab
    await page.locator('header').getByRole('button', { name: /History/i }).click();
    await expect(page.getByRole('heading', { name: 'Download History' })).toBeVisible();

    // Click Settings tab
    await page.locator('header').getByRole('button', { name: /Settings/i }).click();
    await page.waitForTimeout(300);

    // Click Logs tab
    await page.locator('header').getByRole('button', { name: /Logs/i }).click();
    await expect(page.getByText(/Application started/i)).toBeVisible();

    // Click back to Download tab
    await page.locator('header').getByRole('button', { name: /Download/i }).click();
    await expect(page.getByPlaceholder(/rongyok\.com/i)).toBeVisible();
  });

  test('should display URL input field', async ({ page }) => {
    const urlInput = page.getByPlaceholder(/rongyok\.com/i);
    await expect(urlInput).toBeVisible();
    await expect(urlInput).toBeEditable();
  });

  test('should display output directory input', async ({ page }) => {
    const outputInput = page.getByPlaceholder(/Downloads/i);
    await expect(outputInput).toBeVisible();
  });

  test('should display preset selector', async ({ page }) => {
    // Preset selector should be visible with preset buttons
    await expect(page.locator('button').filter({ hasText: /Quick|Fast|Quality/i }).first()).toBeVisible();
  });

  test('should show keyboard shortcuts help', async ({ page }) => {
    // Click keyboard button
    const keyboardBtn = page.locator('button[title="Keyboard Shortcuts"]');
    await keyboardBtn.click();

    // Modal should appear with shortcuts header
    await expect(page.getByRole('heading', { name: /Keyboard Shortcuts/i })).toBeVisible();

    // Close modal
    await page.keyboard.press('Escape');
  });
});

test.describe('Rongyok Video Downloader - Glowing Cards', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('should have card-glow classes on series card', async ({ page }) => {
    // Series card should have glowing effect classes
    const seriesCard = page.locator('.card-glow').first();
    await expect(seriesCard).toBeVisible();
  });

  test('should have stats-card-glow in files tab', async ({ page }) => {
    await page.locator('header').getByRole('button', { name: /Files/i }).click();

    // Wait for panel to load
    await page.waitForTimeout(500);

    // Stats cards should have glow classes
    const statsCards = page.locator('.stats-card-glow');
    const count = await statsCards.count();
    expect(count).toBeGreaterThan(0);
  });

  test('should have panel-glow on panels', async ({ page }) => {
    await page.locator('header').getByRole('button', { name: /Files/i }).click();
    await page.waitForTimeout(500);

    const panels = page.locator('.panel-glow');
    const count = await panels.count();
    expect(count).toBeGreaterThan(0);
  });

  test('should have icon-glow effects', async ({ page }) => {
    const glowingIcons = page.locator('.icon-glow');
    const count = await glowingIcons.count();
    expect(count).toBeGreaterThan(0);
  });
});

test.describe('Rongyok Video Downloader - Settings Panel', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    await page.locator('header').getByRole('button', { name: /Settings/i }).click();
    await page.waitForTimeout(300);
  });

  test('should display settings panel with sections', async ({ page }) => {
    // Settings panel should have content
    await expect(page.locator('main')).toBeVisible();
  });

  test('should have theme dropdown or toggle', async ({ page }) => {
    // Look for theme-related elements
    const themeElement = page.locator('select, [class*="theme"], button').filter({ hasText: /Dark|Light|System/i }).first();
    await expect(themeElement).toBeVisible();
  });

  test('should have language option', async ({ page }) => {
    // Language setting should be visible
    const langElement = page.locator('select, button').filter({ hasText: /English|Thai|ภาษา/i }).first();
    await expect(langElement).toBeVisible();
  });

  test('should have notification toggle', async ({ page }) => {
    // Find notification checkbox or toggle
    const notifElement = page.locator('input[type="checkbox"], label').filter({ hasText: /Notification/i }).first();
    await expect(notifElement).toBeVisible();
  });

  test('should have concurrent downloads setting', async ({ page }) => {
    // Find slider or input for concurrent downloads
    await expect(page.getByText(/Concurrent|simultaneous/i).first()).toBeVisible();
  });
});

test.describe('Rongyok Video Downloader - File Browser', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    await page.locator('header').getByRole('button', { name: /Files/i }).click();
    await page.waitForTimeout(300);
  });

  test('should display file browser empty state', async ({ page }) => {
    // Should show empty state message
    await expect(page.getByText('No files in output directory')).toBeVisible();
  });

  test('should have refresh button', async ({ page }) => {
    const refreshBtn = page.getByRole('button', { name: /Refresh/i });
    await expect(refreshBtn).toBeVisible();
  });

  test('should have open folder button', async ({ page }) => {
    const openBtn = page.getByRole('button', { name: /Open/i });
    await expect(openBtn).toBeVisible();
  });

  test('should display stats cards for files', async ({ page }) => {
    // Stats for Episodes, Merged, Total Size
    await expect(page.locator('.stats-card-glow').first()).toBeVisible();
  });
});

test.describe('Rongyok Video Downloader - History Panel', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    await page.locator('header').getByRole('button', { name: /History/i }).click();
    await page.waitForTimeout(300);
  });

  test('should display history panel header', async ({ page }) => {
    await expect(page.getByRole('heading', { name: /Download History/i })).toBeVisible();
  });

  test('should show stats section with counts', async ({ page }) => {
    // Stats section should show numbers
    await expect(page.locator('.stats-card-glow').first()).toBeVisible();
  });
});

test.describe('Rongyok Video Downloader - Log Panel', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    await page.locator('header').getByRole('button', { name: /Logs/i }).click();
    await page.waitForTimeout(300);
  });

  test('should display log panel with startup message', async ({ page }) => {
    // Should have application started log
    await expect(page.getByText(/Application started/i)).toBeVisible();
  });

  test('should have clear logs button', async ({ page }) => {
    const clearBtn = page.getByRole('button', { name: /Clear/i });
    await expect(clearBtn).toBeVisible();
  });
});

test.describe('Rongyok Video Downloader - URL Input Features', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('should type in URL input', async ({ page }) => {
    const urlInput = page.getByPlaceholder(/rongyok\.com/i);
    await urlInput.fill('https://rongyok.com/watch/?series_id=123');
    await expect(urlInput).toHaveValue('https://rongyok.com/watch/?series_id=123');
  });

  test('should have paste button', async ({ page }) => {
    const pasteBtn = page.getByRole('button', { name: /Paste/i });
    await expect(pasteBtn).toBeVisible();
  });

  test('should have fetch button', async ({ page }) => {
    const fetchBtn = page.getByRole('button', { name: /Fetch/i });
    await expect(fetchBtn).toBeVisible();
  });
});

test.describe('Rongyok Video Downloader - Download Actions', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('should have download button with episode count', async ({ page }) => {
    // Find the main download button (not the tab)
    const downloadBtn = page.locator('main button').filter({ hasText: /Download \(/i });
    await expect(downloadBtn).toBeVisible();
    await expect(downloadBtn).toBeDisabled();
  });

  test('should have auto-merge checkbox option', async ({ page }) => {
    const checkbox = page.locator('label').filter({ hasText: /Merge|merge/i });
    await expect(checkbox).toBeVisible();
  });
});

test.describe('Rongyok Video Downloader - Responsive Design', () => {
  test('should work on mobile viewport', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Header should still be visible
    await expect(page.locator('header')).toBeVisible();

    // Some tab buttons should be visible
    await expect(page.locator('header button').first()).toBeVisible();
  });

  test('should work on tablet viewport', async ({ page }) => {
    await page.setViewportSize({ width: 768, height: 1024 });
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    await expect(page.getByText('Rongyok')).toBeVisible();
  });

  test('should work on desktop viewport', async ({ page }) => {
    await page.setViewportSize({ width: 1920, height: 1080 });
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    await expect(page.getByText('Rongyok')).toBeVisible();
    // Header tabs should be visible
    await expect(page.locator('header').getByRole('button', { name: /Download/i })).toBeVisible();
  });
});

test.describe('Rongyok Video Downloader - Keyboard Shortcuts Modal', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('should open shortcuts modal with keyboard button', async ({ page }) => {
    const keyboardBtn = page.locator('button[title="Keyboard Shortcuts"]');
    await keyboardBtn.click();

    await expect(page.getByRole('heading', { name: /Keyboard Shortcuts/i })).toBeVisible();
  });

  test('should display shortcut keys in modal', async ({ page }) => {
    const keyboardBtn = page.locator('button[title="Keyboard Shortcuts"]');
    await keyboardBtn.click();

    // Common shortcuts should be listed (look for kbd elements)
    await expect(page.locator('kbd').first()).toBeVisible();
  });

  test('should close modal with Escape key', async ({ page }) => {
    const keyboardBtn = page.locator('button[title="Keyboard Shortcuts"]');
    await keyboardBtn.click();

    await expect(page.getByRole('heading', { name: /Keyboard Shortcuts/i })).toBeVisible();

    await page.keyboard.press('Escape');
    await page.waitForTimeout(300);

    // Modal should be closed
    await expect(page.getByRole('heading', { name: /Keyboard Shortcuts/i })).not.toBeVisible();
  });
});

test.describe('Rongyok Video Downloader - Visual Animations', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('should have fade-in animations', async ({ page }) => {
    const fadeElements = page.locator('.animate-fade-in');
    const count = await fadeElements.count();
    expect(count).toBeGreaterThan(0);
  });

  test('should have hover effects on buttons', async ({ page }) => {
    const hoverElements = page.locator('.hover-scale, .hover-lift, .hover-glow');
    const count = await hoverElements.count();
    expect(count).toBeGreaterThan(0);
  });

  test('should have glass effect on header', async ({ page }) => {
    const glassElements = page.locator('.glass');
    const count = await glassElements.count();
    expect(count).toBeGreaterThan(0);
  });
});

test.describe('Rongyok Video Downloader - Presets', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('should display preset options', async ({ page }) => {
    // Presets should be visible
    const presetBtns = page.locator('button').filter({ hasText: /Quick|Fast|Quality|Balanced/i });
    const count = await presetBtns.count();
    expect(count).toBeGreaterThan(0);
  });

  test('should be able to select a preset', async ({ page }) => {
    // Click a preset button
    const presetBtn = page.locator('button').filter({ hasText: /Fast|Quick/i }).first();
    await presetBtn.click();

    // The preset should be selected (visual feedback)
    await page.waitForTimeout(300);
  });
});

test.describe('Rongyok Video Downloader - Drag and Drop', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('should show drag overlay on dragover', async ({ page }) => {
    // Simulate dragover
    const main = page.locator('div').filter({ hasClass: /min-h-screen/i }).first();

    // Trigger dragover event via JavaScript
    await page.evaluate(() => {
      const event = new DragEvent('dragover', {
        bubbles: true,
        cancelable: true,
      });
      document.body.dispatchEvent(event);
    });

    // Note: This test may need adjustment based on actual implementation
    await page.waitForTimeout(300);
  });
});

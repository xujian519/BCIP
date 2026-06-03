import { test, expect } from '@playwright/test';

const mod = process.platform === 'darwin' ? 'Meta' : 'Control';

test.describe('全局快捷键', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/#/');
    await page.waitForLoadState('networkidle');
  });

  test('⌘⇧P 打开命令面板，Esc 关闭', async ({ page }) => {
    await page.keyboard.press(`${mod}+Shift+KeyP`);
    await expect(page.getByRole('dialog', { name: '命令面板' })).toBeVisible();
    await page.keyboard.press('Escape');
    await expect(page.getByRole('dialog', { name: '命令面板' })).toBeHidden();
  });

  test('⌘, 打开设置，Esc 关闭', async ({ page }) => {
    await page.keyboard.press(`${mod}+Comma`);
    await expect(page.getByRole('dialog', { name: '设置' })).toBeVisible();
    await page.keyboard.press('Escape');
    await expect(page.getByRole('dialog', { name: '设置' })).toBeHidden();
  });
});

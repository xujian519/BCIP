import { test, expect } from '@playwright/test';

test.describe('Mock 对话', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/#/');
    await page.waitForLoadState('networkidle');
  });

  test('发送「你好」后收到演示回复', async ({ page }) => {
    const composer = page.getByLabel('Agent 消息输入');
    await expect(composer).toBeEnabled();
    await composer.fill('你好');
    await composer.press('Enter');

    await expect(
      page.getByText('演示模式', { exact: false }).first(),
    ).toBeVisible({ timeout: 20_000 });
  });
});

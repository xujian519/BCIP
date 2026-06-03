import { test, expect } from '@playwright/test';

test.describe('主壳层（VITE_DEV_MOCK）', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/#/');
    await page.waitForLoadState('networkidle');
  });

  test('渲染标题栏、演示横幅与 Composer', async ({ page }) => {
    await expect(page.getByText('云熙智能体')).toBeVisible();
    await expect(page.getByLabel('Agent 消息输入')).toBeVisible();
    await expect(
      page.getByText('演示数据', { exact: false }),
    ).toBeVisible();
  });

  test('阶段 Tab 可切换到对比视图', async ({ page }) => {
    await page.getByRole('banner').getByRole('button', { name: '对比' }).click();
    await expect(page.getByText('权利要求对比')).toBeVisible();
  });
});

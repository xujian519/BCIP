import { test, expect } from '@playwright/test';

test.describe('主壳层（VITE_DEV_MOCK）', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/#/');
    await page.waitForLoadState('networkidle');
  });

  test('渲染标题栏、工作区欢迎页与 Composer', async ({ page }) => {
    await expect(page.getByText('云熙智能体')).toBeVisible();
    await expect(page.getByLabel('Agent 消息输入')).toBeVisible();
    await expect(
      page.getByText('打开一个工作区目录开始工作', { exact: false }),
    ).toBeVisible();
  });

  test('阶段 Tab 可切换激活状态', async ({ page }) => {
    const compareTab = page.getByRole('banner').getByRole('button', { name: '对比' });
    await compareTab.click();
    await expect(compareTab).toHaveClass(/bg-brand-500/);
  });
});

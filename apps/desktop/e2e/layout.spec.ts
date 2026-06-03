import { test, expect } from '@playwright/test';

test.describe('布局 shell', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/#/');
    await page.waitForLoadState('networkidle');
  });

  test('Activity Bar 可展开/收起文件面板', async ({ page }) => {
    const filesButton = page.getByRole('button', { name: '资源管理器' });
    await expect(page.getByText('资源管理器').first()).toBeVisible();

    await filesButton.click();
    await expect(page.getByText('资源管理器').first()).toBeHidden();

    await filesButton.click();
    await expect(page.getByText('资源管理器').first()).toBeVisible();
  });

  test('布局菜单可切换到文档模式', async ({ page }) => {
    await page.getByRole('button', { name: '布局设置' }).click();
    await page.getByRole('button', { name: '文档模式' }).click();

    await expect(page.getByLabel('Agent 消息输入')).toBeHidden();

    await page.getByRole('button', { name: '布局设置' }).click();
    await page.getByRole('button', { name: '三栏布局' }).click();
    await expect(page.getByLabel('Agent 消息输入')).toBeVisible();
  });

  test('搜索面板可打开', async ({ page }) => {
    await page.getByRole('button', { name: '搜索' }).click();
    await expect(page.getByPlaceholder('搜索文件名或内容…')).toBeVisible();
  });

  test('技能与外接渠道面板可打开', async ({ page }) => {
    await page.getByRole('button', { name: '技能' }).click();
    await expect(page.getByText('技能管理')).toBeVisible();

    await page.getByRole('button', { name: 'AI 助手' }).click();
    await expect(page.getByText('外接渠道')).toBeVisible();
    await expect(page.getByText('企业微信')).toBeVisible();
  });

  test('新建任务面板可打开', async ({ page }) => {
    await page.getByRole('button', { name: '新建任务' }).click();
    await expect(page.getByText('会话列表')).toBeVisible();
    await expect(page.getByRole('button', { name: '新建会话' })).toBeVisible();
  });

  test('⌘B 可切换侧栏展开/收起', async ({ page }) => {
    const mod = process.platform === 'darwin' ? 'Meta' : 'Control';
    await expect(page.getByText('资源管理器').first()).toBeVisible();
    await page.keyboard.press(`${mod}+KeyB`);
    await expect(page.getByText('资源管理器').first()).toBeHidden();
    await page.keyboard.press(`${mod}+KeyB`);
    await expect(page.getByText('资源管理器').first()).toBeVisible();
  });
});

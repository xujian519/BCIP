/**
 * 生成 C01–C12 走查截图（默认不纳入 npm run test:e2e）
 * 运行：npm run walkthrough:capture
 */
import { test, expect } from '@playwright/test';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const OUT_DIR = path.join(__dirname, '../docs/walkthrough-screenshots');

const mod = process.platform === 'darwin' ? 'Meta' : 'Control';

test.describe.configure({ mode: 'serial' });

test.describe('walkthrough capture @walkthrough', () => {
  test.use({ viewport: { width: 1440, height: 900 } });

  test.beforeEach(async ({ page }) => {
    await page.goto('/#/');
    await page.waitForLoadState('networkidle');
    await page.evaluate(() => {
      window.dispatchEvent(new CustomEvent('bcip-e2e-rich-thread'));
    });
    await page.waitForTimeout(400);
  });

  test('C01 线程列表', async ({ page }) => {
    const row = page.getByRole('button', { name: /专利检索会话/ }).first();
    await row.waitFor({ state: 'visible' });
    await row.screenshot({ path: path.join(OUT_DIR, 'C01.png') });
  });

  test('C02 用户消息', async ({ page }) => {
    const bubble = page.getByText('帮我检索一下', { exact: false }).first();
    await bubble.screenshot({ path: path.join(OUT_DIR, 'C02.png') });
  });

  test('C03 助手消息', async ({ page }) => {
    const block = page
      .getByText('好的，正在为您检索相关专利', { exact: false })
      .first();
    await block.screenshot({ path: path.join(OUT_DIR, 'C03.png') });
  });

  test('C04 工具调用', async ({ page }) => {
    const card = page.getByRole('button', { name: /search_patents/ }).first();
    await card.click();
    await page.waitForTimeout(300);
    await card.locator('xpath=ancestor::div[contains(@class,"rounded-md")]').first().screenshot({
      path: path.join(OUT_DIR, 'C04.png'),
    });
  });

  test('C05 命令审批', async ({ page }) => {
    await page.evaluate(() => {
      window.dispatchEvent(new CustomEvent('bcip-e2e-show-approval'));
    });
    await page.waitForTimeout(400);
    const panel = page.getByText('允许执行命令？').first();
    await panel.waitFor({ state: 'visible' });
    await panel.locator('xpath=ancestor::div[contains(@class,"flex-col")]').first().screenshot({
      path: path.join(OUT_DIR, 'C05.png'),
    });
    await page.getByRole('button', { name: '拒绝' }).first().click();
  });

  test('C06 MCP 设置', async ({ page }) => {
    await page.keyboard.press(`${mod}+Comma`);
    await page.getByRole('button', { name: 'MCP 服务器' }).click();
    await page.waitForTimeout(300);
    const panel = page.getByRole('dialog', { name: '设置' });
    await panel.screenshot({ path: path.join(OUT_DIR, 'C06.png') });
    await page.keyboard.press('Escape');
  });

  test('C07 设置导航', async ({ page }) => {
    await page.keyboard.press(`${mod}+Comma`);
    const nav = page.getByRole('dialog', { name: '设置' }).locator('nav').first();
    await nav.screenshot({ path: path.join(OUT_DIR, 'C07.png') });
    await page.keyboard.press('Escape');
  });

  test('C08 模型设置', async ({ page }) => {
    await page.keyboard.press(`${mod}+Comma`);
    await page.getByRole('button', { name: '模型与推理' }).click();
    await page.waitForTimeout(300);
    const panel = page.getByRole('dialog', { name: '设置' });
    await panel.screenshot({ path: path.join(OUT_DIR, 'C08.png') });
    await page.keyboard.press('Escape');
  });

  test('C09 用量顶栏', async ({ page }) => {
    const header = page.locator('header').filter({ hasText: '专利检索会话' }).last();
    await header.screenshot({ path: path.join(OUT_DIR, 'C09.png') });
  });

  test('C10 输入区', async ({ page }) => {
    const composer = page.getByLabel('Agent 消息输入');
    await composer.click();
    await composer.fill('/');
    await expect(page.getByText('/help')).toBeVisible();
    await page.locator('#bcip-composer-input').locator('..').locator('..').screenshot({
      path: path.join(OUT_DIR, 'C10.png'),
    });
  });

  test('C11 推理块', async ({ page }) => {
    const thinking = page.getByRole('button', { name: /Thinking/i }).first();
    await thinking.waitFor({ state: 'visible' });
    await thinking.screenshot({ path: path.join(OUT_DIR, 'C11.png') });
  });

  test('C12 断线页脚', async ({ page }) => {
    const footer = page.getByText('已断开').first();
    await footer.locator('xpath=ancestor::footer').screenshot({
      path: path.join(OUT_DIR, 'C12.png'),
    });
  });
});

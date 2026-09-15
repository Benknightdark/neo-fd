import { expect, test } from '@playwright/test';
import {
  type E2eFixture,
  getRecordedCommands,
  installTauriMock,
} from '../support/tauri-mock';

const fixture: E2eFixture = {
  files: {
    '/fixtures/report.txt': '第一行\n秘密資料\n第三行',
  },
  results: [
    {
      path: '/fixtures/report.txt',
      line_num: 2,
      pattern_name: '自定義',
      matched_text: '秘密資料',
    },
  ],
};

test.beforeEach(async ({ page }) => {
  await installTauriMock(page, fixture);
  await page.goto('/');
});

test('可從設定啟動掃描並在結果檢視中看到匹配內容', async ({ page }) => {
  await page.getByLabel('目標路徑').fill('/fixtures');
  await page.getByLabel('自定義 Regex').fill('秘密資料');
  await page.getByLabel('最大匹配筆數').fill('10');
  await page.getByRole('button', { name: '開始掃描' }).click();

  await expect(page.getByText('掃描已完成')).toBeVisible();
  await expect(page.locator('.count-badge')).toHaveText('1 筆');
  await page.getByTitle('平鋪列表檢視').click();
  await expect(
    page.getByText('/fixtures/report.txt', { exact: true }),
  ).toBeVisible();
  await expect(
    page.locator('main').getByText('秘密資料', { exact: true }),
  ).toBeVisible();

  const commands = await getRecordedCommands(page);
  const scanCommand = commands.find(
    (recordedCommand) => recordedCommand.command === 'scan_directory',
  );

  expect(scanCommand?.args).toMatchObject({
    path: '/fixtures',
    maxResults: 10,
    patterns: expect.arrayContaining([['自定義', '秘密資料', false]]),
  });
});

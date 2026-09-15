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
  await page.getByRole('button', { name: '開始掃描' }).click();
  await expect(page.getByText('掃描已完成')).toBeVisible();
  await page.getByTitle('平鋪列表檢視').click();
  await page.getByText('/fixtures/report.txt', { exact: true }).click();
  await expect(
    page.locator('.code-viewer').getByText('秘密資料', { exact: true }),
  ).toBeVisible();
});

test('可讀取、修改並刪除結果檔案', async ({ page }) => {
  await page.getByTitle('修改檔案敏感內容').click();
  const editor = page.locator('.code-editor-textarea');
  await expect(editor).toBeVisible();
  await editor.fill('第一行\n已修正內容\n第三行');
  await page.getByTitle('儲存檔案修改').click();

  await expect(page.getByText(/檔案修改已儲存/)).toBeVisible();
  const commandsAfterSave = await getRecordedCommands(page);
  expect(commandsAfterSave).toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        command: 'read_file_content',
        args: { path: '/fixtures/report.txt' },
      }),
      expect.objectContaining({
        command: 'write_file_content',
        args: {
          path: '/fixtures/report.txt',
          content: '第一行\n已修正內容\n第三行',
        },
      }),
    ]),
  );

  page.once('dialog', (dialog) => dialog.accept());
  await page.getByTitle('自硬碟永久刪除此檔案').click();
  await expect(page.getByText('檔案已成功永久刪除。')).toBeVisible();
  await expect(page.locator('.count-badge')).toHaveText('0 筆');

  const commandsAfterDelete = await getRecordedCommands(page);
  expect(commandsAfterDelete).toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        command: 'delete_file',
        args: { path: '/fixtures/report.txt' },
      }),
    ]),
  );
});

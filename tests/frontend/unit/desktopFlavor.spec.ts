import { existsSync, readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

type TauriConfig = {
  productName: string;
  identifier: string;
  build?: {
    beforeBuildCommand?: string;
  };
  app?: {
    windows?: Array<{
      title: string;
      width: number;
      height: number;
    }>;
  };
  bundle: {
    icon: string[];
  };
};

const projectRoot = resolve(
  dirname(fileURLToPath(import.meta.url)),
  '../../..',
);
const tauriDirectory = resolve(projectRoot, 'neo-fd-desktop/src-tauri');

function readTauriConfig(fileName: string): TauriConfig {
  const filePath = resolve(tauriDirectory, fileName);
  return JSON.parse(readFileSync(filePath, 'utf8')) as TauriConfig;
}

describe('Tauri desktop flavors', () => {
  it('保留正式身份並定義隔離的開發身份', () => {
    const production = readTauriConfig('tauri.conf.json');
    const development = readTauriConfig('tauri.dev.conf.json');

    expect(production.productName).toBe('neo-fd-desktop');
    expect(production.identifier).toBe('com.ben.neo-fd-desktop');
    expect(development.productName).toBe('neo-fd-desktop-dev');
    expect(development.identifier).toBe('com.ben.neo-fd-desktop.dev');
    expect(development.build?.beforeBuildCommand).toBe('npm run build:dev');
    expect(development.app?.windows).toEqual([
      {
        title: 'neo-fd-desktop 開發版',
        width: 800,
        height: 600,
      },
    ]);
    expect(development.bundle.icon).toEqual([
      'icons/dev/32x32.png',
      'icons/dev/128x128.png',
      'icons/dev/128x128@2x.png',
      'icons/dev/icon.icns',
      'icons/dev/icon.ico',
    ]);
  });

  it('提供與正式版不同且可用的開發圖示', () => {
    const development = readTauriConfig('tauri.dev.conf.json');
    const productionIcon = readFileSync(
      resolve(tauriDirectory, 'icons/icon.png'),
    );
    const developmentIcon = readFileSync(
      resolve(tauriDirectory, 'icons/dev/icon.png'),
    );

    expect(developmentIcon).not.toEqual(productionIcon);
    for (const iconPath of development.bundle.icon) {
      expect(existsSync(resolve(tauriDirectory, iconPath))).toBe(true);
    }
  });
});

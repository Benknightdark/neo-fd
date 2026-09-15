import packageInfo from '../../package.json';

export const appVersion = packageInfo.version;

export function getAppEnvironmentLabel(mode: string): '開發' | '正式' {
  return mode === 'development' ? '開發' : '正式';
}

export const appEnvironmentLabel = getAppEnvironmentLabel(import.meta.env.MODE);
export const appDisplayVersion = `${appEnvironmentLabel} - v${appVersion}`;

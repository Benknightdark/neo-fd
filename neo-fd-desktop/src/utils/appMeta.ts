import packageInfo from '../../package.json';

export const appVersion = packageInfo.version;
export const appEnvironmentLabel = import.meta.env.DEV ? '開發' : '正式';
export const appDisplayVersion = `${appEnvironmentLabel} - v${appVersion}`;

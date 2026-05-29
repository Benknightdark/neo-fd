# Frontend Development Guidelines (Scanner Desktop)

## 1. Core Architecture
- **Framework**: Vue 3 with Composition API (`<script setup>`). Do NOT use the Options API.
- **Tauri Integration**: This is a Tauri 2 desktop application. The frontend communicates with the Rust backend via IPC (Inter-Process Communication).

## 2. State Management
- **Mandatory Tool**: Use **Pinia** for all global state management.
- **Convention**: Avoid using raw Vue `ref` or `reactive` for state that is shared across multiple non-parent/child components. Do NOT use Vuex.

## 3. Styling
- **Mandatory Tool**: Use **TailwindCSS** for all styling.
- **Convention**: Avoid `<style scoped>` blocks with raw CSS/SCSS unless absolutely necessary for complex animations or overrides that Tailwind cannot handle. Prefer utility classes directly in the template.

## 4. Tauri IPC Communication
- **Encapsulation**: Do NOT call `@tauri-apps/api/core` `invoke` directly from components.
- **Convention**: Use the `safeInvoke` wrapper located in `src/api/ipc.ts`.
- **API Registry**: All command calls should be registered in `src/api/ipc.ts` (e.g., `scannerApi.startScan`) to provide type safety and centralized management.
- **Error Handling**: 
    - All Rust commands MUST return a `Result<T, E>`.
    - The `safeInvoke` wrapper automatically logs IPC errors and re-throws them as standard `Error` objects.
    - Components should use `try...catch` for UI-specific error feedback (e.g., toasts).

### Example Usage:
```typescript
import { scannerApi } from '@/api/ipc';

try {
  await scannerApi.startScan(path, patterns);
} catch (err) {
  // Show error in UI
  showToast((err as Error).message, 'error');
}
```

## 5. Tooling & Linting
- **Linter/Formatter**: We use **Biome**. Always run `npm run lint` and `npm run format` before committing.
- **Testing**: We use **Vitest** for unit testing and **Playwright** for E2E testing. New features should include corresponding test coverage.

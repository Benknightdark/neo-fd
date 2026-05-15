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
- **Convention**: Create typed wrapper functions in a dedicated `src/api` or `src/commands` module. 
- **Error Handling**: All Rust commands must return a `Result<T, E>`. The frontend wrappers must properly catch these errors and integrate them into the UI (e.g., using a toast notification system or inline error states) rather than silently failing.

## 5. Tooling & Linting
- **Linter/Formatter**: We use **Biome**. Always run `npm run lint` and `npm run format` before committing.
- **Testing**: We use **Vitest** for unit testing and **Playwright** for E2E testing. New features should include corresponding test coverage.

# Repository Guidelines

## Project Structure & Module Organization
- Core app code lives in `src/` using Next.js App Router.
- Route pages are under `src/app/` (for example `src/app/marketplace/page.tsx`, `src/app/console/*`).
- Reusable UI is grouped by domain in `src/components/` (`home/`, `marketplace/`, `product/`, `console/`, `auth/`, `layout/`).
- Shared logic and state live in `src/lib/` and `src/store/`.
- Type definitions belong in `src/types/`.
- Root config files: `next.config.js`, `tailwind.config.ts`, `tsconfig.json`, `.eslintrc.json`.

## Build, Test, and Development Commands
- `pnpm dev` (or `npm run dev`): starts local dev server on port `3001`.
- `pnpm build`: creates production build.
- `pnpm start`: runs the production server.
- `pnpm lint`: runs Next.js ESLint checks.
- `pnpm type-check`: runs TypeScript checks without emitting files.

Use `lint` + `type-check` before opening a PR.

## Coding Style & Naming Conventions
- Language: TypeScript + React function components.
- Indentation: 2 spaces; keep existing semicolon/quote style consistent with surrounding code.
- Components/files: `PascalCase` for component files (for example `ProductCard.tsx`).
- Hooks/stores/utilities: `camelCase` (for example `useAuthStore.ts`, `api-client.ts`).
- Prefer Tailwind utility classes over inline styles.
- Keep components focused and strongly typed; avoid `any` unless justified.

## Testing Guidelines
- No dedicated automated test framework is configured yet.
- Minimum quality gate for every change: `pnpm lint` and `pnpm type-check` must pass.
- For UI changes, verify affected routes manually (for example `/`, `/marketplace`, `/products/[id]`, and related console pages).
- If adding tests later, colocate them near features (for example `src/components/foo/foo.test.tsx`).

## Commit & Pull Request Guidelines
- Follow concise, imperative commit messages. Existing history uses patterns like:
  - `test: complete TEST-028 canonical contracts gate`
  - `Clean old frontend codes`
- Recommended format: `<scope>: <action summary>` (example: `marketplace: refine filter URL sync`).
- PRs should include:
  - clear purpose and impacted routes/modules,
  - linked issue/task ID when available,
  - screenshots or short recordings for UI changes,
  - notes on verification steps performed (`lint`, `type-check`, manual paths).

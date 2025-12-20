---
status: pending
created: 2025-12-20
completed: null
dependencies: []
---

# Spec: React Router configuration

## Description

Install React Router and create the routes configuration for URL-based navigation. This replaces the current `useState`-based page switching in App.tsx with proper client-side routing, enabling deep linking and browser-like navigation in the desktop app.

## Acceptance Criteria

- [ ] `react-router-dom` package installed and in package.json
- [ ] `src/routes.tsx` created with route definitions
- [ ] Routes defined for all existing pages:
  - `/` → Dashboard (index route)
  - `/commands` → Commands page
  - `/recordings` → Recordings page
  - `/settings` → Settings page (with potential nested routes)
- [ ] `createBrowserRouter` or `createHashRouter` used (hash for Tauri compatibility)
- [ ] Layout route wraps pages with AppShell (Header, Sidebar, Footer)
- [ ] 404/catch-all route handled gracefully
- [ ] Routes are typed (element types match page components)
- [ ] No `onNavigate` prop passing pattern (use `useNavigate` instead)

## Test Cases

- [ ] Router initializes without errors
- [ ] `/` renders Dashboard component
- [ ] `/commands` renders Commands component
- [ ] `/recordings` renders Recordings component
- [ ] `/settings` renders Settings component
- [ ] Unknown route shows fallback or redirects to `/`

## Dependencies

None - this is a foundational spec.

## Preconditions

- Existing page components: Dashboard, Commands, Recordings, Settings
- Existing layout components: AppShell, Header, Sidebar, Footer

## Implementation Notes

```typescript
// src/routes.tsx
import { createHashRouter, Outlet } from 'react-router-dom';
import { AppShell } from './components/layout/AppShell';
import { Dashboard } from './pages/Dashboard';
import { Commands } from './pages/Commands';
import { Recordings } from './pages/Recordings';
import { Settings } from './pages/Settings';

// Layout wrapper that provides AppShell
function RootLayout() {
  return (
    <AppShell>
      <Outlet />
    </AppShell>
  );
}

export const router = createHashRouter([
  {
    path: '/',
    element: <RootLayout />,
    children: [
      { index: true, element: <Dashboard /> },
      { path: 'commands', element: <Commands /> },
      { path: 'recordings', element: <Recordings /> },
      { path: 'settings', element: <Settings /> },
    ],
  },
]);
```

**Note:** Using `createHashRouter` for Tauri compatibility - file:// protocol doesn't support browser history API well.

**Page component updates needed:**
- Remove `onNavigate` prop from page components
- Replace `onNavigate('dashboard')` calls with `useNavigate()` hook
- Update Sidebar to use `<Link>` or `<NavLink>` components

## Related Specs

- `app-providers-wiring` - wraps app with RouterProvider
- All page components will need minor updates to use `useNavigate`

## Integration Points

- Production call site: `src/App.tsx` (RouterProvider)
- Connects to: All page components, AppShell layout

## Integration Test

- Test location: `src/__tests__/routing.test.tsx`
- Verification: [ ] Navigation between routes works correctly

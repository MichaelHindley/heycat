import { createHashRouter, Navigate, Outlet } from "react-router-dom";
import { Dashboard, Commands, Recordings, Settings } from "./pages";

/**
 * Root layout component that renders child routes via Outlet.
 *
 * Note: AppShell wrapping is handled by the app-providers-wiring spec.
 * This layout exists to provide a common wrapper for all page routes
 * and can be extended with layout-specific logic.
 */
function RootLayout() {
  return <Outlet />;
}

/**
 * Application router using hash-based routing for Tauri compatibility.
 *
 * Routes:
 * - `/` (index) - Dashboard
 * - `/commands` - Commands page
 * - `/recordings` - Recordings page
 * - `/settings` - Settings page
 * - `*` - Redirects to Dashboard (404 fallback)
 *
 * Note: Using createHashRouter instead of createBrowserRouter because
 * Tauri's file:// protocol doesn't support the browser history API well.
 */
export const router = createHashRouter([
  {
    path: "/",
    element: <RootLayout />,
    children: [
      { index: true, element: <Dashboard /> },
      { path: "commands", element: <Commands /> },
      { path: "recordings", element: <Recordings /> },
      { path: "settings", element: <Settings /> },
      // Catch-all route redirects to dashboard
      { path: "*", element: <Navigate to="/" replace /> },
    ],
  },
]);

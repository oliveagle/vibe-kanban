import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App.tsx';
import './styles/index.css';
import { ClickToComponent } from 'click-to-react-component';
import { VibeKanbanWebCompanion } from 'vibe-kanban-web-companion';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import i18n from './i18n';
import posthog from 'posthog-js';
import { PostHogProvider } from 'posthog-js/react';
import './types/modals';

if (
  import.meta.env.VITE_POSTHOG_API_KEY &&
  import.meta.env.VITE_POSTHOG_API_ENDPOINT
) {
  posthog.init(import.meta.env.VITE_POSTHOG_API_KEY, {
    api_host: import.meta.env.VITE_POSTHOG_API_ENDPOINT,
    capture_pageview: false,
    capture_pageleave: true,
    capture_performance: true,
    autocapture: false,
    opt_out_capturing_by_default: true,
  });
} else {
  console.warn(
    'PostHog API key or endpoint not set. Analytics will be disabled.'
  );
}

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 60 * 5,
      refetchOnWindowFocus: false,
    },
  },
});

class ErrorBoundary extends React.Component<
  { children: React.ReactNode },
  { hasError: boolean; error?: Error }
> {
  constructor(props: { children: React.ReactNode }) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError(error: Error) {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('Error caught by boundary:', error, errorInfo);
  }

  render() {
    if (this.state.hasError) {
      return (
        <div style={{ padding: 20, fontFamily: 'sans-serif' }}>
          <h1>{i18n.t('common:states.error')}</h1>
          <pre style={{ whiteSpace: 'pre-wrap', wordBreak: 'break-word' }}>
            {this.state.error?.toString?.() || 'Unknown error'}
          </pre>
        </div>
      );
    }

    return this.props.children;
  }
}

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <QueryClientProvider client={queryClient}>
      <PostHogProvider client={posthog}>
        <ErrorBoundary>
          <ClickToComponent />
          <VibeKanbanWebCompanion />
          <App />
        </ErrorBoundary>
      </PostHogProvider>
    </QueryClientProvider>
  </React.StrictMode>
);

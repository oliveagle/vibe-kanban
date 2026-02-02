import { Outlet, useSearchParams } from 'react-router-dom';
import { DevBanner } from '@/components/DevBanner';
import { Navbar } from '@/components/layout/Navbar';
import { VersionFooter } from '@/components/VersionFooter';

export function NormalLayout() {
  const [searchParams] = useSearchParams();
  const view = searchParams.get('view');
  const shouldHideNavbar = view === 'preview' || view === 'diffs';

  return (
    <div className="flex flex-col h-full min-h-0">
      <DevBanner />
      {!shouldHideNavbar && <Navbar />}
      <div className="flex-1 min-h-0 overflow-hidden">
        <Outlet />
      </div>
      <VersionFooter />
    </div>
  );
}

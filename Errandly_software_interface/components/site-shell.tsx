'use client';
import {usePathname} from 'next/navigation';
import {Header, Footer} from './site';
export default function SiteShell({children}:{children:React.ReactNode}) {
  const pathname=usePathname();
  return (pathname.startsWith('/workspace') || pathname.startsWith('/admin')) ? <>{children}</> : <><Header/>{children}<Footer/></>;
}

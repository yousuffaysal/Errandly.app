import type { Metadata } from 'next';
import './globals.css';
import SiteShell from '../components/site-shell';
export const metadata: Metadata = {title:{default:'Errandly — Your work. Handled.',template:'%s · Errandly'},description:'Your personal AI for everyday work. Organize files, understand documents, and create reports on your Mac. Local by default. Free beta for Apple Silicon Macs.',icons:{icon:'/favicon.svg'}};
export default function Layout({children}:{children:React.ReactNode}){return <html lang="en"><body><a className="skip" href="#main">Skip to content</a><SiteShell>{children}</SiteShell></body></html>}

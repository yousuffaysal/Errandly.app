import type { Metadata } from 'next';
import AdminDashboard from '../../components/admin-dashboard';
import './admin.css';

// A private page: it shows nothing unless an admin signs in, and search
// engines are told not to list it.
export const metadata: Metadata = { title: 'Admin', robots: { index: false, follow: false } };

export default function AdminPage() {
  return <AdminDashboard />;
}

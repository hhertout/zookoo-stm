import type { ReactNode } from 'react';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';

export default function Footer(): ReactNode {
  const { siteConfig } = useDocusaurusContext();
  const year = new Date().getFullYear();

  return (
    <footer
      className="px-6 py-8"
      style={{ backgroundColor: '#0a0a0a', borderTop: '1px solid #1e1e1e' }}
    >
      <div className="max-w-5xl mx-auto flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
        <div className="flex items-center gap-3">
          <img src="/img/zookoo_tight.png" alt="Zookoo" className="w-6 h-6" />
          <span className="text-sm font-semibold" style={{ color: '#ededed' }}>
            {siteConfig.title}
          </span>
          <span className="text-sm" style={{ color: '#555555' }}>
            · {siteConfig.tagline}
          </span>
        </div>
        <div className="flex items-center gap-6">
          {[
            { label: 'Docs', to: '/docs/intro' },
            { label: 'GitHub', to: 'https://github.com/hhertout/zookoo-stm' },
            {
              label: 'MIT License',
              to: 'https://github.com/hhertout/zookoo-stm/blob/main/LICENSE',
            },
          ].map(({ label, to }) => (
            <Link
              key={label}
              to={to}
              className="text-sm no-underline transition-colors duration-150"
              style={{ color: '#888888' }}
            >
              {label}
            </Link>
          ))}
          <span className="text-sm" style={{ color: '#555555' }}>
            © {year}
          </span>
        </div>
      </div>
    </footer>
  );
}

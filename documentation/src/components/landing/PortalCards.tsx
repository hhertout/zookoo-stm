import type { ReactNode } from 'react';
import Link from '@docusaurus/Link';

const cards = [
  {
    title: 'Quick Start',
    description: 'Get Zookoo running in under 5 minutes with Docker.',
    href: '/docs/category/quick-start',
  },
  {
    title: 'Configuration',
    description: 'Learn how to configure probes, exporters, and discovery.',
    href: '/docs/category/configuration',
  },
  {
    title: 'Metrics Reference',
    description: 'Explore all metrics exported by each probe type.',
    href: '/docs/category/exported-metrics',
  },
];

export default function PortalCards(): ReactNode {
  return (
    <section
      className="py-24 px-6"
      style={{ backgroundColor: '#111111', borderTop: '1px solid #1e1e1e' }}
    >
      <div className="max-w-5xl mx-auto">
        <h2 className="text-2xl font-semibold mb-10" style={{ color: '#ededed' }}>
          Get started
        </h2>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-5">
          {cards.map(({ title, description, href }) => (
            <Link key={title} to={href} className="portal-card">
              <h3
                className="text-base font-semibold mb-2"
                style={{ color: '#ededed' }}
              >
                {title}
              </h3>
              <p
                className="text-sm leading-relaxed flex-1"
                style={{ color: '#888888' }}
              >
                {description}
              </p>
              <span
                className="mt-4 text-sm font-medium"
                style={{ color: '#f97316' }}
              >
                Explore →
              </span>
            </Link>
          ))}
        </div>
      </div>
    </section>
  );
}

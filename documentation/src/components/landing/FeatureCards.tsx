import type { ReactNode } from 'react';

const features = [
  {
    icon: '/img/rust.svg',
    title: 'Rust-powered',
    description:
      'Blazing fast, low footprint. Built for performance-critical environments.',
  },
  {
    icon: '/img/otel.svg',
    title: 'OpenTelemetry native',
    description: 'Export metrics anywhere. Fully compliant with the OTEL standard.',
  },
  {
    icon: '/img/grafana.svg',
    title: 'Grafana-ready',
    description:
      'Visualize in seconds. Designed to integrate seamlessly with Grafana dashboards.',
  },
];

export default function FeatureCards(): ReactNode {
  return (
    <section className="py-24 px-6" style={{ backgroundColor: '#0a0a0a' }}>
      <div className="max-w-5xl mx-auto">
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          {features.map(({ icon, title, description }) => (
            <div key={title} className="feature-card">
              <img src={icon} alt={title} className="w-10 h-10 mb-5 opacity-80" />
              <h3 className="text-base font-semibold mb-2" style={{ color: '#ededed' }}>
                {title}
              </h3>
              <p className="text-sm leading-relaxed" style={{ color: '#888888' }}>
                {description}
              </p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

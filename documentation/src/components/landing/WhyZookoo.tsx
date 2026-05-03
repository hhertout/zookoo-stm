import type { ReactNode } from 'react';
import Link from '@docusaurus/Link';

const args = [
  {
    title: 'Single binary, zero dependencies',
    description: 'Deploy anywhere in seconds. No runtime, no agent manager, no overhead.',
  },
  {
    title: 'Rust performance',
    description:
      'Predictable resource usage with minimal CPU and memory footprint under load.',
  },
  {
    title: 'Flexible exporters',
    description:
      'Send data to OpenTelemetry, Prometheus Remote Write, or TimescaleDB — your choice.',
  },
];

export default function WhyZookoo(): ReactNode {
  return (
    <section className="py-24 px-6" style={{ backgroundColor: '#111111' }}>
      <div className="max-w-5xl mx-auto">
        <p
          className="text-sm font-semibold uppercase tracking-widest mb-4"
          style={{ color: '#f97316' }}
        >
          Why Zookoo?
        </p>
        <h2
          className="text-3xl md:text-4xl font-semibold mb-4 leading-tight"
          style={{ color: '#ededed', letterSpacing: '-0.01em' }}
        >
          Stop juggling Prometheus relabeling rules
          <br className="hidden md:block" />
          just to monitor a single endpoint.
        </h2>
        <p className="text-base mb-12" style={{ color: '#888888', maxWidth: '560px' }}>
          Zookoo is a Blackbox Exporter alternative built for teams who want
          simplicity, performance, and full control over their observability pipeline.
        </p>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-12">
          {args.map(({ title, description }) => (
            <div
              key={title}
              className="p-6 rounded-xl"
              style={{ border: '1px solid #1e1e1e', backgroundColor: '#0a0a0a' }}
            >
              <div
                className="w-2 h-2 rounded-full mb-4"
                style={{ backgroundColor: '#f97316' }}
              />
              <h3 className="text-base font-semibold mb-2" style={{ color: '#ededed' }}>
                {title}
              </h3>
              <p className="text-sm leading-relaxed" style={{ color: '#888888' }}>
                {description}
              </p>
            </div>
          ))}
        </div>

        <div
          className="inline-flex items-center gap-3 px-4 py-3 rounded-lg text-sm"
          style={{ border: '1px solid #1e1e1e', backgroundColor: '#0a0a0a' }}
        >
          <span
            className="px-2 py-0.5 rounded text-xs font-semibold uppercase tracking-wide"
            style={{ backgroundColor: 'rgba(249,115,22,0.15)', color: '#f97316' }}
          >
            Open Source
          </span>
          <span style={{ color: '#888888' }}>Fully open source. MIT licensed.</span>
          <Link
            to="https://github.com/hhertout/zookoo-stm"
            className="font-medium"
            style={{ color: '#f97316' }}
          >
            View on GitHub →
          </Link>
        </div>
      </div>
    </section>
  );
}

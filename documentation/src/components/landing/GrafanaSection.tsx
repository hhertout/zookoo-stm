import type { ReactNode } from 'react';
import Link from '@docusaurus/Link';

export default function GrafanaSection(): ReactNode {
  return (
    <section
      className="py-24 px-6"
      style={{ backgroundColor: '#111111', borderTop: '1px solid #1e1e1e' }}
    >
      <div className="max-w-5xl mx-auto grid grid-cols-1 lg:grid-cols-2 gap-12 items-center">
        {/* Left: text */}
        <div>
          <p
            className="text-sm font-semibold uppercase tracking-widest mb-4"
            style={{ color: '#f97316' }}
          >
            Observability
          </p>
          <h2
            className="text-3xl md:text-4xl font-semibold mb-6 leading-tight"
            style={{ color: '#ededed', letterSpacing: '-0.01em' }}
          >
            Explore your metrics on your favorite platform.
            <br />
            <span style={{ color: '#f97316' }}>Get alerted when something's wrong.</span>
          </h2>
          <p className="text-base mb-8 leading-relaxed" style={{ color: '#888888', maxWidth: '480px' }}>
            Zookoo ships metrics directly into your observability stack.
            Visualize latency, availability, and error rates in Grafana,
            and set up alerts so your team knows before your users do.
          </p>
          <div className="flex flex-col sm:flex-row gap-4">
            <Link to="/docs/intro" className="landing-btn-primary">
              View the docs →
            </Link>
          </div>
        </div>

        {/* Right: mock dashboard card */}
        <div
          className="rounded-2xl p-6 flex flex-col gap-4"
          style={{ border: '1px solid #1e1e1e', backgroundColor: '#0a0a0a' }}
        >
          {/* Fake chart header */}
          <div className="flex items-center justify-between">
            <div>
              <p className="text-xs uppercase tracking-widest mb-1" style={{ color: '#555555' }}>HTTP probe</p>
              <p className="text-lg font-semibold" style={{ color: '#ededed' }}>Response time</p>
            </div>
            <span
              className="px-2 py-1 rounded-md text-xs font-semibold"
              style={{ backgroundColor: 'rgba(34,197,94,0.12)', color: '#22c55e' }}
            >
              ● UP
            </span>
          </div>

          {/* Fake sparkline */}
          <svg viewBox="0 0 300 80" className="w-full" style={{ height: '80px' }}>
            <defs>
              <linearGradient id="sparkGrad" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stopColor="#f97316" stopOpacity="0.25"/>
                <stop offset="100%" stopColor="#f97316" stopOpacity="0"/>
              </linearGradient>
            </defs>
            <path
              d="M0 60 C20 55 40 45 60 48 S100 35 120 38 S160 28 180 32 S220 22 240 25 S270 30 300 20"
              fill="none"
              stroke="#f97316"
              strokeWidth="2"
            />
            <path
              d="M0 60 C20 55 40 45 60 48 S100 35 120 38 S160 28 180 32 S220 22 240 25 S270 30 300 20 L300 80 L0 80 Z"
              fill="url(#sparkGrad)"
            />
          </svg>

          {/* Stats row */}
          <div className="grid grid-cols-3 gap-3">
            {[
              { label: 'Avg latency', value: '42 ms' },
              { label: 'Uptime', value: '99.98%' },
              { label: 'Checks / min', value: '2' },
            ].map(({ label, value }) => (
              <div
                key={label}
                className="p-3 rounded-lg"
                style={{ backgroundColor: '#111111', border: '1px solid #1e1e1e' }}
              >
                <p className="text-xs mb-1" style={{ color: '#555555' }}>{label}</p>
                <p className="text-sm font-semibold" style={{ color: '#ededed' }}>{value}</p>
              </div>
            ))}
          </div>

          {/* Alert row */}
          <div
            className="flex items-center gap-3 px-4 py-3 rounded-lg"
            style={{ backgroundColor: 'rgba(249,115,22,0.06)', border: '1px solid rgba(249,115,22,0.2)' }}
          >
            <span style={{ color: '#f97316' }}>⚠</span>
            <div>
              <p className="text-xs font-semibold" style={{ color: '#f97316' }}>Alert rule triggered</p>
              <p className="text-xs" style={{ color: '#888888' }}>Latency exceeded 500ms threshold on eu-west-1</p>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}

import type { ReactNode } from 'react';
import Link from '@docusaurus/Link';

export default function Hero(): ReactNode {
  return (
    <section
      className="relative flex min-h-screen items-center justify-center overflow-hidden"
      style={{ backgroundColor: '#0a0a0a' }}
    >
      {/* Dot grid */}
      <div
        className="absolute inset-0 pointer-events-none"
        style={{
          backgroundImage: 'radial-gradient(circle, #1e1e1e 1px, transparent 1px)',
          backgroundSize: '24px 24px',
        }}
      />
      {/* Orange glow */}
      <div
        className="absolute inset-0 pointer-events-none"
        style={{
          background:
            'radial-gradient(ellipse 80% 50% at 50% -10%, rgba(249,115,22,0.10) 0%, transparent 70%)',
        }}
      />
      {/* Content */}
      <div className="relative z-10 flex flex-col items-center text-center px-6 max-w-4xl mx-auto">
        <img
          src="/img/zookoo_backgroundless.png"
          alt="Zookoo"
          className="w-96 h-96 mb-10"
          style={{ filter: 'drop-shadow(0 0 24px rgba(249,115,22,0.25))' }}
        />
        <h1
          className="text-5xl md:text-7xl font-semibold leading-tight mb-6"
          style={{
            color: '#ededed',
            letterSpacing: '-0.02em',
            fontFamily: 'Inter, system-ui, sans-serif',
          }}
        >
          Synthetic monitoring,
          <br />
          <span style={{ color: '#f97316' }}>built for speed.</span>
        </h1>
        <p
          className="text-lg md:text-xl max-w-2xl mb-10 leading-relaxed"
          style={{ color: '#888888' }}
        >
          A lightweight, OpenTelemetry-native monitoring agent written in Rust.
          One config file. Zero overhead.
        </p>
        <div className="flex flex-col sm:flex-row gap-4 items-center">
          <Link to="/docs/category/quick-start" className="landing-btn-primary">
            Get started →
          </Link>
          <Link
            to="https://github.com/hhertout/zookoo-stm"
            className="landing-btn-secondary"
          >
            View on GitHub
          </Link>
        </div>
      </div>
    </section>
  );
}

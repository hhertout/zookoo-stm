import type { ReactNode } from 'react';
import CodeBlock from '@theme/CodeBlock';

const hclExample = `defaults {
  log_level    = "info"
  probe_zone   = "eu-west-1"
  service_name = "zookoo"

  probe_location {
    latitude  = 48.858370
    longitude = 2.29448
  }
}

probe "http" "google_check" {
  scrape_interval = "30s"
  targets = [
    {
      url                  = "https://www.google.com"
      method               = "GET"
      expected_status_code = 200
      labels = {
        service = "google"
        env     = "prod"
      }
    }
  ]
  forward_to = [exporter.otlp.default]
}

exporter "otlp" "default" {
  url          = "http://localhost:4317"
  tls_insecure = true
}`;

const bullets = [
  'Define probes, exporters, and discovery in one place',
  'Assign custom labels and scrape intervals per target',
  'Load targets dynamically from a JSON file or API',
];

export default function ConfigShowcase(): ReactNode {
  return (
    <section
      className="py-24 px-6"
      style={{ backgroundColor: '#0a0a0a', borderTop: '1px solid #1e1e1e' }}
    >
      <div className="max-w-6xl mx-auto grid grid-cols-1 lg:grid-cols-2 gap-12 items-center">
        <div>
          <p
            className="text-sm font-semibold uppercase tracking-widest mb-4"
            style={{ color: '#f97316' }}
          >
            Configuration
          </p>
          <h2
            className="text-3xl md:text-4xl font-semibold mb-6 leading-tight"
            style={{ color: '#ededed', letterSpacing: '-0.01em' }}
          >
            One file.
            <br />
            Everything configured.
          </h2>
          <ul className="space-y-4">
            {bullets.map((bullet) => (
              <li
                key={bullet}
                className="flex items-start gap-3 text-sm"
                style={{ color: '#888888' }}
              >
                <span
                  className="mt-1.5 flex-shrink-0 w-1.5 h-1.5 rounded-full"
                  style={{ backgroundColor: '#f97316' }}
                />
                {bullet}
              </li>
            ))}
          </ul>
        </div>
        <div
          className="overflow-hidden rounded-xl"
          style={{ border: '1px solid #1e1e1e' }}
        >
          <CodeBlock language="hcl">
            {hclExample}
          </CodeBlock>
        </div>
      </div>
    </section>
  );
}

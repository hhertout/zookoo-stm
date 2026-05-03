import type { ReactNode } from 'react';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';
import Hero from '@site/src/components/landing/Hero';
import WhyZookoo from '@site/src/components/landing/WhyZookoo';
import FeatureCards from '@site/src/components/landing/FeatureCards';
import ConfigShowcase from '@site/src/components/landing/ConfigShowcase';
import PortalCards from '@site/src/components/landing/PortalCards';
import GrafanaSection from '@site/src/components/landing/GrafanaSection';

export default function Home(): ReactNode {
  const { siteConfig } = useDocusaurusContext();
  return (
    <Layout
      title={siteConfig.title}
      description="A lightweight, OpenTelemetry-native monitoring agent written in Rust."
    >
      <Hero />
      <WhyZookoo />
      <FeatureCards />
      <GrafanaSection />
      <ConfigShowcase />
      <PortalCards />
    </Layout>
  );
}

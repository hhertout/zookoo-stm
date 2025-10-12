import type {ReactNode} from 'react';
import clsx from 'clsx';
import Heading from '@theme/Heading';
import styles from './styles.module.css';

type FeatureItem = {
  title: string;
  Svg: React.ComponentType<React.ComponentProps<'svg'>>;
  description: ReactNode;
};

const FeatureList: FeatureItem[] = [
  {
    title: 'Easy to Use',
    Svg: require('@site/static/img/undraw_docusaurus_mountain.svg').default,
    description: (
      <>
        Zookoo is designed to be user-friendly and easy to set up. With a
        simple configuration file, you can quickly define your monitoring targets
        and start collecting metrics. Whether you are a developer or an
        operations engineer, Zookoo makes synthetic monitoring accessible
        and straightforward.
      </>
    ),
  },
  {
    title: 'Open Telemetry Complient',
    Svg: require('@site/static/img/undraw_docusaurus_tree.svg').default,
    description: (
      <>
        Zookoo is fully compliant with Open Telemetry standards, allowing
        you to collect and export metrics seamlessly. It supports various
        exporters, based on Prometheus metrics.
        Experience the power of Open Telemetry with Zookoo for comprehensive
        insights into your applications and services.
      </>
    ),
  },
  {
    title: 'Powered by Rust',
    Svg: require('@site/static/img/undraw_docusaurus_react.svg').default,
    description: (
      <>
        Blazingly fast and efficient, Zookoo is built with Rust to
        ensure high performance and low resource usage. It leverages the power of
        Rust to provide a robust and reliable synthetic monitoring solution.
        With Zookoo, you can monitor your applications and services with
        confidence, knowing that it is built on a solid foundation that can handle
        the demands of modern software environments.
      </>
    ),
  },
];

function Feature({title, Svg, description}: FeatureItem) {
  return (
    <div className={clsx('col col--4')}>
      <div className="text--center">
        <Svg className={styles.featureSvg} role="img" />
      </div>
      <div className="text--center padding-horiz--md">
        <Heading as="h3">{title}</Heading>
        <p>{description}</p>
      </div>
    </div>
  );
}

export default function HomepageFeatures(): ReactNode {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="row">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}

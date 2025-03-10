import React from "react";
import clsx from "clsx";
import Layout from "@theme/Layout";
import Link from "@docusaurus/Link";
import useDocusaurusContext from "@docusaurus/useDocusaurusContext";
import styles from "./index.module.css";
import HomepageFeatures from "../components/HomepageFeatures";
// @ts-ignore
import GetStarted from "../components/GetStarted.md";

function HomepageHeader() {
  const { siteConfig } = useDocusaurusContext();
  return (
    <header className={clsx("hero hero--dark", styles.heroBanner)}>
      <div className="container">
        <img src="img/cra-logo-rust-white.svg" width={170} />
        <h1 className="hero__title">{siteConfig.title}</h1>
        <p className="hero__subtitle">{siteConfig.tagline}</p>
        <div className={styles.buttons}>
          <Link
            className="button button--outline button--primary button--lg"
            to="/docs/intro"
          >
            Get Started
          </Link>
        </div>
      </div>
    </header>
  );
}

export default function Home(): JSX.Element {
  const { siteConfig } = useDocusaurusContext();
  return (
    <Layout
      title={`${siteConfig.title}`}
      description="Description will go into a meta tag in <head />"
    >
      <HomepageHeader />
      <main>
        {/* <HomepageFeatures /> */}
        <div className="getting-started">
          <div className="container padding-vert--xl text--left">
            <div className="row">
              <div className="col col--4 col--offset-1">
                <h2>Get started in seconds</h2>
                <GetStarted />
              </div>
              <div className="col col--5 col--offset-1">
                <a href="https://github.com/Wulf/create-rust-app/blob/main/docs/create-rust-app-v2.mp4">
                  <img src="img/create-rust-app-v2.gif" />
                </a>
              </div>
            </div>
          </div>
        </div>
        <div>
          <div className="container padding-vert--md text--center">
            <p>
              Note: this documentation site was inspired by create-react-app.dev
              and the two are not related.
            </p>
          </div>
        </div>
      </main>
    </Layout>
  );
}

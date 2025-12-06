import json

# 200 base domains
base_urls = [
    "google.com", "github.com", "microsoft.com", "apple.com", "amazon.com",
    "facebook.com", "twitter.com", "linkedin.com", "instagram.com", "youtube.com",
    "netflix.com", "spotify.com", "reddit.com", "wikipedia.org", "cloudflare.com",
    "digitalocean.com", "heroku.com", "vercel.com", "netlify.com", "docker.com",
    "kubernetes.io", "terraform.io", "ansible.com", "jenkins.io", "gitlab.com",
    "bitbucket.org", "stackoverflow.com", "medium.com", "dev.to", "producthunt.com",
    "dribbble.com", "behance.net", "figma.com", "canva.com", "notion.so",
    "slack.com", "discord.com", "zoom.us", "dropbox.com", "box.com",
    "icloud.com", "stripe.com", "paypal.com", "shopify.com", "wix.com",
    "squarespace.com", "wordpress.com", "blogger.com", "tumblr.com", "pinterest.com",
    "tiktok.com", "snapchat.com", "whatsapp.com", "telegram.org", "signal.org",
    "proton.me", "fastmail.com", "mailchimp.com", "sendgrid.com", "twilio.com",
    "auth0.com", "okta.com", "datadog.com", "newrelic.com", "splunk.com",
    "elastic.co", "mongodb.com", "postgresql.org", "mysql.com", "redis.io",
    "memcached.org", "rabbitmq.com", "nginx.org", "apache.org", "prometheus.io",
    "grafana.com", "influxdata.com", "consul.io", "nomadproject.io", "packer.io",
    "vagrantup.com", "helm.sh", "istio.io", "envoyproxy.io", "linkerd.io",
    "fluxcd.io", "crossplane.io", "kustomize.io", "rancher.com", "portainer.io",
    "traefik.io", "caddyserver.com", "oracle.com", "ibm.com", "salesforce.com",
    "adobe.com", "atlassian.com", "jetbrains.com", "npmjs.com", "pypi.org",
    "rubygems.org", "nuget.org", "gradle.org", "rust-lang.org", "golang.org",
    "python.org", "nodejs.org", "php.net", "ruby-lang.org", "swift.org",
    "kotlinlang.org", "typescriptlang.org", "dart.dev", "flutter.dev", "reactjs.org",
    "vuejs.org", "angular.io", "svelte.dev", "nextjs.org", "nuxt.com",
    "remix.run", "astro.build", "tailwindcss.com", "getbootstrap.com", "chakra-ui.com",
    "deno.land", "bun.sh", "vitejs.dev", "webpack.js.org", "rollupjs.org",
    "turbo.build", "nx.dev", "pnpm.io", "yarnpkg.com", "brew.sh",
    "ubuntu.com", "debian.org", "fedoraproject.org", "archlinux.org", "nixos.org",
    "freebsd.org", "openbsd.org", "alpinelinux.org", "rockylinux.org", "almalinux.org",
    "redhat.com", "suse.com", "vmware.com", "proxmox.com", "synology.com",
    "ubnt.com", "cisco.com", "akamai.com", "fastly.com", "jsdelivr.com",
    "unpkg.com", "cdnjs.com", "vultr.com", "linode.com", "hetzner.com",
    "ovhcloud.com", "scaleway.com", "contabo.com", "upcloud.com", "hostinger.com",
    "bluehost.com", "godaddy.com", "namecheap.com", "hover.com", "porkbun.com",
    "cloudns.net", "easydns.com", "sentry.io", "launchdarkly.com", "segment.com",
    "amplitude.com", "mixpanel.com", "heap.io", "hotjar.com", "fullstory.com",
    "logrocket.com", "bugsnag.com", "rollbar.com", "airbrake.io", "honeybadger.io"
]

# Generate 1000 entries with path variations
entries = []
paths = ["", "/about", "/docs", "/api", "/status"]

for i in range(1000):
    url_idx = i % len(base_urls)
    path_idx = (i // len(base_urls)) % len(paths)
    
    domain = base_urls[url_idx]
    path = paths[path_idx]
    
    service = domain.split('.')[0].replace('-', '_')
    if path:
        service = f"{service}_{path.strip('/')}"
    
    full_url = f"https://{domain}{path}"
    entry = {"url": full_url, "labels": {"service": service}}
    entries.append(entry)

with open('targets-1000.json', 'w') as f:
    json.dump(entries, f, indent=2)

print(f"Created targets-1000.json with {len(entries)} URLs")

---
title: Deploy
---

# Deploying your app

> Note: This page is currently a stub. Help us write it!

## GitHub Pages

## Netlify

## Vercel

## Cloudflare Workers

## fly.io

## Azure

## AWS

## Google Cloud

## deed.page

[deed.page](https://deed.page) hosts static files without an account. Build the
release version of your app, pack the `public/` folder that `cargo-sycamore`
writes to `dist/`, and upload the archive with a single request:

```bash
cargo sycamore build --release
tar -C dist/public -czf site.tar.gz .
curl -T site.tar.gz "https://deed.page/v1/sites?slug=my-sycamore-app"
```

Pick your own slug; if it is already taken, the request returns `409` and you
can try another one. The response contains the URL of your site
(`https://my-sycamore-app.deed.page/`) and a token. The token is only shown
once, so save it: you need to send it as `Authorization: Bearer <token>` to
update or delete the site later. Because the app is served from the root of its
own subdomain, the default `public-url` of `/` works as is. See the
[deed.page docs](https://deed.page/docs) for size limits and the rest of the
API.

Missing a service? Contribute to the docs by adding it!

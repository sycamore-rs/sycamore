---
title: Announcing Sycamore v0.9.0
description:
  Reactivity v3, View v2, resources API and suspense, SSR streaming, attributes
  passthrough, new website, and more!
date: 2024-10-24
---

# Announcing Sycamore v0.9.0

I'm happy to announce the release of Sycamore v0.9.0. Sycamore is a reactive
Rust UI framework for building web apps using WebAssembly. This release is by
far the biggest Sycamore release we've ever had, with tons of new features and
improvements.

Sycamore's community has also grown a lot since the v0.8.0 release. We've gone
from just over _1.0k_ stars to _2.8k_ stars on GitHub. What used to be just over
_350_ discord members has now grown to _626_!

## A shinny new website

We now have a shinny new website along with a shinny new domain:
[sycamore.dev](https://sycamore.dev)! This was long overdue. We were previously
using a Netlify subdomain so this change hopefully makes Sycamore look more
legitimate. The old website had a bunch of issues such as buggy navigation, no
server side rendering support, and an awkward layout. This new website redesign
fixes many of those things. The old docs are still available at the old website
but the index page will now automatically redirect to the new website.

A lot of the content has also been rewritten and updated for this new version of
Sycamore. This includes a brand new "Introduction" section which helps guide you
through creating your first Sycamore app, a simple Todo manager. This introduces
various topics such as the view macro, the basics of reactivity, and how
rendering lists work. This will hopefully help new users interested in Sycamore
to get started with the main concepts.

There are still currently a few sections of the docs that needs writting or
simply needs a few more details. You can help us out by contributing to the
docs! Simply go to the relevant page and click on "Edit this page on GitHub" at
the bottom and send us a Pull Request.

## Reactivity v3

What is probably the biggest new feature of this release is our new reactivity
system, dubbed _Reactivity v3_! In Reactivity v2, introduced in the
[v0.8](/post/announcing-v0-8-0), we eliminated the need for cloning signals and
other reactive primitives into closures. This, however, came at the expense of
introducing lifetimes for tracking whether a signal was alive and could be
accessed.

## View v2

## Resources and Suspense

## SSR streaming

## Attribute passthrough

## Smaller changes

### Reactive `NodeRef`

### Optional attributes

## The future of Sycamore

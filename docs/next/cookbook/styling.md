---
title: Styling with CSS
---

# Styling with CSS

Styling is definitely important for a modern web app. Sycamore is not
opinionated when it comes to styling and lets you use whichever method works
best for you. Below are some common options.

## Raw CSS files

The simplest is just to use raw CSS files. To do so, just add the relevant
`<link>` tag to our `index.html`.

## CSS framework

To use a CSS framework, just include the CSS file inside the static asset
directory and include it in your `index.html`.

## Tailwind CSS

One particular kind of CSS frameworks worthy of note are utility-first CSS
frameworks (such as [Tailwind CSS](https://tailwindcss.com/) or
[Windi CSS](https://windicss.org/)).

`cargo-sycamore` has built-in support for Tailwind CSS. Add the following line
to your `Sycamore.toml`

```toml
[tools.tailwindcss]
enable = true
input = "index.css"
```

and create an `index.css` file in your project root with the contents

```css
@import "tailwindcss";
```

Now you should be able to use Tailwind CSS directly in your Sycamore components!

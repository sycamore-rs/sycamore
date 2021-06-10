# Sycamore documentation website

This is the source code for the docs website hosted at [sycamore-rs.netlify.app](https://sycamore-rs.netlify.app)

The website itself is a Sycamore app. View the Rust code in `src/`. All the documentation itself can be found in the `markdown/` folder.

The docs website use [TailwindCSS](https://tailwindcss.com) for styling. This unfortunately means we need to use npm and friends. To make development easier, use `npm run dev` to start the `trunk` dev server as well as `tailwindcss` but in [JIT mode](https://tailwindcss.com/docs/just-in-time-mode).

To make a production build of the docs website, use `npm run prod`.

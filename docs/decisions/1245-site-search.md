## #1245 - Docs site search

**What was decided.** Zola builds the search index (`build_search_index = true`,
`index_format = "elasticlunr_javascript"`, so `search_index.en.js`). The header
box in `site/templates/base.html` loads that index with elasticlunr 0.9.5
vendored at `site/static/vendor/elasticlunr.min.js` and its MIT licence beside
it. Result links are the page URLs Zola built from `base_url`, so the base
path is carried without a hard-coded host.

**Why.** The site must work with no third-party service at run time, and the
index is already produced by Zola, so the only addition is the browser side.
Zola's pinned version (v0.23.6 in release.yml) already emits this format.

**Considered and dropped.** A CDN script (cdnjs or jsdelivr): it would need a
network fetch on every visit and a pinned exact version anyway, so vendoring
was the cheaper choice. Full-page bodies in the index: dropped, the index
would grow past 200 KB per build of the guide; the body is cut to 2000
characters.

**Cost to change later.** Replacing elasticlunr means swapping the vendored
file and the inline script's `Index.load` and `search` calls. The search
options (`[search]` in `config.toml`) are one block.

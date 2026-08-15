import { HtmlBasePlugin } from "@11ty/eleventy";
import techdoc from "eleventy-plugin-techdoc";

const postCardStart = `    <article class="blog-post">
      <a href="{{ post.url }}" class="post-title">{{ post.data.title }}</a>`;
const postCardWithImage = `    <article class="blog-post">
      {% if post.data.leadImage %}
      <a href="{{ post.url }}" class="post-image-link">
        <img class="post-card-image" src="{{ post.data.leadImage.src }}" alt="{{ post.data.leadImage.alt }}" width="{{ post.data.leadImage.width }}" height="{{ post.data.leadImage.height }}" loading="lazy">
      </a>
      {% endif %}
      <a href="{{ post.url }}" class="post-title">{{ post.data.title }}</a>`;

const proseLayoutReplacements = [
  {
    key: "_includes/layouts/docs.njk",
    replacements: [
      ['<div class="docs-layout">', '<div class="docs-layout prose-layout">'],
      ['<article class="docs-content">', '<article class="docs-content prose prose-content">'],
    ],
  },
  {
    key: "_includes/layouts/blog.njk",
    replacements: [
      ['<article class="blog-post">', '<article class="blog-post prose-layout prose prose-content">'],
      [
        '    <div class="blog-post-content">',
        `    {% if leadImage %}
    <figure class="blog-post-lead">
      <img src="{{ leadImage.src }}" alt="{{ leadImage.alt }}" width="{{ leadImage.width }}" height="{{ leadImage.height }}" decoding="async" fetchpriority="high">
    </figure>
    {% endif %}

    <div class="blog-post-content">`,
      ],
    ],
  },
  {
    key: "blog/index.njk",
    replacements: [
      ['<div class="blog-container">', '<div class="blog-container prose-layout">'],
      [postCardStart, postCardWithImage],
    ],
  },
  ...["blog/tags-pages.njk", "blog/categories-pages.njk"].map((key) => ({
    key,
    replacements: [[postCardStart, postCardWithImage]],
  })),
];

function applyProseLayouts(eleventyConfig) {
  const templates = eleventyConfig.virtualTemplates;
  if (!templates) return;

  const base = templates["_includes/layouts/base.njk"];
  if (base) {
    const headerStart = base.content.indexOf('  <header class="site-header">');
    const headerEnd = base.content.indexOf("  </header>", headerStart);
    if (headerStart >= 0 && headerEnd >= 0) {
      base.content = `${base.content.slice(0, headerStart)}  {% include "partials/site-navigation.njk" %}\n${base.content.slice(headerEnd + 12)}`;
    }
    base.content = base.content.replace(
      "  {% block scripts %}{% endblock %}",
      '  <script defer src="/assets/vendor/mermaid.min.js"></script>\n  <script defer src="/assets/js/mermaid-init.js"></script>\n\n  {% block scripts %}{% endblock %}',
    );
  }

  for (const override of proseLayoutReplacements) {
    const template = templates[override.key];
    if (!template) continue;
    template.content = override.replacements.reduce(
      (content, [from, to]) => content.replace(from, to),
      template.content,
    );
  }
}

export default function configureEleventy(eleventyConfig) {
  eleventyConfig.addPlugin(techdoc, {
    site: {
      name: "dmx",
      url: "https://dmx.dev",
      description:
        "Dart code generation on save: copyWith, equality, toString and typed JSON, written into the class you annotated. No part files.",
      ogImage: "/assets/images/dmx-mark.png",
      ogImageWidth: "1200",
      ogImageHeight: "1200",
    },
    features: {
      blog: true,
      docs: true,
      darkMode: true,
      i18n: false,
    },
  });
  eleventyConfig.amendLibrary("md", (markdown) => {
    const fence = markdown.renderer.rules.fence;
    markdown.renderer.rules.fence = (tokens, index, options, environment, renderer) => {
      const token = tokens[index];
      if (token.info.trim() === "mermaid") {
        return `<pre class="mermaid">${markdown.utils.escapeHtml(token.content)}</pre>`;
      }
      return fence(tokens, index, options, environment, renderer);
    };
  });
  eleventyConfig.addPlugin(applyProseLayouts);
  eleventyConfig.addPlugin(HtmlBasePlugin);

  // GitHub Pages reads the custom domain from this file in the published
  // artifact, which is what puts the site at the root of dmx.dev and makes
  // `pathPrefix: "/"` correct [playground.hosting].
  eleventyConfig.addPassthroughCopy("src/CNAME");
  eleventyConfig.addPassthroughCopy("src/assets/css/docs.css");
  eleventyConfig.addPassthroughCopy({
    "src/styles/base.css": "assets/css/base.css",
    "src/styles/layout.css": "assets/css/layout.css",
    "src/styles/responsive.css": "assets/css/responsive.css",
    "src/styles/sections.css": "assets/css/sections.css",
  });
  eleventyConfig.addPassthroughCopy({
    "node_modules/mermaid/dist/mermaid.min.js": "assets/vendor/mermaid.min.js",
    "src/assets/js/mermaid-init.js": "assets/js/mermaid-init.js",
    "src/assets/images/blog": "assets/images/blog",
    "src/assets/images/dmx-mark.svg": "assets/images/dmx-mark.svg",
    "src/assets/images/dmx-mark.png": "assets/images/dmx-mark.png",
    "src/assets/images/favicon-32.png": "assets/images/favicon-32.png",
    "src/assets/images/apple-touch-icon.png": "assets/images/apple-touch-icon.png",
  });

  return {
    dir: { input: "src", output: "dist" },
    markdownTemplateEngine: "njk",
    pathPrefix: "/",
  };
}
